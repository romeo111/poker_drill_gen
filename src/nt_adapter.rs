use serde_json::{json, Value};
use crate::training_engine::models::{Card, Position, TrainingScenario};

/// Convert a `Card` to the string format expected by the Angular client.
/// rank 10 → "10s", all others → "Ts" style from Display, i.e. "As", "Kh".
fn to_client_card(c: &Card) -> String {
    if c.rank.0 == 10 {
        format!("10{}", c.suit)
    } else {
        c.to_string()
    }
}

/// Build the 5-slot community_cards array for NtTableState.
fn community_cards(board: &[Card]) -> Value {
    let mut slots = Vec::with_capacity(5);
    for i in 0..5usize {
        let card_str = if i < board.len() {
            to_client_card(&board[i])
        } else {
            String::new()
        };
        slots.push(json!({
            "id": i,
            "card": card_str,
            "isCombination": false,
            "isNoCombination": false
        }));
    }
    Value::Array(slots)
}

/// Derive the NtTableState game_state string from number of board cards.
fn game_state_str(board_len: usize) -> &'static str {
    match board_len {
        0 => "PreFlop",
        3 => "Flop",
        4 => "Turn",
        _ => "River",
    }
}

/// Build a hidden (opponent) card entry.
fn hidden_card(id: usize) -> Value {
    json!({ "id": id, "card": "b", "isCombination": false, "isNoCombination": false })
}

/// Build a visible card entry.
fn visible_card(id: usize, c: &Card) -> Value {
    json!({ "id": id, "card": to_client_card(c), "isCombination": false, "isNoCombination": false })
}

/// Build the pre_actions block (all false).
fn pre_actions() -> Value {
    json!({
        "check": false,
        "call":  false,
        "fold":  false,
        "raise": false,
        "bet":   false
    })
}

/// Build an empty seat entry.
fn empty_seat(seat_idx: u8) -> Value {
    json!({
        "seat_idx": seat_idx,
        "player_id": 0,
        "is_playing": false,
        "is_active": false,
        "is_folded": false,
        "is_all_in": false,
        "is_in_sit_out": false,
        "rebuy_time": null,
        "stack": { "value": 0, "currency": "xPKR" },
        "name": "",
        "bet": 0,
        "last_action": "",
        "cards": [],
        "action_option": { "actions": [], "min_bet": 0, "max_bet": 0, "call_amount": 0 },
        "pre_actions": pre_actions(),
        "country": null,
        "image": null,
        "isShowdown": false,
        "is_active": false,
        "emoji": null
    })
}

/// Determine the seat_idx_bb/sb/button values for data_state.
/// Returns (bb_seat, sb_seat, btn_seat) — 0 means "not applicable" for that slot.
fn position_seats(hero_pos: Position, villain_pos: Position) -> (u8, u8, u8) {
    let seat_for = |p: Position| -> u8 {
        if p == hero_pos { 1 } else { 2 }
    };

    let mut bb: u8 = 0;
    let mut sb: u8 = 0;
    let mut btn: u8 = 0;

    for pos in [hero_pos, villain_pos] {
        let s = seat_for(pos);
        match pos {
            Position::BB  => bb  = s,
            Position::SB  => sb  = s,
            Position::BTN => btn = s,
            _             => {}
        }
    }
    (bb, sb, btn)
}

/// Map a `TrainingScenario` to an NtTableState JSON object ready for the Angular client.
///
/// `hero_player_id` is the numeric player ID assigned to the viewing user.
pub fn to_nt_table_state(scenario: &TrainingScenario, hero_player_id: u32) -> Value {
    let setup = &scenario.table_setup;
    let board = &setup.board;

    // Find villain player state (first non-hero).
    let villain = setup.players.iter().find(|p| !p.is_hero);
    let villain_pos = villain.map(|v| v.position).unwrap_or(Position::BB);
    let villain_stack = villain.map(|v| v.stack).unwrap_or(100);
    let hero_player = setup.players.iter().find(|p| p.is_hero);
    let hero_stack = hero_player.map(|p| p.stack).unwrap_or(100);

    let (bb_seat, sb_seat, btn_seat) = position_seats(setup.hero_position, villain_pos);

    let game_state = game_state_str(board.len());

    // Pot and bet as chip values (raw u32 from the scenario).
    let pot = setup.pot_size as f64;
    let current_bet = setup.current_bet as f64;

    // Build seats array: indices 0-5 (0=empty, 1=hero, 2=villain, 3-5=empty).
    let hero_seat = json!({
        "seat_idx": 1,
        "player_id": hero_player_id,
        "is_playing": true,
        "is_active": true,
        "is_folded": false,
        "is_all_in": false,
        "is_in_sit_out": false,
        "rebuy_time": null,
        "stack": { "value": hero_stack, "currency": "xPKR" },
        "name": "You",
        "bet": 0,
        "last_action": "",
        "cards": [
            visible_card(0, &setup.hero_hand[0]),
            visible_card(1, &setup.hero_hand[1])
        ],
        "action_option": {
            "actions": [],
            "min_bet": 0,
            "max_bet": 0,
            "call_amount": current_bet
        },
        "pre_actions": pre_actions(),
        "country": null,
        "image": null,
        "isShowdown": false,
        "is_active": true,
        "emoji": null
    });

    let villain_last_action = if setup.current_bet > 0 { "Bet" } else { "" };

    let villain_seat = json!({
        "seat_idx": 2,
        "player_id": hero_player_id + 1,
        "is_playing": true,
        "is_active": false,
        "is_folded": false,
        "is_all_in": false,
        "is_in_sit_out": false,
        "rebuy_time": null,
        "stack": { "value": villain_stack, "currency": "xPKR" },
        "name": "Villain",
        "bet": current_bet,
        "last_action": villain_last_action,
        "cards": [hidden_card(0), hidden_card(1)],
        "action_option": { "actions": [], "min_bet": 0, "max_bet": 0, "call_amount": 0 },
        "pre_actions": pre_actions(),
        "country": null,
        "image": null,
        "isShowdown": false,
        "is_active": false,
        "emoji": null
    });

    json!({
        "nt_type": "NtTableState",
        "player_id": hero_player_id,
        "pool_id": 0,
        "data": {
            "data_state": {
                "table_id": 9999,
                "display_table_id": format!("training/{}", scenario.scenario_id),
                "active_seat_idx": 1,
                "seat_idx_bb": bb_seat,
                "seat_idx_sb": sb_seat,
                "seat_idx_button": btn_seat,
                "pot": [pot],
                "sb_amount": 1.0_f64,
                "bb_amount": 2.0_f64,
                "action_time_limit": { "secs": 0, "nanos": 0 },
                "delay_type": "UserActionDelay",
                "pool_type": "CommonHoldem",
                "blitz": false,
                "spectating": false,
                "pots": [[{ "pot_id": 0, "value": pot, "displayValue": pot, "position": "" }]]
            },
            "table_state": {
                "game_state": game_state,
                "community_cards": community_cards(board),
                "showdown_state": { "first_seat_idx_to_show": 0, "winners": {} }
            },
            "seats_state": [
                empty_seat(0),
                hero_seat,
                villain_seat,
                empty_seat(3),
                empty_seat(4),
                empty_seat(5)
            ]
        },
        "service_type": "free"
    })
}
