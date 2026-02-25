//! Preflop topic generators: open-raise, ICM push/fold, anti-limper isolation,
//! squeeze play, and big blind defense.
//!
//! All five topics have an empty board (0 community cards).  Hand strength is
//! classified using `evaluator::classify_hand()` (5-category system).
//!
//! ## Topics in this file
//!
//! - **T1 Preflop Decision** (`generate` / `generate_open`) — Open-raise, call,
//!   or fold based on hand category, position (6-max or 9-max), and spot type
//!   (OpenRaise / FacingOpen / ThreeBetPot).
//! - **T5 ICM & Tournament** (`generate_icm`) — Push or fold in a tournament
//!   setting.  Uses a PushTier system: base threshold per tournament stage,
//!   adjusted by hand strength.
//! - **T9 Anti-Limper Isolation** (`generate_anti_limper`) — Iso-raise a limper,
//!   overlimp, or fold.  Premium/Strong hands always iso-raise; trash always
//!   folds.
//! - **T11 Squeeze Play** (`generate_squeeze`) — Facing an open + one or more
//!   callers from BTN: 3-bet squeeze with premiums, call playable hands for
//!   implied odds, fold marginal/trash.
//! - **T12 Big Blind Defense** (`generate_bb_defense`) — Facing a single raise
//!   from BB: 3-bet strong hands, call playable hands exploiting the BB
//!   discount, fold trash.

use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{classify_hand, hand_category_name, HandCategory},
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

// ═══════════════════════════════════════════════════════════════════════════
// T1 — Preflop Decision (PF-)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
enum PreflopSpot {
    OpenRaise,   // Hero faces a limped pot or everyone folded
    FacingOpen,  // Hero faces a single raise
    ThreeBetPot, // Hero faces a 3-bet after having opened
}

fn select_spot<R: Rng>(rng: &mut R) -> PreflopSpot {
    match rng.gen_range(0..3) {
        0 => PreflopSpot::OpenRaise,
        1 => PreflopSpot::FacingOpen,
        _ => PreflopSpot::ThreeBetPot,
    }
}

const POSITIONS_6MAX: &[Position] = &[
    Position::UTG, Position::HJ, Position::CO,
    Position::BTN, Position::SB, Position::BB,
];

const POSITIONS_9MAX: &[Position] = &[
    Position::UTG, Position::UTG1, Position::UTG2,
    Position::LJ, Position::HJ, Position::CO,
    Position::BTN, Position::SB, Position::BB,
];

fn random_position<R: Rng>(rng: &mut R, is_6max: bool) -> Position {
    let pool = if is_6max { POSITIONS_6MAX } else { POSITIONS_9MAX };
    pool[rng.gen_range(0..pool.len())]
}

fn stack_for_difficulty<R: Rng>(rng: &mut R, diff: DifficultyLevel) -> u32 {
    match diff {
        DifficultyLevel::Beginner     => rng.gen_range(80..=120),
        DifficultyLevel::Intermediate => rng.gen_range(40..=150),
        DifficultyLevel::Advanced     => rng.gen_range(15..=300),
    }
}

/// Backward-compatible entry point — delegates to [`generate_open`].
pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    generate_open(rng, difficulty, scenario_id, text_style)
}

/// T1 — Preflop Decision (PF-).
///
/// RNG order: gen_bool → gen_range(0..3) → position → stack → Deck::new_shuffled
/// → deal×2 → per-player stacks → build_spot (no further rng).
pub fn generate_open<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let is_6max = rng.gen_bool(0.5);
    let table_size = if is_6max { 6usize } else { 9 };
    let spot = select_spot(rng);
    let hero_pos = random_position(rng, is_6max);
    let effective_stack = stack_for_difficulty(rng, difficulty);

    let mut deck = Deck::new_shuffled(rng);
    let hero_cards: [Card; 2] = [deck.deal(), deck.deal()];

    let cat = classify_hand(hero_cards);
    let pos_type = if hero_pos.is_late() { "IP" } else { "OOP" };
    let branch_key = match spot {
        PreflopSpot::ThreeBetPot => format!("ThreeBetPot:{}", hand_category_name(cat)),
        PreflopSpot::OpenRaise   => format!("OpenRaise:{}:{}", hand_category_name(cat), pos_type),
        PreflopSpot::FacingOpen  => format!("FacingOpen:{}:{}", hand_category_name(cat), pos_type),
    };

    // Build player list (simplified)
    let positions = if is_6max { POSITIONS_6MAX } else { POSITIONS_9MAX };
    let players: Vec<PlayerState> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| PlayerState {
            seat: i as u8 + 1,
            position: pos,
            stack: if pos == hero_pos {
                effective_stack
            } else {
                stack_for_difficulty(rng, difficulty)
            },
            is_hero: pos == hero_pos,
            is_active: true,
        })
        .collect();

    let bb = 2u32;
    let (pot_size, current_bet, question, answers) =
        build_spot(rng, spot, hero_pos, hero_cards, effective_stack, bb, difficulty, table_size, text_style);

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand: hero_cards,
        board: vec![],
        players,
        pot_size,
        current_bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::PreflopDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

fn build_spot<R: Rng>(
    _rng: &mut R,
    spot: PreflopSpot,
    pos: Position,
    hand: [Card; 2],
    stack: u32,
    bb: u32,
    _difficulty: DifficultyLevel,
    table_size: usize,
    text_style: TextStyle,
) -> (u32, u32, String, Vec<AnswerOption>) {
    let cat = classify_hand(hand);
    let cat_name = hand_category_name(cat);
    let hand_str = format!("{}{}", hand[0], hand[1]);
    let pos_str = format!("{}", pos);
    let stack_bb = stack / bb;

    match spot {
        // ---- Hero acts first (open-raise spot) ----------------------------
        PreflopSpot::OpenRaise => {
            let pot = bb + bb / 2; // SB + BB already in
            let open_size = if stack_bb >= 40 { bb * 3 } else { bb * 2 };
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You have {hand_str} in {pos_str} at a {table_size}-handed table. \
                     Stack: {stack_bb} big blinds. Everyone before you folded. What do you do?"
                ),
                TextStyle::Technical => format!(
                    "You hold {hand_str} in {pos_str} at a {table_size}-handed table. \
                     Effective stack is {stack_bb} BB. Action folds to you. What is your action?"
                ),
            };
            // Single correct answer: raise if hand is strong enough, fold otherwise.
            // Limping is never correct here.
            let should_raise = matches!(cat, HandCategory::Premium | HandCategory::Strong)
                || (pos.is_late() && matches!(cat, HandCategory::Playable | HandCategory::Marginal));
            let correct = if should_raise { "B" } else { "A" };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand from {pos_str} lacks the equity and \
                         playability to justify entering the pot against a full range.")
            } else {
                format!("Too tight. {hand_str} ({cat_name}) from {pos_str} with {stack_bb} BB \
                         has enough value to open-raise profitably.")
            };
            let raise_body = if correct == "B" {
                format!("Correct. {hand_str} ({cat_name}) from {pos_str} merits an open-raise. \
                         A standard 2-3x sizing builds a pot with the best of it and denies \
                         equity to weaker holdings.")
            } else {
                format!("Raising with a {cat_name} hand from {pos_str} over-commits your stack \
                         ({stack_bb} BB) with insufficient equity. A fold is better.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} is too weak for {pos_str}. Folding saves your chips for a better hand.")
                        } else {
                            format!("Folding is a mistake here. {hand_str} is strong enough to bet from {pos_str}. Don't throw away the opportunity.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) from {pos_str} with {stack_bb} BB: \
                             {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: format!("Raise to {} BB", open_size / bb),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Raise! {hand_str} is a good hand in {pos_str}. Bet {open_size} chips and take control of the pot.")
                        } else {
                            format!("Raising is too risky here — {hand_str} isn't strong enough from {pos_str} with {stack_bb} big blinds. Fold instead.")
                        },
                        TextStyle::Technical => format!(
                            "Raising to {open_size} chips ({} BB) with {hand_str} ({cat_name}) \
                             from {pos_str}: {raise_body}",
                            open_size / bb
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: "Call".to_string(),
                    is_correct: false,
                    explanation: match text_style {
                        TextStyle::Simple => format!(
                            "Just calling the big blind here is a bad idea. It lets everyone in cheaply, and you lose control of the hand. Either raise or fold."
                        ),
                        TextStyle::Technical => format!(
                            "Limping with {hand_str} from {pos_str}: In most cash game formats \
                             limping is a leak — it invites multiway pots with no initiative, \
                             weakening your range and giving the BB a free squeeze opportunity."
                        ),
                    },
                },
            ];
            (pot, 0, q, answers)
        }

        // ---- Hero faces a single raise ------------------------------------
        PreflopSpot::FacingOpen => {
            let raiser_size = if stack_bb >= 40 { bb * 3 } else { bb * 2 };
            let pot = bb / 2 + bb + raiser_size; // SB + BB + open
            let three_bet = raiser_size * 3;
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You have {hand_str} in {pos_str} ({stack_bb} big blinds). \
                     Someone raised to {} big blinds. What do you do?",
                    raiser_size / bb
                ),
                TextStyle::Technical => format!(
                    "You hold {hand_str} in {pos_str} ({stack_bb} BB deep). \
                     A player raises to {} BB. Action is on you. What do you do?",
                    raiser_size / bb
                ),
            };
            // Single correct answer to guarantee invariant.
            let correct = match cat {
                HandCategory::Premium  => "C",  // 3-bet for value
                HandCategory::Strong   => "C",  // 3-bet (may also call, 3-bet is cleaner)
                HandCategory::Playable => if pos.is_late() { "C" } else { "B" },
                HandCategory::Marginal => "A",  // fold regardless of position
                HandCategory::Trash    => "A",  // always fold trash vs a raise
            };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand vs a raise from {pos_str} is an easy fold. \
                         This hand lacks equity to continue profitably against a raising range.")
            } else {
                format!("Too tight. {hand_str} ({cat_name}) has sufficient equity against a \
                         typical raising range to continue from {pos_str}.")
            };
            let call_body = if correct == "B" {
                format!("Correct. Calling with {hand_str} ({cat_name}) from {pos_str} retains \
                         pot control and lets you realize equity with positional advantage.")
            } else if correct == "A" {
                format!("Calling with a {cat_name} hand invests chips without sufficient equity. \
                         A fold is cleaner from {pos_str}.")
            } else {
                format!("Calling with {hand_str} ({cat_name}) is too passive — a 3-bet for \
                         value is higher EV here from {pos_str}.")
            };
            let threebet_body = if correct == "C" {
                format!("Correct. A 3-bet with {hand_str} ({cat_name}) from {pos_str} extracts \
                         value from worse hands, denies equity, and builds a pot with an equity \
                         advantage.")
            } else {
                format!("3-betting a {cat_name} hand bloats the pot unfavourably from {pos_str}. \
                         You risk a 4-bet or playing a large pot with insufficient hand strength.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} from {pos_str} isn't strong enough to call or re-raise here. Save your chips.")
                        } else {
                            format!("Folding is too cautious — {hand_str} is good enough to continue here.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) vs a raise from {pos_str}: {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: "Call".to_string(),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Correct. Call with {hand_str} from {pos_str}. You have a decent hand and a good position — see the flop.")
                        } else if correct == "A" {
                            format!("Calling with {hand_str} isn't worth it — this hand can't beat a raise. Fold.")
                        } else {
                            format!("Calling is too passive here — re-raise with {hand_str} to build the pot while you have the advantage.")
                        },
                        TextStyle::Technical => format!(
                            "Calling with {hand_str} ({cat_name}) in {pos_str}: {call_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: format!("Raise to {} BB", three_bet / bb),
                    is_correct: correct == "C",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "C" {
                            format!("Re-raise! {hand_str} from {pos_str} is strong enough to bet big. This builds the pot when you have the best hand.")
                        } else {
                            format!("Re-raising {hand_str} here is too risky. You'd be putting in a lot of chips with a hand that isn't strong enough.")
                        },
                        TextStyle::Technical => format!(
                            "3-betting to {three_bet} with {hand_str} ({cat_name}) from \
                             {pos_str}: {threebet_body}"
                        ),
                    },
                },
            ];
            (pot, raiser_size, q, answers)
        }

        // ---- Hero faces a 3-bet after opening ----------------------------
        PreflopSpot::ThreeBetPot => {
            let hero_open = bb * 3;
            let three_bet_size = hero_open * 3;
            let pot = bb / 2 + bb + hero_open + three_bet_size;
            let four_bet = three_bet_size * 3;
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You bet {} big blinds with {hand_str} from {pos_str} \
                     ({stack_bb} big blinds). Your opponent re-raised to {} big blinds. What do you do?",
                    hero_open / bb,
                    three_bet_size / bb
                ),
                TextStyle::Technical => format!(
                    "You opened to {} BB with {hand_str} from {pos_str} \
                     ({stack_bb} BB deep). A player re-raises to {} BB. What do you do?",
                    hero_open / bb,
                    three_bet_size / bb
                ),
            };
            let correct = match cat {
                HandCategory::Premium  => "C", // 4-bet for value
                HandCategory::Strong   => "B", // call or small 4-bet
                HandCategory::Playable => "B", // call with implied odds
                HandCategory::Marginal => "A", // fold
                HandCategory::Trash    => "A", // fold
            };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand cannot profitably continue vs a 3-bet \
                         given the pot odds and likely 3-bet range of your opponent.")
            } else {
                "Folding is too tight here. Your hand has sufficient equity against a \
                 balanced 3-bet range to continue.".to_string()
            };
            let call_body = if correct == "B" {
                "Correct. Calling preserves stack depth and lets you navigate postflop with \
                 good implied odds, avoiding getting all-in preflop with a dominated or \
                 marginal hand.".to_string()
            } else {
                "Simply calling here leaves money on the table with a premium holding — \
                 4-betting for value is higher EV against a 3-bet range.".to_string()
            };
            let fourbet_body = if correct == "C" {
                "Correct. With a premium hand you should 4-bet for value. This polarizes \
                 your range, forces folds of hands with equity against you, and builds the \
                 pot with the best of it.".to_string()
            } else {
                format!("4-betting a {cat_name} hand turns your hand face-up and gets called \
                         (or shoved) by hands that dominate you, resulting in a large pot \
                         with negative EV.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} can't beat your opponent's re-raise range profitably. Let this one go.")
                        } else {
                            format!("Folding here is too cautious — you have enough of a hand to continue.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) vs 3-bet: {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: "Call".to_string(),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Correct. Call and see the flop. {hand_str} has good enough potential and you keep the pot manageable.")
                        } else {
                            format!("Just calling here wastes the opportunity — re-raise for value with this strong hand.")
                        },
                        TextStyle::Technical => format!(
                            "Calling the 3-bet with {hand_str} ({cat_name}) from {pos_str}: \
                             {call_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: format!("Raise to {} BB", four_bet / bb),
                    is_correct: correct == "C",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "C" {
                            format!("Correct. Re-raise again! {hand_str} is a premium hand. Build the pot — you have the best of it here.")
                        } else {
                            format!("Re-raising here puts too many chips at risk with {hand_str}. Call or fold instead.")
                        },
                        TextStyle::Technical => format!(
                            "4-betting with {hand_str} ({cat_name}): {fourbet_body}"
                        ),
                    },
                },
            ];
            (pot, three_bet_size, q, answers)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T5 — ICM & Tournament Decision (IC-)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub enum TournamentStage {
    EarlyLevels,
    MiddleStages,
    Bubble,
    FinalTable,
}

impl std::fmt::Display for TournamentStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TournamentStage::EarlyLevels  => write!(f, "Early Levels"),
            TournamentStage::MiddleStages => write!(f, "Middle Stages"),
            TournamentStage::Bubble       => write!(f, "Bubble"),
            TournamentStage::FinalTable   => write!(f, "Final Table"),
        }
    }
}

fn random_stage<R: Rng>(rng: &mut R) -> TournamentStage {
    match rng.gen_range(0..4) {
        0 => TournamentStage::EarlyLevels,
        1 => TournamentStage::MiddleStages,
        2 => TournamentStage::Bubble,
        _ => TournamentStage::FinalTable,
    }
}

/// Hand strength tiers for push/fold (simplified).
#[derive(Debug, Clone, Copy)]
enum PushTier {
    Premium,  // AA, KK, QQ, AKs — always push short stacks
    Strong,   // JJ, TT, AQ, AK — push at moderate depths
    Playable, // Mid pairs, suited broadways — push only when short
    Weak,     // Everything else — only push when desperate
}

fn classify_push_tier(hand: [Card; 2]) -> PushTier {
    let (r1, r2) = {
        let mut ranks = [hand[0].rank.0, hand[1].rank.0];
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        (ranks[0], ranks[1])
    };
    let suited = hand[0].suit == hand[1].suit;
    let pair = r1 == r2;

    if pair && r1 >= 12 { return PushTier::Premium; }       // QQ+
    if r1 == 14 && r2 == 13 && suited { return PushTier::Premium; } // AKs
    if pair && r1 >= 10 { return PushTier::Strong; }         // JJ, TT
    if r1 == 14 && r2 >= 12 { return PushTier::Strong; }    // AK, AQ
    if pair && r1 >= 7 { return PushTier::Playable; }        // 77-99
    if r1 == 14 && r2 >= 10 && suited { return PushTier::Playable; } // ATs+
    if r1 >= 12 && r2 >= 11 && suited { return PushTier::Playable; } // KQs, KJs, QJs
    PushTier::Weak
}

/// Simplified ICM pressure: base threshold in BB modified by hand strength.
/// Real ICM requires knowing payouts; here we use simplified thresholds.
fn push_threshold_bb(stage: TournamentStage, tier: PushTier) -> u32 {
    let base = match stage {
        TournamentStage::EarlyLevels  => 20,
        TournamentStage::MiddleStages => 15,
        TournamentStage::Bubble       => 10,
        TournamentStage::FinalTable   => 12,
    };
    // Premium hands can push at deeper stacks; weak hands need more desperation
    match tier {
        PushTier::Premium  => base + 8,
        PushTier::Strong   => base + 3,
        PushTier::Playable => base,
        PushTier::Weak     => base.saturating_sub(4),
    }
}

/// T5 — ICM & Tournament Decision (IC-).
///
/// RNG order: gen_range(0..4) for stage → hero_stack → villain_stack →
/// players_remaining → Deck::new_shuffled → deal×2.
pub fn generate_icm<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let stage = random_stage(rng);
    let bb = 100u32; // tournament chips, 100 = 1 BB

    let hero_stack_bb = match difficulty {
        DifficultyLevel::Beginner     => rng.gen_range(6..=18u32),
        DifficultyLevel::Intermediate => rng.gen_range(4..=25),
        DifficultyLevel::Advanced     => rng.gen_range(3..=30),
    };

    let villain_stack_bb: u32 = rng.gen_range(20..=60);
    let hero_stack = hero_stack_bb * bb;
    let villain_stack = villain_stack_bb * bb;

    let players_remaining = match stage {
        TournamentStage::EarlyLevels  => rng.gen_range(60..=120u32),
        TournamentStage::MiddleStages => rng.gen_range(25..=60),
        TournamentStage::Bubble       => rng.gen_range(10..=18),
        TournamentStage::FinalTable   => rng.gen_range(3..=9),
    };

    let paid_spots = (players_remaining as f32 * 0.15).ceil() as u32;

    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let hero_pos = Position::BTN;
    let pos_str = format!("{}", hero_pos);
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);

    let push_tier = classify_push_tier(hero_hand);
    let threshold = push_threshold_bb(stage, push_tier);
    let should_push = hero_stack_bb <= threshold;

    let stage_name = match stage {
        TournamentStage::EarlyLevels  => "Early",
        TournamentStage::MiddleStages => "Middle",
        TournamentStage::Bubble       => "Bubble",
        TournamentStage::FinalTable   => "FinalTable",
    };
    let branch_key = format!("{}:{}", stage_name, if should_push { "Push" } else { "Fold" });

    let pot = bb + bb / 2; // standard antes + blinds estimate

    let risk_premium_pct: f32 = match stage {
        TournamentStage::Bubble       => 20.0,
        TournamentStage::FinalTable   => 15.0,
        TournamentStage::MiddleStages => 8.0,
        TournamentStage::EarlyLevels  => 3.0,
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "Tournament: {stage}. {players_remaining} players left, top {paid_spots} get paid. \
             You have {hand_str} on the Button with {hero_stack_bb} big blinds. \
             Your opponent in the Big Blind has {villain_stack_bb} big blinds. \
             Everyone else folded. Go all-in or fold?"
        ),
        TextStyle::Technical => format!(
            "Tournament: {stage}. {players_remaining} players remain, top {paid_spots} paid. \
             You hold {hand_str} on the {pos_str} with {hero_stack_bb} BB. \
             Villain on the BB has {villain_stack_bb} BB. \
             Action folds to you. Do you shove all-in or fold?"
        ),
    };

    let push_body = if should_push {
        format!(
            "Correct. At {hero_stack_bb} BB, your stack faces significant blind pressure \
             (you'll lose ~{:.0}% per orbit). ICM risk premium at this stage is ~{risk_premium_pct:.0}%, \
             but your hand still has enough equity to profitably shove against a \
             wide BB calling range. Stack preservation via folding only deepens the \
             blinds crisis.",
            100.0 / hero_stack_bb as f32
        )
    } else {
        format!(
            "Shoving with {hero_stack_bb} BB is premature. At this stack depth the \
             ICM risk premium (~{risk_premium_pct:.0}% at {stage}) means you \
             over-risk your tournament equity. Wait for a better spot or a stronger \
             hand."
        )
    };
    let push_explanation = match text_style {
        TextStyle::Simple => if should_push {
            format!("Correct — go all-in! With only {hero_stack_bb} big blinds, your stack is shrinking fast. Waiting for a perfect hand will cost you too much. Shove now.")
        } else {
            format!("Going all-in too early at {hero_stack_bb} big blinds risks your tournament life needlessly. You still have time to find a better spot.")
        },
        TextStyle::Technical => format!(
            "Shoving {hero_stack_bb} BB with {hand_str} from {pos_str} during {stage}: {push_body}"
        ),
    };

    let fold_body = if !should_push {
        format!(
            "Correct. With {hero_stack_bb} BB you are not yet in desperation territory. \
             Preserving your stack when ICM pressure is ~{risk_premium_pct:.0}% \
             is rational — a marginal shove risks your entire tournament life \
             for a modest chip gain."
        )
    } else {
        format!(
            "Folding is too passive here. With only {hero_stack_bb} BB and increasing \
             blind levels, you must find spots to accumulate chips. Folding here \
             leaves you critically short and forces even worse all-in spots later \
             with less fold equity."
        )
    };
    let fold_explanation = match text_style {
        TextStyle::Simple => if !should_push {
            format!("Correct — fold. You still have enough chips ({hero_stack_bb} big blinds) to wait for a better spot. Don't risk elimination unnecessarily.")
        } else {
            format!("Folding here is wrong — with {hero_stack_bb} big blinds your stack is getting dangerously low. You need to shove while you still have some chips to be scary.")
        },
        TextStyle::Technical => format!(
            "Folding {hand_str} from {pos_str} with {hero_stack_bb} BB during {stage}: {fold_body}"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "All-in".to_string(),
            is_correct: should_push,
            explanation: push_explanation,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Fold".to_string(),
            is_correct: !should_push,
            explanation: fold_explanation,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1,
            position: Position::BB,
            stack: villain_stack,
            is_hero: false,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: hero_pos,
            stack: hero_stack,
            is_hero: true,
            is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::Tournament,
        hero_position: hero_pos,
        hero_hand,
        board: vec![],
        players,
        pot_size: pot,
        current_bet: 0,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::ICMAndTournamentDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T9 — Anti-Limper Isolation (AL-)
// ═══════════════════════════════════════════════════════════════════════════

fn iso_raise_bb(limper_count: u8) -> u32 {
    match limper_count {
        1 => 4,
        2 => 5,
        _ => 6, // 3+ limpers
    }
}

fn al_is_in_position(pos: Position) -> bool {
    matches!(pos, Position::CO | Position::BTN)
}

fn al_position_label(pos: Position) -> &'static str {
    match pos {
        Position::CO  => "Cutoff",
        Position::BTN => "Button",
        Position::SB  => "Small Blind",
        _             => "Unknown",
    }
}

/// T9 — Anti-Limper Isolation (AL-).
///
/// RNG order: Deck::new_shuffled → deal×2 → gen_range(0..3) for position →
/// gen_range(1..=3) for limper count → stack.
pub fn generate_anti_limper<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];

    let hero_pos = match rng.gen_range(0..3) {
        0 => Position::CO,
        1 => Position::BTN,
        _ => Position::SB,
    };

    let limper_count: u8 = rng.gen_range(1..=3);
    let ip = al_is_in_position(hero_pos);

    let bb = 2u32;
    let stack_bb: u32 = match difficulty {
        DifficultyLevel::Beginner     => rng.gen_range(60..=120),
        DifficultyLevel::Intermediate => rng.gen_range(30..=150),
        DifficultyLevel::Advanced     => rng.gen_range(15..=200),
    };
    let stack = stack_bb * bb;
    let pot = bb + (bb / 2) + (bb * limper_count as u32); // BB + SB + limpers

    let cat = classify_hand(hero_hand);
    let iso_bb = iso_raise_bb(limper_count);
    let iso_chips = iso_bb * bb;

    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let pos_str = al_position_label(hero_pos);
    let limper_word = if limper_count == 1 { "limper" } else { "limpers" };
    let pos_qualifier = if ip { "in position" } else { "out of position" };

    // Correct answer (single ID):
    // Premium/Strong                   → "C" (Iso-raise always)
    // Playable + IP (CO/BTN)           → "C" (Iso-raise with position)
    // Playable + SB (OOP)              → "B" (Overlimp)
    // Marginal/Trash                   → "A" (Fold)
    let correct: &str = match cat {
        HandCategory::Premium | HandCategory::Strong         => "C",
        HandCategory::Playable if ip                         => "C",
        HandCategory::Playable                               => "B",
        HandCategory::Marginal | HandCategory::Trash         => "A",
    };

    let branch_key = match (cat, ip) {
        (HandCategory::Premium, _)      => "Premium".to_string(),
        (HandCategory::Strong, _)       => "Strong".to_string(),
        (HandCategory::Playable, true)  => "Playable:IP".to_string(),
        (HandCategory::Playable, false) => "Playable:OOP".to_string(),
        (HandCategory::Marginal, _)     => "Marginal".to_string(),
        (HandCategory::Trash, _)        => "Trash".to_string(),
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You have {hand_str} in {pos_str} ({stack_bb} big blinds). \
             {limper_count} player(s) just called the big blind without raising. \
             Pot: {pot} chips. What do you do?"
        ),
        TextStyle::Technical => format!(
            "You hold {hand_str} ({cat}) on the {pos_str} ({pos_qualifier}, {stack_bb} BB deep). \
             {limper_count} player(s) limp in front of you. Pot is {pot} chips. \
             What is your action?"
        ),
    };

    // --- Explanations ---

    let fold_exp = match text_style {
        TextStyle::Simple => if matches!(cat, HandCategory::Marginal | HandCategory::Trash) {
            format!(
                "Correct — fold. {hand_str} isn't strong enough here, even against players who just called. Wait for a better hand."
            )
        } else {
            format!(
                "Folding {hand_str} here is too cautious — you have enough of a hand to bet and take control."
            )
        },
        TextStyle::Technical => if matches!(cat, HandCategory::Marginal | HandCategory::Trash) {
            format!(
                "Correct. A {cat} hand from {pos_str} against {limper_count} {limper_word} is a \
                 clear fold. Iso-raising with {hand_str} builds a large pot without sufficient \
                 equity against even limping ranges. Overlimping is even worse — it invites more \
                 players and removes any initiative. Fold and wait for a stronger hand."
            )
        } else {
            format!(
                "Folding {hand_str} ({cat}) from {pos_str} is too tight. You have enough hand \
                 strength and/or positional advantage to profitably enter the pot here. \
                 Limpers have shown weakness — exploit it."
            )
        },
    };

    let overlimp_exp = match text_style {
        TextStyle::Simple => match (cat, ip) {
            (HandCategory::Playable, false) => format!(
                "Correct — just call. With {hand_str} from the Small Blind (you'll act first all game), raising is risky. Call cheaply and see if you hit the flop."
            ),
            _ => if ip {
                format!(
                    "Just calling here wastes your positional advantage. You're acting last — raise to take control and play heads-up."
                )
            } else {
                format!(
                    "Just calling here is too passive with {hand_str}. Raise — you have a strong enough hand to take control."
                )
            },
        },
        TextStyle::Technical => match (cat, ip) {
            (HandCategory::Playable, false) => format!(
                "Correct. Overlimping with {hand_str} ({cat}) from the Small Blind is the best \
                 play. Iso-raising to {iso_chips} chips would build a large pot that you'll play \
                 from the worst position at the table (OOP every street). Instead, calling 1 BB \
                 lets you see a cheap flop with a speculative hand and realise implied odds \
                 without committing too many chips. Note: iso-raise from CO or BTN with this hand."
            ),
            _ => format!(
                "Overlimping with {hand_str} ({cat}) from {pos_str} is too passive. \
                 {}",
                if ip {
                    format!(
                        "You have positional advantage (IP) — iso-raising to {iso_chips} chips \
                         ({iso_bb} BB) is higher EV. It denies limpers' cheap flops, wins \
                         dead money outright sometimes, and sets up a profitable postflop spot \
                         in position."
                    )
                } else {
                    format!(
                        "This hand is too strong to just call — iso-raise to {iso_chips} chips \
                         ({iso_bb} BB) to punish the limpers and build the pot with initiative."
                    )
                }
            ),
        },
    };

    let iso_exp = match text_style {
        TextStyle::Simple => match (cat, ip) {
            (HandCategory::Premium | HandCategory::Strong, _) => format!(
                "Correct — raise to {iso_chips} chips ({iso_bb} big blinds)! You have a strong hand. Don't let the other players see a cheap flop — make them pay or fold."
            ),
            (HandCategory::Playable, true) => format!(
                "Correct — raise to {iso_chips} chips ({iso_bb} big blinds)! You'll be acting last all hand, which is a big advantage. Raise to play heads-up in a strong position."
            ),
            _ => format!(
                "Raising here puts a lot of chips into a pot where you'll be acting first every street — a tough spot with {hand_str}. Call or fold instead."
            ),
        },
        TextStyle::Technical => match (cat, ip) {
            (HandCategory::Premium | HandCategory::Strong, _) => format!(
                "Correct. Iso-raising to {iso_chips} chips ({iso_bb} BB) with {hand_str} ({cat}) \
                 is mandatory from {pos_str}. You never let limpers see a cheap flop with a \
                 premium or strong hand. The raise: (1) defines your hand as strong, \
                 (2) builds a pot with an equity advantage, (3) often wins uncontested vs \
                 {limper_count} {limper_word}. Size is {iso_bb} BB to account for {limper_count} \
                 limper(s) already in the pot."
            ),
            (HandCategory::Playable, true) => format!(
                "Correct. Iso-raising to {iso_chips} chips ({iso_bb} BB) with {hand_str} ({cat}) \
                 from {pos_str} (IP) is correct. Limpers are almost always weaker than a raiser's \
                 range. With positional advantage postflop you can: c-bet profitably on a wide \
                 range of boards, win with fold equity, and extract value when you connect. \
                 {iso_bb} BB accounts for {limper_count} {limper_word} already limped."
            ),
            _ => format!(
                "Iso-raising to {iso_chips} chips with {hand_str} ({cat}) from {pos_str} \
                 (OOP) builds too large a pot to play from the worst position at the table. \
                 With a {cat} hand OOP, overlimping or folding is better than iso-raising."
            ),
        },
    };

    let players = vec![
        // Simplified: show one limper + hero
        PlayerState {
            seat: 1,
            position: Position::UTG,
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
            explanation: overlimp_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {} BB", iso_bb),
            is_correct: correct == "C",
            explanation: iso_exp,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board: vec![],
        players,
        pot_size: pot,
        current_bet: bb, // the limp amount
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::AntiLimperIsolation,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T11 — Squeeze Play (SQ-)
// ═══════════════════════════════════════════════════════════════════════════

/// Strength of hero's holding in a preflop squeeze spot.
#[derive(Debug, Clone, Copy)]
enum HoleStrength {
    Premium,     // AA, KK, QQ, AKs — squeeze for maximum value
    Speculative, // Mid pairs 77–99, suited connectors — call for implied odds
    Weak,        // Off-suit rag, dominated holdings — fold
}

impl std::fmt::Display for HoleStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoleStrength::Premium     => write!(f, "premium (AA/KK/QQ/AKs)"),
            HoleStrength::Speculative => write!(f, "speculative (mid pair / suited connector)"),
            HoleStrength::Weak        => write!(f, "weak (off-suit rag / dominated hand)"),
        }
    }
}

/// T11 — Squeeze Play (SQ-).
///
/// RNG order: Deck::new_shuffled → deal×2 → gen_range(0..3) for strength →
/// callers → sizing.
pub fn generate_squeeze<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];

    let strength = match rng.gen_range(0..3) {
        0 => HoleStrength::Premium,
        1 => HoleStrength::Speculative,
        _ => HoleStrength::Weak,
    };

    let callers: u32 = match difficulty {
        DifficultyLevel::Beginner     => 1,
        DifficultyLevel::Intermediate => rng.gen_range(1..=2),
        DifficultyLevel::Advanced     => rng.gen_range(1..=3),
    };

    let bb = 2u32;
    let (open_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (3u32, 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(2..=4), rng.gen_range(60..=120)),
        DifficultyLevel::Advanced     => (rng.gen_range(2..=5), rng.gen_range(25..=150)),
    };

    // Dead money before hero acts: open + callers × open + SB (1 BB simplified)
    let pot_bb = open_bb + callers * open_bb + 1;
    let pot    = pot_bb * bb;
    let stack  = stack_bb * bb;

    // Squeeze sizing: ~3× the open + 1 open per caller (isolation premium)
    let squeeze_bb = open_bb * 3 + callers * open_bb;
    let squeeze    = squeeze_bb * bb;

    // Correct action:
    // Premium     → Squeeze (dominate the field, build a pot you likely win)
    // Speculative → Call   (implied odds justify set/draw play with callers in)
    // Weak        → Fold   (dominated equity, no profitable path)
    let correct: &str = match strength {
        HoleStrength::Premium     => "C",
        HoleStrength::Speculative => "B",
        HoleStrength::Weak        => "A",
    };

    let branch_key = match strength {
        HoleStrength::Premium     => "Premium:Squeeze",
        HoleStrength::Speculative => "Speculative:Call",
        HoleStrength::Weak        => "Weak:Fold",
    }.to_string();

    let hero_pos   = Position::BTN;
    let opener_pos = Position::UTG;
    let hand_str   = format!("{}{}", hero_hand[0], hero_hand[1]);
    let caller_str = if callers == 1 {
        "1 caller".to_string()
    } else {
        format!("{callers} callers")
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "Before the flop. You have {hand_str} on the Button. \
             One player raised to {open_bb} big blinds and {caller_str} called. \
             Pot: {pot} chips. Stack: {stack} chips. \
             A big re-raise would be ~{squeeze_bb} big blinds. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Preflop squeeze. You hold {hand_str} ({strength}) on the Button. \
             UTG opens to {open_bb} BB, {caller_str} in between. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             A squeeze would be ~{squeeze} chips ({squeeze_bb} BB). \
             What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Weak => format!(
                        "Correct — fold. Your hand isn't strong enough to enter a large pot against multiple active players."
                    ),
                    _ => format!(
                        "Folding here is too cautious — you have a good enough hand to re-raise or call."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Weak => format!(
                        "Correct. Folding a {strength} in this squeeze spot is the right play. \
                         Your hand has poor equity against the opener's range and the callers — \
                         even the BTN pot-odds discount doesn't compensate for dominated equity. \
                         Wait for a better spot."
                    ),
                    _ => format!(
                        "Folding a {strength} gives up significant equity. Premium hands profit \
                         most when the pot is large and opponents are dominated. Speculative \
                         hands need callers for implied odds. Only fold genuinely weak holdings."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({open_bb} BB)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Speculative => format!(
                        "Correct — call. With a hand that plays well in big pots, you can call and try to hit a big hand on the flop."
                    ),
                    HoleStrength::Premium => format!(
                        "Just calling isn't the best play here — re-raise to take control and thin the field."
                    ),
                    HoleStrength::Weak => format!(
                        "Just calling isn't the best play here — re-raise to take control and thin the field."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Speculative => format!(
                        "Correct. Calling with a {strength} is optimal. Multiple callers create \
                         a large pot and improve your implied odds for sets, straights, and \
                         flushes. Squeezing bloats the pot where you may be dominated. Calling \
                         keeps your range disguised and preserves the implied-odds edge."
                    ),
                    HoleStrength::Premium => format!(
                        "Calling with a {strength} leaves too much value on the table. You have \
                         a massive equity edge — squeezing forces dominated hands to pay a steep \
                         price, often winning the dead money outright or playing a large pot as \
                         a significant favourite."
                    ),
                    HoleStrength::Weak => format!(
                        "Calling with a {strength} is -EV. You have poor equity against both the \
                         opener and the callers with no strong implied-odds potential. Fold."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Squeeze to {squeeze} chips ({squeeze_bb} BB)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Premium => format!(
                        "Correct — re-raise big! With {hand_str} you have a great hand. A big re-raise will often win the pot right now, or leave you heads-up against one player with the best hand."
                    ),
                    HoleStrength::Speculative => format!(
                        "Re-raising here with {hand_str} isn't justified. Your hand isn't strong enough to play a huge pot."
                    ),
                    HoleStrength::Weak => format!(
                        "Re-raising here with {hand_str} isn't justified. Your hand isn't strong enough to play a huge pot."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Premium => format!(
                        "Correct. Squeezing to {squeeze_bb} BB with a {strength} is the highest-EV \
                         play. Your hand has dominant equity over the field. A squeeze isolates, \
                         collects dead money when folds come, and builds a large pot played as a \
                         heavy favourite when called. Never limp or flat with premium hands in a \
                         squeeze spot."
                    ),
                    HoleStrength::Speculative => format!(
                        "Squeezing with a {strength} turns a good implied-odds hand into a \
                         commitment bluff. If called, you are out of position with a hand that \
                         needs to hit the board — facing a range that calls 3-bets and likely \
                         dominates you. Calling is higher EV."
                    ),
                    HoleStrength::Weak => format!(
                        "Squeezing with a {strength} is a low-equity bluff. The opener and \
                         callers have uncapped ranges — expect 4-bets and calls from better \
                         hands. This play has strongly negative expected value."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: opener_pos, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,   stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::SqueezePlay,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board:         vec![],
            players,
            pot_size:      pot,
            current_bet:   open_bb * bb,
        },
        question,
        answers,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T12 — Big Blind Defense (BD-)
// ═══════════════════════════════════════════════════════════════════════════

/// Hero's holding category when defending the Big Blind against a single raise.
#[derive(Debug, Clone, Copy)]
enum DefenseStrength {
    Strong,   // JJ+, AK, AQs — 3-bet for value from the BB
    Playable, // 22–TT, suited connectors, broadways — call with pot-odds discount
    Weak,     // Off-suit non-broadway trash — fold even from the BB
}

impl std::fmt::Display for DefenseStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefenseStrength::Strong   => write!(f, "strong (JJ+/AK/AQs)"),
            DefenseStrength::Playable => write!(f, "playable (mid pair / suited connector / broadway)"),
            DefenseStrength::Weak     => write!(f, "weak (off-suit trash)"),
        }
    }
}

/// T12 — Big Blind Defense (BD-).
///
/// RNG order: Deck::new_shuffled → deal×2 → gen_range(0..3) for strength →
/// gen_range(0..3) for villain_pos → sizing.
pub fn generate_bb_defense<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];

    let strength = match rng.gen_range(0..3) {
        0 => DefenseStrength::Strong,
        1 => DefenseStrength::Playable,
        _ => DefenseStrength::Weak,
    };

    // Villain's position determines their opening range width
    let villain_pos = match rng.gen_range(0..3) {
        0 => Position::UTG,
        1 => Position::CO,
        _ => Position::BTN,
    };

    let bb = 2u32;
    let (raise_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (3u32, 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(2..=4), rng.gen_range(60..=120)),
        DifficultyLevel::Advanced     => (rng.gen_range(2..=5), rng.gen_range(25..=150)),
    };

    // Pot before hero acts: raise + 1 BB (hero's dead big blind; SB folds)
    let pot_bb    = raise_bb + 1;
    let pot       = pot_bb * bb;
    let stack     = stack_bb * bb;

    // Standard BB 3-bet sizing: ~3× raise + 1 dead BB re-invested
    let three_bet_bb = raise_bb * 3 + 1;
    let three_bet    = three_bet_bb * bb;

    // Correct action:
    // Strong   → 3-bet (value 3-bet, build pot with equity advantage)
    // Playable → Call  (BB discount makes defence profitable; good implied odds)
    // Weak     → Fold  (even the BB discount can't save off-suit trash)
    let correct: &str = match strength {
        DefenseStrength::Strong   => "C",
        DefenseStrength::Playable => "B",
        DefenseStrength::Weak     => "A",
    };

    let branch_key = match strength {
        DefenseStrength::Strong   => "Strong:ThreeBet",
        DefenseStrength::Playable => "Playable:Call",
        DefenseStrength::Weak     => "Weak:Fold",
    }.to_string();

    let hero_pos = Position::BB;
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Before the flop. You have {hand_str} in the Big Blind. \
             {villain_pos} raised to {raise_bb} big blinds. Everyone else folded. \
             Pot: {pot} chips. Stack: {stack} chips. \
             A re-raise would be ~{three_bet_bb} big blinds. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Big Blind defense. You hold {hand_str} ({strength}) in the Big Blind. \
             {villain_pos} raises to {raise_bb} BB. Action folds to you. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             A 3-bet would be ~{three_bet} chips ({three_bet_bb} BB). \
             What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    DefenseStrength::Weak => format!(
                        "Correct — fold. Even with your money already in, {hand_str} isn't strong enough to continue against this raise."
                    ),
                    _ => format!(
                        "Folding here throws away money you already put in. You have a playable hand — call at minimum."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Weak => format!(
                        "Correct. Folding a {strength} from the BB is correct even with the \
                         pot-odds discount. Off-suit non-broadway hands have poor equity and \
                         will routinely flop dominated pairs, no draws, and difficult second-best \
                         spots. Save the chips for a better hand."
                    ),
                    _ => format!(
                        "Folding a {strength} from the BB is too tight. You already have 1 BB \
                         invested and are getting a direct pot-odds discount. Strong hands should \
                         3-bet; playable hands should call. Only trash warrants a fold here."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({raise_bb} BB)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    DefenseStrength::Playable => format!(
                        "Correct — call! You're already part-way in with the Big Blind and {hand_str} can see a flop at a discount. Don't fold this away."
                    ),
                    DefenseStrength::Strong => format!(
                        "Just calling is too passive — re-raise with this strong hand to build the pot."
                    ),
                    DefenseStrength::Weak => format!(
                        "Just calling is too passive — re-raise with this strong hand to build the pot."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Playable => format!(
                        "Correct. Calling with a {strength} from the BB is the best play. \
                         Your pot-odds discount and direct equity make defence profitable. \
                         Playable hands (pairs, suited connectors, broadways) have enough equity \
                         and implied odds to justify calling {raise_bb} BB and seeing a flop. \
                         Avoid 3-betting marginal hands OOP."
                    ),
                    DefenseStrength::Strong => format!(
                        "Calling with a {strength} from the BB misses a value opportunity. \
                         You have a large equity advantage over {villain_pos}'s range — a 3-bet \
                         builds the pot while you're ahead and may win the dead money outright. \
                         Calling allows villain to realise their equity cheaply."
                    ),
                    DefenseStrength::Weak => format!(
                        "Calling with a {strength} from the BB is still -EV. The pot-odds \
                         discount helps but doesn't overcome the fact that off-suit trash has \
                         minimal equity, poor flop-hit rate, and faces a strong opening range. \
                         Fold and wait."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("3-bet to {three_bet} chips ({three_bet_bb} BB)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    DefenseStrength::Strong => format!(
                        "Correct — re-raise to {three_bet_bb} big blinds! {hand_str} is a strong hand. Make your opponent pay to continue and take control of the pot."
                    ),
                    _ => format!(
                        "Re-raising here is too aggressive with {hand_str}. Just call and see the flop."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Strong => format!(
                        "Correct. 3-betting to {three_bet_bb} BB with a {strength} from the BB \
                         is the highest-EV play. You have a significant equity advantage over \
                         {villain_pos}'s opening range. A 3-bet builds the pot while you're ahead, \
                         denies equity to dominated hands, and forces a tough decision. Against a \
                         wide opener (CO/BTN) this is even more profitable."
                    ),
                    DefenseStrength::Playable => format!(
                        "3-betting with a {strength} from the BB turns a +EV call into a \
                         marginal bluff-raise. Playable hands do not have enough raw equity to \
                         3-bet for value against most opening ranges, and bloating the pot OOP \
                         with a speculative hand is risky. Calling preserves implied odds."
                    ),
                    DefenseStrength::Weak => format!(
                        "3-betting with a {strength} from the BB is a bluff with no equity \
                         foundation. Even if it works occasionally, this play has poor \
                         risk-to-reward — villain can 4-bet or call with many better hands. \
                         Fold instead."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: villain_pos, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,    stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::BigBlindDefense,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board:         vec![],
            players,
            pot_size:      pot,
            current_bet:   raise_bb * bb,
        },
        question,
        answers,
    }
}
