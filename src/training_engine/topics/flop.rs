//! Flop topic generators: c-bet, pot odds, check-raise, semi-bluff, 3-bet pot c-bet.
//!
//! All five topics deal a 3-card flop and ask hero what to do.  Board texture
//! (from `evaluator::board_texture`) is the primary driver for c-bet sizing.
//! Draw classification (from `evaluator::classify_draw`) drives pot-odds,
//! semi-bluff, and check-raise decisions.
//!
//! ## Topics in this file
//!
//! - **T2 C-Bet** — Size a continuation bet based on board texture and range
//!   advantage (Dry → small, Wet → large, no advantage → check).
//! - **T3 Pot Odds** — Call or fold with a drawing hand by comparing pot odds
//!   to draw equity (flush draw ~35%, OESD ~32%, combo ~54%, gutshot ~17%).
//! - **T7 Check-Raise** — OOP (BB) on the flop: check-raise strong hands and
//!   combo draws, check-call medium holdings, fold when the board favours
//!   villain's range and hero has no draw.
//! - **T8 Semi-Bluff** — Raise with a draw on the flop as a semi-bluff to win
//!   the pot immediately or with a made hand on a later street.
//! - **T13 3-Bet Pot C-Bet** — C-bet sizing in a 3-bet pot (smaller SPR, higher
//!   stakes): bet on favourable textures, check back weak hands.

use rand::Rng;
use crate::training_engine::{
    evaluator::{
        board_texture, classify_draw, draw_equity_flop, has_flush_draw, has_straight_draw,
        hero_has_flush_draw, hero_has_straight_draw, required_equity,
        BoardTexture, DrawType,
    },
    helpers::{board_str, deal, hand_str},
    models::*,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Shared helper — draw description for Simple text style
// Used by generate_pot_odds (T3) and generate_semi_bluff (T8) to produce
// beginner-friendly explanations of what a "flush draw" or "OESD" means.
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_simple_label(dt: DrawType) -> &'static str {
    match dt {
        DrawType::FlushDraw => "flush draw (you need one more card of the same suit to make a flush)",
        DrawType::OESD      => "straight draw (you can complete a straight on either end)",
        DrawType::ComboDraw => "two-way draw (flush or straight possible)",
        DrawType::GutShot   => "inside straight draw (only one card completes your straight)",
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T2 — Postflop Continuation Bet (CB-)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn generate_cbet<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 3);

    let texture = board_texture(&board);

    // Stack / pot sizes
    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(60..=130), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    let hero_pos = if rng.gen_bool(0.5) { Position::BTN } else { Position::CO };

    let players = vec![
        PlayerState {
            seat: 1,
            position: Position::BB,
            stack,
            is_hero: false,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: hero_pos,
            stack,
            is_hero: true,
            is_active: true,
        },
    ];

    // Range advantage flag (simplified: hero has range adv from late pos on low boards)
    let lowest_rank = board.iter().map(|c| c.rank.0).min().unwrap_or(14);
    let hero_has_range_adv = hero_pos.is_late() && lowest_rank <= 8;

    let board_s = board_str(&board);
    let hand_s = hand_str(hero_hand);
    let pos_str = format!("{}", hero_pos);
    let texture_str = format!("{}", texture);

    let branch_key = match (&texture, hero_has_range_adv) {
        (BoardTexture::Dry, true)  => "Dry:RangeAdv".to_string(),
        (BoardTexture::Dry, false) => "Dry:NoRangeAdv".to_string(),
        (BoardTexture::SemiWet, _) => "SemiWet".to_string(),
        (BoardTexture::Wet, _)     => "Wet".to_string(),
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You bet before the flop and your opponent checked. You have {hand_s} in {pos_str}. \
             The first three cards: {board_s}. Pot: {pot} chips. Stack: {stack} chips. \
             Options: check, bet small (~{} chips), bet big (~{} chips), or overbet (~{} chips). What do you do?",
            pot / 3,
            pot * 3 / 4,
            pot * 5 / 4
        ),
        TextStyle::Technical => format!(
            "You raised preflop and are the aggressor. You hold {hand_s} on {pos_str}. \
             The flop comes {board_s} (a {texture_str} board). The pot is {pot} chips \
             ({pot_bb} BB). Your stack is {stack} chips ({stack_bb} BB). \
             Villain checks to you. Bet options: small (~33% pot = {} chips), \
             large (~75% pot = {} chips), or overbet (~125% pot = {} chips). What do you do?",
            pot / 3,
            pot * 3 / 4,
            pot * 5 / 4
        ),
    };

    let answers = build_cbet_answers(
        &hand_s, &pos_str, &texture_str, &board_s,
        texture.clone(), hero_has_range_adv, pot, stack_bb, difficulty,
        text_style,
    );

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: 0,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::PostflopContinuationBet,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

fn build_cbet_answers(
    hand_str: &str,
    pos_str: &str,
    texture_str: &str,
    board_str: &str,
    texture: BoardTexture,
    hero_range_adv: bool,
    pot: u32,
    stack_bb: u32,
    _difficulty: DifficultyLevel,
    text_style: TextStyle,
) -> Vec<AnswerOption> {
    // On dry boards with range advantage, 33% is often optimal.
    // On wet boards, larger sizing or check is better.
    let (correct_id, check_rationale, small_rationale, large_rationale, overbet_rationale) =
        match texture {
            BoardTexture::Dry if hero_range_adv => (
                "B",
                match text_style {
                    TextStyle::Simple => format!(
                        "Checking gives your opponent a free card. You're in a strong position here — a small bet is the better play."
                    ),
                    TextStyle::Technical => format!(
                        "Checking with {hand_str} on a {texture_str} board ({board_str}) sacrifices \
                         fold equity and gives villain a free card. From {pos_str} with range \
                         advantage, betting is better than checking."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. A small bet works well here. The board is dry (no likely draws), so a cheap bet is enough to keep the pressure on and collect chips."
                    ),
                    TextStyle::Technical => format!(
                        "A 33% pot c-bet on a {texture_str} board is correct here. It exploits your \
                         range advantage from {pos_str}, applies pressure at low risk, and denies \
                         equity to villain's backdoor draws and overcards."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Betting this big on a dry board is too much. A small bet gets the same job done for less risk."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot sizing on a {texture_str} board is unnecessarily large. Villain \
                         folds hands you beat and calls with hands that have equity, making this \
                         sizing -EV on a board where a small bet achieves the same goals."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Overbetting on a dry board is too aggressive — a small bet achieves the same goal more cheaply."
                    ),
                    TextStyle::Technical => format!(
                        "An overbet on a {texture_str} board from {pos_str} is exploitable. \
                         Villain's calling range will have enough equity against an overbet that \
                         you cannot profitably use this sizing as a bluff."
                    ),
                },
            ),
            BoardTexture::Dry => (
                "A",
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. Check here. The board is dry (no likely draws) and you don't have a big advantage. A free card costs you little and lets you see how the hand develops."
                    ),
                    TextStyle::Technical => format!(
                        "Checking with {hand_str} on a {texture_str} board ({board_str}) from \
                         {pos_str} is correct when you lack clear range advantage. A check allows \
                         you to control pot size and reassess on the turn."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "A small bet can work but checking is safer here — you don't have a clear advantage on this board."
                    ),
                    TextStyle::Technical => format!(
                        "A small c-bet can work here but without range advantage you may be \
                         building a pot when your range is symmetric with villain's. Checking \
                         back and re-evaluating is higher EV from {pos_str}."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Betting big here over-commits your chips. Check or fold is better when you don't have the advantage."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot c-bet on a {texture_str} board without range advantage \
                         over-commits with a polarized-looking line when your range may not \
                         support the sizing."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Overbetting without an advantage on this board is a big mistake. Check instead."
                    ),
                    TextStyle::Technical => format!(
                        "Overbetting a {texture_str} board without a clear nut advantage is a \
                         leak. Reserve overbets for boards where your range is significantly \
                         stronger than villain's."
                    ),
                },
            ),
            BoardTexture::SemiWet | BoardTexture::Wet => (
                "C",
                match text_style {
                    TextStyle::Simple => format!(
                        "Checking here lets your opponent draw to a better hand for free. Bet to make them pay."
                    ),
                    TextStyle::Technical => format!(
                        "Checking on a {texture_str} board ({board_str}) surrenders too much \
                         equity and gives villain free cards with flush/straight draws. From \
                         {pos_str} you should charge draws with a sizable bet."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "A small bet is too cheap — your opponent can afford to call and try to improve. Bet bigger to make it expensive."
                    ),
                    TextStyle::Technical => format!(
                        "A 33% pot bet on a {texture_str} board is too small — it gives villain \
                         correct pot odds to call with flush draws (~35% equity) without paying \
                         a premium, diluting your fold equity."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. Bet big! The board has draws (possible flush or straight). Make your opponent pay a lot to try to beat you."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot c-bet on a {texture_str} board is correct. It charges draws \
                         incorrect pot odds ({:.0}% required equity vs ~35% actual for flush draw), \
                         protects your hand, and maintains fold equity against weak pairs.",
                        crate::training_engine::evaluator::required_equity(
                            (pot as f32 * 0.75) as u32, pot
                        ) * 100.0
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "An overbet on the first three cards is usually too much too soon. A big bet (75%) already does the job."
                    ),
                    TextStyle::Technical => format!(
                        "An overbet on a {texture_str} board with {stack_bb} BB remaining can \
                         be used as a polarized bluff, but is generally reserved for the river \
                         or specific high-equity situations. On the flop it often folds too \
                         much equity and collapses your range."
                    ),
                },
            ),
        };

    vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct_id == "A",
            explanation: check_rationale,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Bet small".to_string(),
            is_correct: correct_id == "B",
            explanation: small_rationale,
        },
        AnswerOption {
            id: "C".to_string(),
            text: "Bet large".to_string(),
            is_correct: correct_id == "C",
            explanation: large_rationale,
        },
        AnswerOption {
            id: "D".to_string(),
            text: "Overbet".to_string(),
            is_correct: false,
            explanation: overbet_rationale,
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// T3 — Pot Odds & Equity (PO-)
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute hero draw equity for pot-odds decisions (supports 1 or 2 streets).
fn pot_odds_equity(draw: DrawType, streets: u8) -> f32 {
    match draw {
        DrawType::FlushDraw => crate::training_engine::evaluator::flush_draw_equity(streets),
        DrawType::OESD      => crate::training_engine::evaluator::oesd_equity(streets),
        DrawType::ComboDraw => crate::training_engine::evaluator::combo_draw_equity(streets),
        DrawType::GutShot   => if streets == 2 { 0.17 } else { 0.09 },
    }
}

pub fn generate_pot_odds<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 3);

    // Determine draw type from the actual board (best effort) or assign randomly
    let flush = has_flush_draw(&board);
    let straight = has_straight_draw(&board);
    let draw_type = match (flush, straight) {
        (true, true)  => DrawType::ComboDraw,
        (true, false) => DrawType::FlushDraw,
        (false, true) => DrawType::OESD,
        _             => DrawType::GutShot,
    };

    let bb = 2u32;
    let (pot_bb, bet_pct) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(8..=12u32), 0.50f32),
        DifficultyLevel::Intermediate => {
            let p = rng.gen_range(6..=20);
            let b = 0.33 + rng.gen::<f32>() * (1.0 - 0.33);
            (p, b)
        },
        DifficultyLevel::Advanced     => {
            let p = rng.gen_range(4..=30);
            let b = 0.25 + rng.gen::<f32>() * (1.5 - 0.25);
            (p, b)
        },
    };
    let pot = pot_bb * bb;
    let bet = (pot as f32 * bet_pct).round() as u32;
    let streets_remaining: u8 = 2; // flop scenario, two streets to come

    let req_eq = required_equity(bet, pot);
    let actual_eq = pot_odds_equity(draw_type, streets_remaining);
    let should_call = actual_eq >= req_eq;

    let draw_name = match draw_type {
        DrawType::FlushDraw => "FlushDraw",
        DrawType::OESD      => "OESD",
        DrawType::ComboDraw => "ComboDraw",
        DrawType::GutShot   => "GutShot",
    };
    let branch_key = format!("{}:{}", draw_name, if should_call { "Call" } else { "Fold" });

    let hand_s = hand_str(hero_hand);
    let board_s = board_str(&board);
    let hero_pos = Position::BB;

    let draw_type_label = format!("{}", draw_type);
    let draw_type_simple_label = draw_simple_label(draw_type);

    let question = match text_style {
        TextStyle::Simple => format!(
            "You have {hand_s} and are chasing a {draw_type_simple_label} after the first three cards: {board_s}. \
             Pot: {pot} chips. Your opponent bet {bet} chips. Do you call or fold?"
        ),
        TextStyle::Technical => format!(
            "You hold {hand_s} and have a {draw_type_label} on the flop {board_s}. \
             The pot is {pot} chips ({pot_bb} BB). Villain bets {bet} chips \
             ({:.0}% of pot). Do you call or fold?",
            bet_pct * 100.0
        ),
    };

    let call_explanation = match text_style {
        TextStyle::Simple => if should_call {
            format!(
                "Correct — call! You have a good chance of improving your hand ({:.0}% roughly), and the price to call is fair. You'll win enough when you hit to make this worthwhile.",
                actual_eq * 100.0
            )
        } else {
            format!(
                "Calling here is a mistake. Your hand ({draw_type_simple_label}) doesn't have a good enough chance of improving to make this call worth the price."
            )
        },
        TextStyle::Technical => format!(
            "Call analysis: Pot after call = {} chips. You are calling {bet} chips. \
             Required equity = {bet}/{} = {:.1}%. \
             Approximate {draw_type_label} equity with 2 streets = {:.1}%. \
             {} Therefore calling {} correct here.",
            pot + bet,
            pot + bet,
            req_eq * 100.0,
            actual_eq * 100.0,
            if should_call {
                "Your equity EXCEEDS the required equity."
            } else {
                "Your equity is BELOW the required equity."
            },
            if should_call { "IS" } else { "is NOT" },
        ),
    };

    let fold_explanation = match text_style {
        TextStyle::Simple => if !should_call {
            format!(
                "Correct — fold. Your hand ({draw_type_simple_label}) only improves roughly {:.0}% of the time, and the price to call is too high for those odds. Save your chips.",
                actual_eq * 100.0
            )
        } else {
            format!(
                "Folding is wrong here — your hand has a good enough chance of improving to make this call worth it."
            )
        },
        TextStyle::Technical => format!(
            "Fold analysis: You need {:.1}% equity to call (calling {bet} into a pot of {} chips). \
             Your {draw_type_label} has approximately {:.1}% equity with 2 cards to come. \
             {} Folding {} correct.",
            req_eq * 100.0,
            pot + bet,
            actual_eq * 100.0,
            if !should_call {
                "Since your equity is below the break-even threshold, folding preserves chips."
            } else {
                "However, folding here discards positive expected value since your draw \
                 exceeds the required equity."
            },
            if !should_call { "IS" } else { "is NOT" },
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Call".to_string(),
            is_correct: should_call,
            explanation: call_explanation,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Fold".to_string(),
            is_correct: !should_call,
            explanation: fold_explanation,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1, position: Position::BTN, stack: 200, is_hero: false, is_active: true,
        },
        PlayerState {
            seat: 2, position: hero_pos, stack: 200, is_hero: true, is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::PotOddsAndEquity,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T7 — Check-Raise Spot (CR-)
// ═══════════════════════════════════════════════════════════════════════════════

/// How much the flop board favours the BB caller vs the IP raiser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardFavour {
    /// Low/connected board (rank sum <= 20) — hits BB's wide range.
    BBFavorable,
    /// High/dry board — hits IP raiser's strong preflop range.
    IPFavorable,
}

/// How the hero hand interacts with the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandInteraction {
    /// Pairs a board card (strong made hand on the flop).
    Strong,
    /// Has a flush draw and/or straight draw.
    Draw,
    /// No pair, no draw.
    Weak,
}

fn classify_board(board: &[Card]) -> BoardFavour {
    let sum: u16 = board.iter().map(|c| c.rank.0 as u16).sum();
    if sum <= 20 {
        BoardFavour::BBFavorable
    } else {
        BoardFavour::IPFavorable
    }
}

fn classify_hand_interaction(hand: [Card; 2], board: &[Card]) -> HandInteraction {
    let flush = hero_has_flush_draw(hand, board);
    let straight = hero_has_straight_draw(hand, board);

    if flush || straight {
        return HandInteraction::Draw;
    }

    // Strong: hero pairs any board card
    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    let hits_board = hand.iter().any(|c| board_ranks.contains(&c.rank.0));
    if hits_board {
        HandInteraction::Strong
    } else {
        HandInteraction::Weak
    }
}

/// True when the hero hand has both a flush draw AND a straight draw component.
fn is_combo_draw(hand: [Card; 2], board: &[Card]) -> bool {
    hero_has_flush_draw(hand, board) && hero_has_straight_draw(hand, board)
}

pub fn generate_check_raise<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 3);

    let board_favour = classify_board(&board);
    let interaction = classify_hand_interaction(hero_hand, &board);
    let combo = is_combo_draw(hero_hand, &board);

    let branch_key = match (board_favour, interaction, combo) {
        (BoardFavour::BBFavorable, HandInteraction::Strong, _)   => "BBFav:Strong".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Draw, true)  => "BBFav:ComboDraw".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Draw, false) => "BBFav:Draw".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Weak, _)     => "BBFav:Weak".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Strong, _)   => "IPFav:Strong".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Draw, true)  => "IPFav:ComboDraw".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Draw, false) => "IPFav:Draw".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Weak, _)     => "IPFav:Weak".to_string(),
    };

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(50..=130), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    // Villain (IP) bets ~50-70% pot
    let villain_bet_pct: u32 = rng.gen_range(50..=70);
    let villain_bet = (pot * villain_bet_pct / 100).max(bb);
    let cr_size = villain_bet * 5 / 2; // 2.5x raise

    let hero_pos = Position::BB;
    let villain_pos = Position::BTN;

    let board_s = board_str(&board);
    let hand_s = hand_str(hero_hand);
    let board_favour_str = match board_favour {
        BoardFavour::BBFavorable => "BB-favorable (low/connected)",
        BoardFavour::IPFavorable => "IP-favorable (high/dry)",
    };
    let interaction_str = match (interaction, combo) {
        (HandInteraction::Draw, true)  => "combo draw",
        (HandInteraction::Draw, false) => "draw",
        (HandInteraction::Strong, _)   => "strong hand (pairs the board)",
        (HandInteraction::Weak, _)     => "weak/air",
    };

    // Correct answer (single ID):
    // BBFavorable + Strong -> "C" (Check-raise for value)
    // ComboDraw on any board -> "C" (Check-raise semi-bluff)
    // Weak + IPFavorable -> "A" (Fold)
    // Everything else -> "B" (Check-call)
    let correct: &str = match (board_favour, interaction) {
        (BoardFavour::BBFavorable, HandInteraction::Strong) => "C",
        (_, HandInteraction::Draw) if combo                 => "C",
        (BoardFavour::IPFavorable, HandInteraction::Weak)   => "A",
        _                                                   => "B",
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You're in the Big Blind (you act first). First three cards: {board_s}. \
             You have {hand_s}. The Button bet {villain_bet} chips. \
             Pot: {pot} chips. Stack: {stack} chips. What do you do?"
        ),
        TextStyle::Technical => format!(
            "You are in the Big Blind (OOP). Flop: {board_s} ({board_favour_str}). \
             You hold {hand_s} ({interaction_str}). \
             Villain on the Button bets {villain_bet} chips ({villain_bet_pct}% pot). \
             Pot is {pot} chips ({pot_bb} BB). Stack: {stack} chips ({stack_bb} BB). \
             What is your action?"
        ),
    };

    let fold_exp = match text_style {
        TextStyle::Simple => if matches!((board_favour, interaction),
            (BoardFavour::IPFavorable, HandInteraction::Weak)) {
            format!(
                "Correct — fold. You have nothing and the cards favour your opponent's hand. Putting more chips in would be throwing them away."
            )
        } else {
            format!(
                "Folding here is too cautious — you have enough of a hand to continue. Call or raise."
            )
        },
        TextStyle::Technical => if matches!((board_favour, interaction),
            (BoardFavour::IPFavorable, HandInteraction::Weak)) {
            format!(
                "Correct. With {interaction_str} on a {board_favour_str} board ({board_s}), \
                 you have no pair, no draw, and the board heavily favours villain's preflop range. \
                 Calling invests {villain_bet} chips with almost no equity. Fold."
            )
        } else {
            format!(
                "Folding {hand_s} ({interaction_str}) here is too tight. You have enough \
                 equity or positional leverage to continue, either by calling or check-raising. \
                 A fold surrenders too much to villain's {villain_bet_pct}% pot bet."
            )
        },
    };

    let call_exp = match text_style {
        TextStyle::Simple => if correct == "B" {
            format!(
                "Correct — call. You have enough of a hand to continue, but not quite enough to raise. Call {villain_bet} chips and see the next card."
            )
        } else if matches!((board_favour, interaction),
            (BoardFavour::BBFavorable, HandInteraction::Strong)) {
            format!(
                "Just calling here leaves money on the table. You have a strong hand on a board that favours you — raise to build the pot!"
            )
        } else {
            format!(
                "Just calling here is too passive. With a powerful draw, raise to put maximum pressure on your opponent."
            )
        },
        TextStyle::Technical => if correct == "B" {
            format!(
                "Correct. Check-calling with {hand_s} ({interaction_str}) on {board_s} \
                 is the best play. You have equity to continue but not the ideal conditions for \
                 a check-raise (either the board doesn't favour your range, or your draw alone \
                 doesn't justify building a large pot OOP). Call {villain_bet} and re-evaluate \
                 on the turn."
            )
        } else if matches!((board_favour, interaction),
            (BoardFavour::BBFavorable, HandInteraction::Strong)) {
            format!(
                "Check-calling with a strong hand on a BB-favorable board leaves value on the \
                 table. You should check-raise to {cr_size} chips to build the pot while you're \
                 ahead and deny villain's equity from backdoor draws and overcards."
            )
        } else {
            format!(
                "Check-calling is passive here. With {interaction_str} on {board_s}, a \
                 check-raise to {cr_size} chips extracts more value and applies pressure. \
                 Calling gives villain a free turn card to improve or bluff again cheaply."
            )
        },
    };

    let cr_exp = match text_style {
        TextStyle::Simple => match (board_favour, interaction, correct) {
            (BoardFavour::BBFavorable, HandInteraction::Strong, "C") => format!(
                "Correct — raise to {cr_size} chips! You have a strong hand and the cards are in your favour. Build the pot while you're ahead."
            ),
            (_, HandInteraction::Draw, "C") => format!(
                "Correct — raise to {cr_size} chips! You have a powerful draw with about a 54% chance of winning. Raising wins the pot immediately if your opponent folds, and builds a big pot when they call."
            ),
            (BoardFavour::IPFavorable, _, _) => format!(
                "Raising here is a bluff into your opponent's strong card range. They're unlikely to fold and you risk a lot of chips with a weak hand."
            ),
            _ => format!(
                "Raising without a very strong hand or a powerful draw is too aggressive here. Call instead."
            ),
        },
        TextStyle::Technical => match (board_favour, interaction, correct) {
            (BoardFavour::BBFavorable, HandInteraction::Strong, "C") => format!(
                "Correct. Check-raising to {cr_size} chips (2.5\u{00d7} villain's {villain_bet}) with \
                 {hand_s} ({interaction_str}) on a {board_favour_str} board ({board_s}) is \
                 the highest-EV play. This board hits your BB defending range (low/connected) \
                 much harder than villain's late-position range. You protect your hand, build \
                 the pot with the best of it, and deny villain cheap equity."
            ),
            (_, HandInteraction::Draw, "C") => format!(
                "Correct. Check-raising to {cr_size} chips (2.5\u{00d7} villain's {villain_bet}) as a \
                 combo-draw semi-bluff with {hand_s} on {board_s} is correct. Your combo \
                 draw has ~54% equity on the flop — you are a slight favourite! The check-raise \
                 wins the pot outright when villain folds, and builds a large pot when villain \
                 calls into your equity advantage."
            ),
            (BoardFavour::IPFavorable, _, _) => format!(
                "Check-raising on a {board_favour_str} board ({board_s}) with {hand_s} \
                 ({interaction_str}) is a bluff into villain's strongest range. This board \
                 connects heavily with late-position preflop hands; your check-raise has very \
                 low fold equity and risks getting 3-bet off a weak hand."
            ),
            _ => format!(
                "Check-raising with only a {interaction_str} (not a combo draw) may be \
                 too aggressive here. Without either a very strong made hand or a combo draw, \
                 the check-raise over-commits chips OOP without sufficient equity to back it up."
            ),
        },
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: fold_exp,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Call".to_string(),
            is_correct: correct == "B",
            explanation: call_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {} BB", cr_size / bb),
            is_correct: correct == "C",
            explanation: cr_exp,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1,
            position: hero_pos,
            stack,
            is_hero: true,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: villain_pos,
            stack,
            is_hero: false,
            is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: villain_bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::CheckRaiseSpot,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T8 — Semi-Bluff Decision (SB-)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn generate_semi_bluff<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 3);

    let draw_type = classify_draw(&board);

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (60u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(35..=120), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    // Villain bets 50-75% pot
    let villain_bet_pct: u32 = rng.gen_range(50..=75);
    let villain_bet = (pot * villain_bet_pct / 100).max(bb);
    let raise_size = villain_bet * 5 / 2; // 2.5x semi-bluff raise

    // Hero is randomly IP or OOP
    let hero_is_ip = rng.gen_bool(0.5);
    let hero_pos = if hero_is_ip { Position::BTN } else { Position::BB };
    let villain_pos = if hero_is_ip { Position::BB } else { Position::CO };

    let branch_key = match draw_type {
        DrawType::ComboDraw => "ComboDraw".to_string(),
        DrawType::FlushDraw => "FlushDraw".to_string(),
        DrawType::OESD      => format!("OESD:{}", if stack_bb >= 40 { "Deep" } else { "Short" }),
        DrawType::GutShot   => "GutShot".to_string(),
    };

    let board_s = board_str(&board);
    let hand_s = hand_str(hero_hand);
    let pos_str = format!("{}", hero_pos);
    let equity = draw_equity_flop(draw_type);
    let position_label = if hero_is_ip { "in position" } else { "out of position" };
    let position_label_simple = if hero_is_ip { "acting last (good position)" } else { "acting first (tough position)" };

    let draw_type_label = format!("{}", draw_type);
    let draw_type_simple_label = draw_simple_label(draw_type);

    // Correct answer (single ID):
    // ComboDraw         -> "C" (Raise — near-favourite, maximise pressure)
    // FlushDraw + IP    -> "B" (Call — realise equity in position)
    // OESD + stack >= 40 -> "C" (Raise — fold equity + semi-bluff)
    // GutShot           -> "A" (Fold — insufficient equity)
    // FlushDraw + OOP   -> "B" (Call — can't raise without positional advantage)
    let correct: &str = match draw_type {
        DrawType::ComboDraw                         => "C",
        DrawType::OESD if stack_bb >= 40            => "C",
        DrawType::FlushDraw | DrawType::OESD        => "B",
        DrawType::GutShot                           => "A",
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You have {hand_s} and are chasing a {draw_type_simple_label} after the first three cards: {board_s}. \
             You're {position_label_simple}. Your opponent bet {villain_bet} chips. \
             Pot: {pot} chips. Your draw wins roughly {:.0}% of the time. What do you do?",
            equity * 100.0
        ),
        TextStyle::Technical => format!(
            "You hold {hand_s} and have a {draw_type_label} on the flop {board_s}. \
             You are {position_label} ({pos_str}, {stack_bb} BB deep). \
             Villain bets {villain_bet} chips ({villain_bet_pct}% pot). \
             Pot is {pot} chips ({pot_bb} BB). \
             Your {draw_type_label} has ~{:.0}% equity. What do you do?",
            equity * 100.0
        ),
    };

    // --- Explanations ---

    let fold_exp = match text_style {
        TextStyle::Simple => if matches!(draw_type, DrawType::GutShot) {
            format!(
                "Correct — fold. An inside straight draw only wins about 17% of the time (roughly 1 in 6). The price to call is too high for those odds. Save your chips."
            )
        } else {
            format!(
                "Folding is a mistake — your {draw_type_simple_label} wins often enough to continue."
            )
        },
        TextStyle::Technical => if matches!(draw_type, DrawType::GutShot) {
            format!(
                "Correct. A gutshot (~17% equity) gives you roughly 4 outs. \
                 To call {villain_bet} chips into a {}-chip pot you need {:.1}% equity — \
                 your draw falls well short at 17%. Even with implied odds, a gutshot \
                 rarely justifies the call, and raising as a semi-bluff risks too many \
                 chips with insufficient raw equity.",
                pot + villain_bet,
                villain_bet as f32 / (pot + villain_bet) as f32 * 100.0
            )
        } else {
            format!(
                "Folding a {draw_type_label} (~{:.0}% equity) is too tight here. You have enough \
                 equity to continue — either by calling to realise it, or raising as a \
                 semi-bluff when conditions are right.",
                equity * 100.0
            )
        },
    };

    let call_exp = match text_style {
        TextStyle::Simple => match (draw_type, hero_is_ip, correct) {
            (DrawType::FlushDraw, true, "B") => format!(
                "Correct — call. You have a flush draw (~35% chance) and you're in good position (acting last). Call and see the next card — if you hit your flush you can bet big."
            ),
            (DrawType::FlushDraw, false, "B") | (DrawType::OESD, _, "B") => format!(
                "Correct — call. Your draw wins enough of the time to make calling worth it here. Just calling is safer than raising when you're acting first."
            ),
            _ => format!(
                "Calling is an option, but raising with your {draw_type_simple_label} puts more pressure on your opponent and wins the pot more often."
            ),
        },
        TextStyle::Technical => match (draw_type, hero_is_ip, correct) {
            (DrawType::FlushDraw, true, "B") => format!(
                "Correct. Calling with a {draw_type_label} (~{:.0}% equity) from {pos_str} (IP) is \
                 the best play. You have position to control the pot on future streets — check \
                 back or bet when you hit, give up cheaply when you miss. Raising risks bloating \
                 the pot without the positional advantage needed to navigate it well.",
                equity * 100.0
            ),
            (DrawType::FlushDraw, false, "B") | (DrawType::OESD, _, "B") => format!(
                "Correct. Calling with a {draw_type_label} (~{:.0}% equity) {position_label} is correct \
                 here. Your stack depth ({stack_bb} BB) and/or position make a semi-bluff raise \
                 suboptimal — calling lets you realise equity without bloating the pot OOP or \
                 risking a re-raise at shallow depth.",
                equity * 100.0
            ),
            _ => format!(
                "Calling is an option but not the highest-EV line here. With a {draw_type_label} \
                 (~{:.0}% equity) {position_label}, a semi-bluff raise to {raise_size} chips \
                 adds fold equity on top of your draw equity, making raising more profitable.",
                equity * 100.0
            ),
        },
    };

    let raise_exp = match text_style {
        TextStyle::Simple => match (draw_type, hero_is_ip, correct) {
            (DrawType::ComboDraw, _, "C") => format!(
                "Correct — raise to {raise_size} chips! Your two-way draw wins about 54% of the time — you're actually a slight favourite! Raising wins the pot right now if your opponent folds, or builds a big pot when you're favoured."
            ),
            (DrawType::OESD, _, "C") => format!(
                "Correct — raise to {raise_size} chips! A straight draw on both ends wins about 32% of the time, plus raising might make your opponent fold right now. The raise pays off whether they fold or call."
            ),
            _ => format!(
                "Raising here is too risky. Your draw doesn't win often enough to justify putting in so many chips. Just call."
            ),
        },
        TextStyle::Technical => match (draw_type, hero_is_ip, correct) {
            (DrawType::ComboDraw, _, "C") => format!(
                "Correct. Raising to {raise_size} chips (2.5\u{00d7} villain's {villain_bet}) with a \
                 {draw_type_label} on {board_s} is the highest-EV play. Your combo draw has ~54% \
                 equity — you are a slight favourite! Raising wins the pot outright when villain \
                 folds (~40% of the time) and builds a large pot when villain calls into your \
                 equity edge. Never just call with a combo draw when you can apply maximum pressure."
            ),
            (DrawType::OESD, _, "C") => format!(
                "Correct. Raising to {raise_size} chips (2.5\u{00d7} villain's {villain_bet}) with an \
                 {draw_type_label} at {stack_bb} BB depth is correct. Your OESD has ~32% equity plus \
                 significant fold equity: villain must fold hands like top pair to avoid getting \
                 stacked. At {stack_bb} BB the semi-bluff raise sets up a profitable shove on \
                 the turn or a clean check when you miss."
            ),
            _ => format!(
                "Raising to {raise_size} chips as a semi-bluff with a {draw_type_label} \
                 {position_label} is too aggressive here. You risk building a large pot \
                 without sufficient equity to back it up. Calling is the stronger line."
            ),
        },
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: fold_exp,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Call".to_string(),
            is_correct: correct == "B",
            explanation: call_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {} BB", raise_size / bb),
            is_correct: correct == "C",
            explanation: raise_exp,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1,
            position: villain_pos,
            stack,
            is_hero: false,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: hero_pos,
            stack,
            is_hero: true,
            is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: villain_bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::SemiBluffDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T13 — 3-Bet Pot C-Bet (3B-)
// ═══════════════════════════════════════════════════════════════════════════════

/// Flop texture in a 3-bet pot continuation-bet spot.
#[derive(Debug, Clone, Copy)]
enum FlopTexture {
    Dry, // Rainbow, uncoordinated — villain's calling range hits infrequently
    Wet, // Two-tone or connected — villain has many draws and top pairs
}

/// Hero's flop hand strength as the 3-better.
#[derive(Debug, Clone, Copy)]
enum FlopStrength {
    Strong, // Top pair+, overpair, set — clear c-bet
    Weak,   // Missed, underpair to board — consider giving up
}

impl std::fmt::Display for FlopTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlopTexture::Dry => write!(f, "dry (rainbow, uncoordinated)"),
            FlopTexture::Wet => write!(f, "wet (two-tone / connected)"),
        }
    }
}

impl std::fmt::Display for FlopStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlopStrength::Strong => write!(f, "strong (top pair+ / overpair / set)"),
            FlopStrength::Weak   => write!(f, "weak (missed / underpair to board)"),
        }
    }
}

pub fn generate_3bet_cbet<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 3);

    let texture  = if rng.gen_bool(0.5) { FlopTexture::Dry } else { FlopTexture::Wet };
    let fstrength = if rng.gen_bool(0.5) { FlopStrength::Strong } else { FlopStrength::Weak };

    let bb = 2u32;
    // 3-bet pots are bigger: pre-flop pot is typically 7-11 BB
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(10..=14u32), 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(8..=18),     rng.gen_range(50..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(6..=22),     rng.gen_range(30..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    // SPR is low in 3-bet pots — stacks commit quickly
    let spr = stack as f32 / pot as f32;

    let small_bet = (pot as f32 * 0.33).round() as u32;
    let large_bet = (pot as f32 * 0.67).round() as u32;

    // Correct action:
    // Dry + Strong -> small c-bet (~33%): dry boards miss villain's range; a small probe
    //               is enough to extract value and start building toward a commitment
    //               given the low SPR.
    // Wet + Strong -> large c-bet (~67%): charge draws, deny equity, commit the stack.
    // Any + Weak   -> check: no equity; in a low-SPR pot any bet is a large commitment
    //               that is hard to fold to a raise.
    let (correct, branch_key): (&str, &str) = match (texture, fstrength) {
        (FlopTexture::Dry, FlopStrength::Strong) => ("B", "Dry:Strong:SmallCbet"),
        (FlopTexture::Wet, FlopStrength::Strong) => ("C", "Wet:Strong:LargeCbet"),
        (FlopTexture::Dry, FlopStrength::Weak)   => ("A", "Dry:Weak:Check"),
        (FlopTexture::Wet, FlopStrength::Weak)   => ("A", "Wet:Weak:Check"),
    };
    let branch_key = branch_key.to_string();

    let hero_pos  = Position::BTN;
    let hand_s  = hand_str(hero_hand);
    let board_s = board_str(&board);

    let question = match text_style {
        TextStyle::Simple => format!(
            "You re-raised before the flop and your opponent called. First three cards: {board_s}. \
             You have {hand_s} on the Button. Pot: {pot} chips. Stack: {stack} chips. \
             Your opponent checked to you. What do you do?"
        ),
        TextStyle::Technical => format!(
            "3-bet pot c-bet. You hold {hand_s} ({fstrength}) on the Button (the 3-better). \
             Villain called your 3-bet from the Big Blind. Board: {board_s} ({texture}). \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips (SPR ~{spr:.1}). \
             Villain checks to you. What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check back".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match (texture, fstrength) {
                    (_, FlopStrength::Weak) => format!(
                        "Correct — check. Your hand is weak here. No need to bet — see the next card for free."
                    ),
                    _ => format!(
                        "Checking here gives up on a hand worth betting. Take the initiative."
                    ),
                },
                TextStyle::Technical => match (texture, fstrength) {
                    (_, FlopStrength::Weak) => format!(
                        "Correct. Checking back a {fstrength} on a {texture} board is right in \
                         this 3-bet pot. With SPR ~{spr:.1}, any c-bet represents a meaningful \
                         commitment — if villain check-raises, folding is costly and calling risks \
                         stacking off with poor equity. Check to keep the pot small and preserve \
                         the option to bluff a favourable turn card."
                    ),
                    _ => format!(
                        "Checking a {fstrength} in this 3-bet pot surrenders value. The low SPR \
                         ({spr:.1}) means bets are decisive — extract now while you have the \
                         equity lead. Villain's BB calling range is wide and misses the board \
                         frequently. C-bet to build the pot you are likely to win."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("C-bet small ({small_bet} chips ~33%)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match (texture, fstrength) {
                    (FlopTexture::Dry, FlopStrength::Strong) => format!(
                        "Correct — bet small. The board is dry (no likely draws). A small bet is enough to collect chips and keep pressure on."
                    ),
                    (FlopTexture::Wet, FlopStrength::Strong) => format!(
                        "A small bet isn't enough on this type of board. Bet bigger or check."
                    ),
                    _ => format!(
                        "A small bet isn't enough on this type of board. Bet bigger or check."
                    ),
                },
                TextStyle::Technical => match (texture, fstrength) {
                    (FlopTexture::Dry, FlopStrength::Strong) => format!(
                        "Correct. A small c-bet (~33% pot) with a {fstrength} on a {texture} board \
                         is optimal. Dry boards miss villain's wide BB calling range frequently, \
                         so a small probe achieves two goals: it extracts value from any pair or \
                         draw while — given SPR ~{spr:.1} — starting to build toward a natural \
                         stack commitment. Villain must act immediately with limited backdoor outs."
                    ),
                    (FlopTexture::Wet, FlopStrength::Strong) => format!(
                        "A small c-bet on a {texture} board undersizes the protection needed. \
                         Villain has many draws (flush draws, straight draws) that can call 33% \
                         cheaply and realise equity. A larger bet (~67%) forces them to pay the \
                         correct price and extracts more from made hands that call."
                    ),
                    _ => format!(
                        "C-betting 33% with a {fstrength} is still a significant commitment at \
                         SPR ~{spr:.1}. Any bet is hard to walk back in a 3-bet pot. Check back \
                         and reassess on the turn."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("C-bet large ({large_bet} chips ~67%)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match (texture, fstrength) {
                    (FlopTexture::Wet, FlopStrength::Strong) => format!(
                        "Correct — bet big! The board has possible draws. Make your opponent pay dearly to chase them."
                    ),
                    (FlopTexture::Dry, FlopStrength::Strong) => format!(
                        "Betting big here is too much — a check or small bet fits this situation better."
                    ),
                    _ => format!(
                        "Betting big here is too much — a check or small bet fits this situation better."
                    ),
                },
                TextStyle::Technical => match (texture, fstrength) {
                    (FlopTexture::Wet, FlopStrength::Strong) => format!(
                        "Correct. A large c-bet (~67% pot) with a {fstrength} on a {texture} board \
                         is the highest-EV line. Wet boards give villain flush draws, straight draws, \
                         and top pairs. Betting large charges every draw immediately, denies cheap \
                         equity, and naturally commits the remaining stack at SPR ~{spr:.1}."
                    ),
                    (FlopTexture::Dry, FlopStrength::Strong) => format!(
                        "A large c-bet on a {texture} board slightly over-bets the situation. \
                         Dry boards rarely hit villain's calling range — a smaller bet (33%) \
                         achieves the same fold equity while sizing more accurately to the \
                         low-draw texture. Save larger sizings for boards with more draws."
                    ),
                    _ => format!(
                        "C-betting 67% with a {fstrength} at SPR ~{spr:.1} puts a large portion \
                         of your stack in with poor equity. If called or raised, you have little \
                         fold equity and a tough decision. Check back instead."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: Position::BB, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,     stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::ThreeBetPotCbet,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board,
            players,
            pot_size:      pot,
            current_bet:   0,
        },
        question,
        answers,
    }
}
