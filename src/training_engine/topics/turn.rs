//! Turn topic generators: barrel, probe bet, and delayed c-bet.
//!
//! All three topics deal a 4-card board (flop + turn) and ask hero what to do
//! on the turn.  The key analysis in each:
//!
//! - **T6 Turn Barrel** — Classifies the turn card (Blank / ScareBroadway /
//!   DrawComplete) and flop texture to decide whether to double-barrel IP.
//! - **T15 Turn Probe** — Hero is OOP (BB) after the flop checks through.
//!   Hand strength (Strong / Medium / Weak) determines probe sizing.
//! - **T16 Delayed C-Bet** — Hero is IP (BTN) after checking back the flop.
//!   Combines hand strength with turn-card type (Blank / Scare) to decide
//!   whether to fire a delayed c-bet and at what size.

use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{board_texture, BoardTexture},
    helpers::{hand_str, board_str, heads_up, scenario},
    models::*,
};

// ═══════════════════════════════════════════════════════════════════════════════
// T6 — Turn Barrel Decision (TB-)
//
// Hero c-bet the flop and villain called.  Now the turn is dealt.  Should hero
// fire a second barrel?  The answer depends on the turn card type:
//   - DrawComplete  → check (villain's range improved)
//   - ScareBroadway → bet large (credible threat from IP range)
//   - Blank + Wet   → bet medium (charge draws)
//   - Blank + Dry   → check (no reason to barrel without value)
// ═══════════════════════════════════════════════════════════════════════════════

/// How the turn card changes the board relative to the flop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BarrelTurnCard {
    /// Low card, doesn't complete draws or add scare cards.
    Blank,
    /// Broadway card (T+) that favours the IP opener's range.
    ScareBroadway,
    /// Card that completes a likely flush or straight draw.
    DrawComplete,
}

fn classify_barrel_turn(flop: &[Card], turn: &Card) -> BarrelTurnCard {
    let turn_suit_count = flop.iter().filter(|c| c.suit == turn.suit).count();
    if turn_suit_count >= 2 {
        return BarrelTurnCard::DrawComplete;
    }
    let mut ranks: Vec<u8> = flop.iter().map(|c| c.rank.0).collect();
    ranks.push(turn.rank.0);
    ranks.sort_unstable();
    ranks.dedup();
    for window in ranks.windows(4) {
        if window[3] - window[0] <= 4 {
            return BarrelTurnCard::DrawComplete;
        }
    }
    if turn.rank.0 >= 10 {
        return BarrelTurnCard::ScareBroadway;
    }
    BarrelTurnCard::Blank
}

pub fn generate_barrel<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let flop: Vec<Card> = deck.deal_n(3);
    let turn = deck.deal();

    let texture = board_texture(&flop);
    let turn_type = classify_barrel_turn(&flop, &turn);

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(14..=22)),
        DifficultyLevel::Intermediate => (rng.gen_range(50..=130), rng.gen_range(10..=28)),
        DifficultyLevel::Advanced     => (rng.gen_range(25..=200), rng.gen_range(8..=40)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    let hero_pos = if rng.gen_bool(0.5) { Position::BTN } else { Position::CO };

    let players = vec![
        PlayerState { seat: 1, position: Position::BB, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos, stack, is_hero: true, is_active: true },
    ];

    let branch_key = match turn_type {
        BarrelTurnCard::DrawComplete  => "DrawComplete".to_string(),
        BarrelTurnCard::ScareBroadway => "ScareBroadway".to_string(),
        BarrelTurnCard::Blank => match texture {
            BoardTexture::Wet | BoardTexture::SemiWet => "Blank:Wet".to_string(),
            BoardTexture::Dry                         => "Blank:Dry".to_string(),
        },
    };

    let flop_str = board_str(&flop);
    let hs = hand_str(hero_hand);
    let pos_str = format!("{}", hero_pos);
    let texture_str = format!("{}", texture);
    let turn_str = turn.to_string();

    let turn_label = match turn_type {
        BarrelTurnCard::Blank         => "blank",
        BarrelTurnCard::ScareBroadway => "scare Broadway card",
        BarrelTurnCard::DrawComplete  => "draw-completing card",
    };
    let turn_label_simple = match turn_type {
        BarrelTurnCard::Blank         => "blank (doesn't help either player much)",
        BarrelTurnCard::ScareBroadway => "big card (J, Q, K, or A)",
        BarrelTurnCard::DrawComplete  => "possible draw-completing card",
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You bet after the first three cards and your opponent called. You have {hs} in {pos_str}. \
             First three cards: {flop_str}. Fourth card: {turn_str} (a {turn_label_simple}). \
             Pot: {pot} chips. Stack: {stack} chips. \
             Your opponent checks to you. Options: check, bet medium (~{} chips), bet big (~{} chips). What do you do?",
            pot / 2, pot * 4 / 5
        ),
        TextStyle::Technical => format!(
            "You c-bet the flop and villain called. You hold {hs} from {pos_str}. \
             Flop: {flop_str} ({texture_str}). Turn: {turn_str} (a {turn_label}). \
             Pot is {pot} chips ({pot_bb} BB), stack {stack} chips ({stack_bb} BB). \
             Villain checks to you. Bet options: medium (~50% pot = {} chips) or \
             large (~80% pot = {} chips). What do you do?",
            pot / 2, pot * 4 / 5
        ),
    };

    let correct: &str = match turn_type {
        BarrelTurnCard::DrawComplete  => "A",
        BarrelTurnCard::ScareBroadway => "C",
        BarrelTurnCard::Blank => {
            if matches!(texture, BoardTexture::Wet | BoardTexture::SemiWet) { "B" } else { "A" }
        }
    };

    let check_exp = match text_style {
        TextStyle::Simple => match turn_type {
            BarrelTurnCard::DrawComplete => "Correct — check. The new card may have completed your opponent's draw. Betting here is risky — take a free look at the next card.".to_string(),
            BarrelTurnCard::ScareBroadway => "Checking here lets your opponent off the hook. The big card actually helps your story — bet to take the pot.".to_string(),
            BarrelTurnCard::Blank => {
                if correct == "A" {
                    "Correct — check. The new card doesn't change much on a dry board. No need to bet without a strong hand.".to_string()
                } else {
                    "Checking gives your opponent a free card when draws are still possible. Bet to make them pay.".to_string()
                }
            }
        },
        TextStyle::Technical => match turn_type {
            BarrelTurnCard::DrawComplete => format!(
                "Correct. The {turn_str} completes potential draws — villain's check-calling range \
                 is now stronger and your bluff equity has collapsed. Checking back controls the pot \
                 and takes a free showdown or river spot."
            ),
            BarrelTurnCard::ScareBroadway => format!(
                "The {turn_str} is a scare card that actually hits your late-position preflop \
                 range harder than villain's calling range. Checking surrenders fold equity when \
                 barrelling is profitable."
            ),
            BarrelTurnCard::Blank => {
                if correct == "A" {
                    format!(
                        "Correct. On a {texture_str} dry board a blank turn ({turn_str}) gives you \
                         no reason to barrel without a value hand or clear draw. Checking back to \
                         control pot size is the strongest play."
                    )
                } else {
                    format!(
                        "Checking on a {texture_str} board with draws still live gives villain a \
                         free card. You should charge draws with a medium-sized bet."
                    )
                }
            }
        },
    };

    let bet50_exp = match text_style {
        TextStyle::Simple => match turn_type {
            BarrelTurnCard::DrawComplete => "Betting into a possible completed draw is risky — your opponent may now have a better hand than you. Check.".to_string(),
            BarrelTurnCard::ScareBroadway => "A medium bet works but a bigger bet puts more pressure on your opponent when the big card hits.".to_string(),
            BarrelTurnCard::Blank => {
                if correct == "B" {
                    "Correct — bet medium. Draws are still possible and a medium bet makes it expensive for your opponent to chase them.".to_string()
                } else {
                    "Betting medium on a dry board without a strong hand wastes chips. Check instead.".to_string()
                }
            }
        },
        TextStyle::Technical => match turn_type {
            BarrelTurnCard::DrawComplete => format!(
                "Barrelling into a completed draw is a leak. The {turn_str} strengthens villain's \
                 check-calling range; a bet risks getting check-raised or called by made hands \
                 that now beat you."
            ),
            BarrelTurnCard::ScareBroadway => format!(
                "A 50% pot bet is an option but undersizes the scare-card advantage. When a \
                 Broadway card ({turn_str}) hits, your polarised range can support a larger barrel \
                 to maximise fold equity from villain's medium-strength hands."
            ),
            BarrelTurnCard::Blank => {
                if correct == "B" {
                    format!(
                        "Correct. A ~50% pot barrel on a {texture_str} board gives villain \
                         incorrect pot odds to continue with flush draws (~20% equity on the turn). \
                         It charges draws without over-committing."
                    )
                } else {
                    format!(
                        "Betting 50% pot on a {texture_str} dry board without a value hand or draw \
                         is a marginal bluff with little fold equity. Checking back is higher EV."
                    )
                }
            }
        },
    };

    let bet80_exp = match text_style {
        TextStyle::Simple => match turn_type {
            BarrelTurnCard::DrawComplete => "Betting big into a possible completed draw is a big mistake — you could be betting into a made hand.".to_string(),
            BarrelTurnCard::ScareBroadway => "Correct — bet big! The big card (J/Q/K/A) looks scary to your opponent and suggests you have a strong hand. A big bet here forces tough decisions.".to_string(),
            BarrelTurnCard::Blank => "Betting big without a good reason on this board is too aggressive. Check or bet medium.".to_string(),
        },
        TextStyle::Technical => match turn_type {
            BarrelTurnCard::DrawComplete => format!(
                "A large barrel into a completed draw board is a significant mistake. Villain's \
                 check-calling range is polarised toward made hands after the {turn_str}; an \
                 80% pot bet as a bluff has very low fold equity and costs you a lot when called."
            ),
            BarrelTurnCard::ScareBroadway => format!(
                "Correct. An ~80% pot barrel on the {turn_str} leverages the scare card to \
                 maximise fold equity. Your range (opening from {pos_str}) is heavily weighted \
                 toward Broadway cards, making this bet highly credible and difficult for \
                 villain's medium pairs and draws to continue against."
            ),
            BarrelTurnCard::Blank => format!(
                "An 80% pot bet on a blank turn without a strong hand or draw over-commits \
                 resources. Size down or check back — large barrels on {texture_str} boards \
                 without the nuts can become difficult to follow through on the river."
            ),
        },
    };

    let answers = vec![
        AnswerOption { id: "A".to_string(), text: "Check".to_string(), is_correct: correct == "A", explanation: check_exp },
        AnswerOption { id: "B".to_string(), text: "Bet medium".to_string(), is_correct: correct == "B", explanation: bet50_exp },
        AnswerOption { id: "C".to_string(), text: "Bet large".to_string(), is_correct: correct == "C", explanation: bet80_exp },
    ];

    let mut board = flop;
    board.push(turn);

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: 0,
    };

    TrainingScenario { scenario_id, topic: TrainingTopic::TurnBarrelDecision, branch_key, table_setup, question, answers }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T15 — Turn Probe Bet (PB-)
//
// Hero is in the Big Blind (OOP).  Both players checked the flop — villain's
// range is capped (no sets, no strong top pairs).  On the turn hero can
// "probe" — bet to exploit villain's capped range:
//   - Strong hand → large probe (~70% pot) for max value
//   - Medium hand → small probe (~40% pot) for thin value
//   - Weak hand   → check (no equity to justify a bluff)
// ═══════════════════════════════════════════════════════════════════════════════

/// Hero's hand strength for the probe-bet decision.
#[derive(Debug, Clone, Copy)]
enum ProbeStrength {
    Strong,
    Medium,
    Weak,
}

impl std::fmt::Display for ProbeStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeStrength::Strong => write!(f, "strong (top pair+ / strong draw)"),
            ProbeStrength::Medium => write!(f, "medium (middle pair / weak draw)"),
            ProbeStrength::Weak   => write!(f, "weak (bottom pair / air)"),
        }
    }
}

fn probe_strength_simple(ps: ProbeStrength) -> &'static str {
    match ps {
        ProbeStrength::Strong => "strong hand",
        ProbeStrength::Medium => "medium hand",
        ProbeStrength::Weak   => "weak hand",
    }
}

pub fn generate_probe<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(4);

    let strength = match rng.gen_range(0..3) {
        0 => ProbeStrength::Strong,
        1 => ProbeStrength::Medium,
        _ => ProbeStrength::Weak,
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(6..=14u32), 80u32),
        DifficultyLevel::Intermediate => (rng.gen_range(4..=20),    rng.gen_range(40..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(4..=30),    rng.gen_range(20..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let small_probe = (pot as f32 * 0.40).round() as u32;
    let large_probe = (pot as f32 * 0.70).round() as u32;

    let correct: &str = match strength {
        ProbeStrength::Strong => "C",
        ProbeStrength::Medium => "B",
        ProbeStrength::Weak   => "A",
    };

    let branch_key = match strength {
        ProbeStrength::Strong => "Strong:ProbeLarge",
        ProbeStrength::Medium => "Medium:ProbeSmall",
        ProbeStrength::Weak   => "Weak:Check",
    };

    let hero_pos = Position::BB;
    let hs = hand_str(hero_hand);
    let bs = board_str(&board);
    let strength_simple = probe_strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Both players checked after the first three cards. Fourth card: {bs}. \
             You have {hs} ({strength_simple}) in the Big Blind (you act first). \
             Pot: {pot} chips. Stack: {stack} chips. \
             Options: check, bet small ({small_probe} chips), bet big ({large_probe} chips). What do you do?"
        ),
        TextStyle::Technical => format!(
            "Turn probe spot. You hold {hs} ({strength}) in the Big Blind (OOP). \
             The flop was checked through by both players. Board (flop + turn): {bs}. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             You are first to act on the turn. \
             Probe options: small ({small_probe} chips ~40%), large ({large_probe} chips ~70%). \
             What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    ProbeStrength::Weak => "Correct — check. Your hand is weak and your opponent didn't bet on the flop — no reason to bet now.".to_string(),
                    _ => "Checking here misses an opportunity. Your hand is strong enough to bet and take the pot.".to_string(),
                },
                TextStyle::Technical => match strength {
                    ProbeStrength::Weak => format!(
                        "Correct. Checking a {strength} in this OOP probe spot is right. \
                         You have no equity justification to bet — a probe would be a pure bluff \
                         into a player who checked back the flop (a capped but still medium-strong \
                         range). Check, and consider check-folding if villain bets."
                    ),
                    _ => format!(
                        "Checking a {strength} when you can take the initiative is too passive. \
                         Villain checked back the flop — their range is capped (no sets, no \
                         strong top pairs). Use that information and probe to build the pot or \
                         apply pressure."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Probe small ({small_probe} chips ~40%)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    ProbeStrength::Medium => "Correct — bet small. Your hand is decent but not great. A small bet tests the water and may win the pot without risking too much.".to_string(),
                    ProbeStrength::Strong => "A small bet doesn't do enough here — bet bigger to put real pressure on, or just check.".to_string(),
                    ProbeStrength::Weak   => "A small bet doesn't do enough here — bet bigger to put real pressure on, or just check.".to_string(),
                },
                TextStyle::Technical => match strength {
                    ProbeStrength::Medium => format!(
                        "Correct. A small probe (~40% pot) with a {strength} is the best line. \
                         Your hand has some equity (a pair or draw) and a small bet applies \
                         pressure without over-committing. If called, you have reasonable pot \
                         odds for a river bet or free showdown. If raised, you can fold \
                         without a catastrophic loss."
                    ),
                    ProbeStrength::Strong => format!(
                        "A small probe with a {strength} undersizes the value available. Villain's \
                         flop check-back range includes top pairs and floats — a larger probe \
                         (~70%) extracts more from one-pair hands and charges any draws more \
                         effectively."
                    ),
                    ProbeStrength::Weak => format!(
                        "Probing small with a {strength} is a bluff with poor equity. Villain may \
                         have floated the flop with a medium hand and will call or raise. \
                         Without equity to fall back on, this bet risks chips without justification."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Probe large ({large_probe} chips ~70%)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    ProbeStrength::Strong => "Correct — bet big! You have a strong hand and the Button didn't bet after the flop (a sign of weakness). Take the pot now with a big bet.".to_string(),
                    ProbeStrength::Medium => "Betting big here is too aggressive for your hand strength. Bet small or check.".to_string(),
                    ProbeStrength::Weak   => "Betting big here is too aggressive for your hand strength. Bet small or check.".to_string(),
                },
                TextStyle::Technical => match strength {
                    ProbeStrength::Strong => format!(
                        "Correct. A large probe (~70% pot) with a {strength} is the highest-EV \
                         play. Villain's check-back range is capped and contains many one-pair \
                         and draw hands. A larger bet extracts maximum value, charges draws \
                         effectively, and builds a significant pot worth fighting for on the river."
                    ),
                    ProbeStrength::Medium => format!(
                        "A large probe with a {strength} over-commits to a hand with moderate \
                         equity. If raised, you face a tough spot with a middle pair or weak \
                         draw. A smaller probe (~40%) achieves semi-bluff value at lower risk."
                    ),
                    ProbeStrength::Weak => format!(
                        "Probing large with a {strength} is a high-risk bluff with minimal equity. \
                         Villain's check-back range often contains medium-strength hands that call \
                         or raise large bets. Check to preserve your stack."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: Position::BTN, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,      stack, is_hero: true,  is_active: true },
    ];

    scenario(scenario_id, TrainingTopic::TurnProbeBet, branch_key,
        GameType::CashGame, hero_pos, hero_hand, board, players, pot, 0, question, answers)
}

// ═══════════════════════════════════════════════════════════════════════════════
// T16 — Delayed C-Bet (DC-)
//
// Hero opened from BTN, BB called, and hero checked back the flop (no c-bet).
// On the turn hero can fire a "delayed c-bet".  Decision matrix:
//   - Strong hand (any turn)       → medium c-bet (~60%)
//   - Medium hand + Blank turn     → small c-bet (~33%)
//   - Medium hand + Scare turn     → check (pot control)
//   - Weak hand   (any turn)       → check (no equity to bluff)
//
// `TurnStrength` and `TurnCard` are `pub(crate)` so tests can verify the
// classifiers directly.
// ═══════════════════════════════════════════════════════════════════════════════

/// Hero's hand strength on a 4-card board, visible to tests via `pub(crate)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnStrength {
    Strong,
    Medium,
    Weak,
}

impl std::fmt::Display for TurnStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TurnStrength::Strong => write!(f, "strong (overpair / top pair good kicker / two pair / set)"),
            TurnStrength::Medium => write!(f, "medium (middle pair / weak top pair / underpair)"),
            TurnStrength::Weak   => write!(f, "weak (missed / low pair / air)"),
        }
    }
}

fn delayed_strength_simple(s: TurnStrength) -> &'static str {
    match s {
        TurnStrength::Strong => "strong hand",
        TurnStrength::Medium => "medium hand",
        TurnStrength::Weak   => "weak hand",
    }
}

pub(crate) fn classify_turn_strength(hero: [Card; 2], board: &[Card]) -> TurnStrength {
    let h0 = hero[0].rank.0;
    let h1 = hero[1].rank.0;
    let high = h0.max(h1);

    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    let board_max = board_ranks.iter().copied().max().unwrap_or(0);

    let pocket_pair = h0 == h1;
    let matches_h0 = board_ranks.iter().filter(|&&r| r == h0).count();
    let matches_h1 = board_ranks.iter().filter(|&&r| r == h1).count();

    if pocket_pair {
        if matches_h0 >= 1 { return TurnStrength::Strong; }
        if high > board_max { return TurnStrength::Strong; }
        return TurnStrength::Medium;
    }

    let paired_both = matches_h0 >= 1 && matches_h1 >= 1;
    let paired_any  = matches_h0 >= 1 || matches_h1 >= 1;

    if paired_both { return TurnStrength::Strong; }

    if paired_any {
        let paired_rank = if matches_h0 >= 1 { h0 } else { h1 };
        if paired_rank == board_max {
            let kicker = if paired_rank == h0 { h1 } else { h0 };
            if kicker >= 11 { return TurnStrength::Strong; }
            return TurnStrength::Medium;
        }
        return TurnStrength::Medium;
    }

    TurnStrength::Weak
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnCard {
    Blank,
    Scare,
}

pub(crate) fn classify_turn_card(flop: &[Card], turn: &Card) -> TurnCard {
    let flop_max = flop.iter().map(|c| c.rank.0).max().unwrap_or(0);
    if turn.rank.0 > flop_max { return TurnCard::Scare; }

    let turn_suit_count = flop.iter().filter(|c| c.suit == turn.suit).count();
    if turn_suit_count >= 2 { return TurnCard::Scare; }

    let mut all_ranks: Vec<u8> = flop.iter().map(|c| c.rank.0).collect();
    all_ranks.push(turn.rank.0);
    all_ranks.sort_unstable();
    all_ranks.dedup();
    if all_ranks.len() >= 4 {
        for w in all_ranks.windows(4) {
            if w[3] - w[0] <= 4 { return TurnCard::Scare; }
        }
    }

    TurnCard::Blank
}

pub fn generate_delayed_cbet<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(4);

    let flop = &board[..3];
    let turn = &board[3];

    let strength = classify_turn_strength(hero_hand, &board);
    let turn_type = classify_turn_card(flop, turn);

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(6..=14u32), 80u32),
        DifficultyLevel::Intermediate => (rng.gen_range(4..=20),    rng.gen_range(40..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(4..=30),    rng.gen_range(20..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let small_cbet = (pot as f32 * 0.33).round() as u32;
    let medium_cbet = (pot as f32 * 0.60).round() as u32;

    let correct: &str = match (strength, turn_type) {
        (TurnStrength::Strong, _)                   => "C",
        (TurnStrength::Medium, TurnCard::Blank)     => "B",
        (TurnStrength::Medium, TurnCard::Scare)     => "A",
        (TurnStrength::Weak, _)                     => "A",
    };

    let turn_label = match turn_type {
        TurnCard::Blank => "Blank",
        TurnCard::Scare => "Scare",
    };
    let strength_label = match strength {
        TurnStrength::Strong => "Strong",
        TurnStrength::Medium => "Medium",
        TurnStrength::Weak   => "Weak",
    };
    let branch_key = format!("{strength_label}:{turn_label}");

    let hero_pos = Position::BTN;
    let hs = hand_str(hero_hand);
    let bs = board_str(&board);
    let texture = board_texture(&board);
    let strength_simple = delayed_strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "You raised before the flop from the Button. The Big Blind called. \
             On the flop you checked behind. Now on the turn the board is: {bs}. \
             You have {hs} ({strength_simple}). \
             Pot: {pot} chips. Stack: {stack} chips. \
             Villain checks to you. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Delayed c-bet spot. You opened BTN, BB called. You checked back the flop \
             (no c-bet). Board: {bs} ({texture}). \
             You hold {hs} ({strength}). \
             Turn card is a {turn_label}. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             Villain checks. Delayed c-bet options: small ({small_cbet} ~33%), \
             medium ({medium_cbet} ~60%), or check. What is your play?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match (strength, turn_type) {
                    (TurnStrength::Weak, _) =>
                        "Correct — check. Your hand missed the board. Betting here with nothing \
                         risks chips for no reason. Check and see a free river.".to_string(),
                    (TurnStrength::Medium, TurnCard::Scare) =>
                        "Correct — check for pot control. The turn card changed the board and \
                         your medium hand may no longer be best. Keep the pot small.".to_string(),
                    (TurnStrength::Medium, TurnCard::Blank) =>
                        "Checking here is too passive. You have a decent hand on a quiet turn card — \
                         a small bet would get value and protect against draws.".to_string(),
                    (TurnStrength::Strong, _) =>
                        "Checking here wastes your strong hand. You skipped the flop — \
                         now is the time to bet and build the pot.".to_string(),
                },
                TextStyle::Technical => match (strength, turn_type) {
                    (TurnStrength::Weak, _) => format!(
                        "Correct. With a {strength} you have no equity to justify a delayed c-bet. \
                         Villain's flop check/check line doesn't cap their range enough to bluff \
                         profitably. Check and realise any equity you have."
                    ),
                    (TurnStrength::Medium, TurnCard::Scare) => format!(
                        "Correct. The scare turn card (overcard / draw completion) devalues your \
                         {strength}. A delayed c-bet here bloats the pot when villain's continuing \
                         range has improved. Check for pot control and reassess on the river."
                    ),
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Checking a {strength} on a blank turn is too passive. The blank didn't \
                         change the board — a small delayed c-bet (~33%) extracts thin value \
                         and denies equity to overcards."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "Checking a {strength} on the turn after already checking the flop forfeits \
                         too much value. You must fire a delayed c-bet to build the pot — villain's \
                         range includes many hands that will call a medium sizing."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Small delayed c-bet ({small_cbet} chips ~33%)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match (strength, turn_type) {
                    (TurnStrength::Medium, TurnCard::Blank) =>
                        "Correct — bet small. You have a decent hand on a quiet board. A small bet \
                         gets value from worse hands and makes draws pay a bit, without risking too much.".to_string(),
                    (TurnStrength::Strong, _) =>
                        "A small bet is too timid with a strong hand. Bet bigger to build the pot \
                         and charge draws properly.".to_string(),
                    _ =>
                        "A small bet here doesn't accomplish much. With your hand, either check back \
                         or bet bigger.".to_string(),
                },
                TextStyle::Technical => match (strength, turn_type) {
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Correct. A small delayed c-bet (~33% pot) with a {strength} on a blank \
                         turn is optimal. You extract thin value, deny equity to overcards and \
                         gutshots, and keep the pot manageable if raised."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "A 33% sizing with a {strength} leaves too much value on the table. \
                         Villain's range includes one-pair and draw hands that will call ~60%. \
                         Size up to maximise EV."
                    ),
                    (TurnStrength::Medium, TurnCard::Scare) => format!(
                        "Betting small on a scare turn with a {strength} risks getting raised off \
                         the best hand. The scare card improves villain's continuing range — \
                         pot control via check is preferred."
                    ),
                    (TurnStrength::Weak, _) => format!(
                        "A small delayed c-bet with a {strength} is a bluff with poor equity. \
                         Villain's calling range on this runout beats you. Save chips and check."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Medium delayed c-bet ({medium_cbet} chips ~60%)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    TurnStrength::Strong =>
                        "Correct — bet medium! You have a strong hand and you already checked \
                         the flop. Time to get value. A ~60% pot bet puts pressure on weaker \
                         hands and makes draws pay.".to_string(),
                    TurnStrength::Medium =>
                        "A medium bet is too big for your hand strength. You risk too many chips \
                         when you might not have the best hand.".to_string(),
                    TurnStrength::Weak =>
                        "Betting big with a weak hand is reckless. Check and save your chips.".to_string(),
                },
                TextStyle::Technical => match strength {
                    TurnStrength::Strong => format!(
                        "Correct. A medium delayed c-bet (~60% pot) with a {strength} is highest-EV. \
                         After checking the flop your range is perceived as weak, so villain will \
                         call more liberally. This sizing extracts max value from one-pair and draw \
                         hands while setting up a river shove."
                    ),
                    TurnStrength::Medium => format!(
                        "A 60% pot delayed c-bet over-commits a {strength}. If raised, you're in \
                         a tough spot with a marginal hand. A smaller sizing (~33%) achieves the \
                         same protection at lower risk, or check on a scare card."
                    ),
                    TurnStrength::Weak => format!(
                        "A large delayed c-bet with a {strength} is a high-risk bluff. \
                         Villain's range after calling preflop and seeing a check-through \
                         includes many sticky hands. Check to preserve your stack."
                    ),
                },
            },
        },
    ];

    let players = heads_up(hero_pos, Position::BB, stack, stack);
    scenario(scenario_id, TrainingTopic::DelayedCbet, branch_key,
        GameType::CashGame, hero_pos, hero_hand, board, players, pot, 0, question, answers)
}
