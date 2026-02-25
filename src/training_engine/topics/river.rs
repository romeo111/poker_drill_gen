//! River topic generators: bluff spot, value bet, and call-or-fold.
//!
//! All three topics deal a full 5-card board and ask hero to act on the river.
//!
//! - **T4 Bluff Spot** — Hero has no showdown value.  Decision depends on the
//!   bluff type (missed flush draw / capped range / bricked overcards) and the
//!   stack-to-pot ratio (SPR).  Low SPR → check; high SPR → large bluff.
//! - **T10 River Value Bet** — Hero has a made hand.  Sizing depends on hand
//!   strength: overbet the nuts, large bet a strong hand, check a medium hand.
//! - **T14 River Call or Fold** — Villain bets into hero.  Decision depends on
//!   hero's hand strength vs the bet size: raise strong vs small bets, call
//!   marginal vs standard bets, fold weak vs large bets.

use rand::Rng;
use crate::training_engine::{
    helpers::{deal, hand_str, board_str, heads_up, scenario},
    models::*,
};

// ═══════════════════════════════════════════════════════════════════════════════
// T4 — Bluff Spot (BL-)
//
// Hero holds a busted hand on the river (no showdown value).  Villain checks.
// The correct action depends on the bluff type and SPR:
//   - CappedRange or low SPR (<2.0) → check (villain calls too wide)
//   - MissedFlushDraw / OvercardBrick + high SPR → bet large (~75% pot)
//
// `required_fold_frequency` computes how often villain must fold for a bluff
// to break even: bet / (pot + bet).
// ═══════════════════════════════════════════════════════════════════════════════

/// Why hero has no showdown value — drives the bluff story.
#[derive(Debug, Clone, Copy)]
enum BluffType {
    MissedFlushDraw,
    CappedRange,
    OvercardBrick,
}

impl std::fmt::Display for BluffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BluffType::MissedFlushDraw => write!(f, "missed flush draw"),
            BluffType::CappedRange     => write!(f, "capped / checked-back range"),
            BluffType::OvercardBrick   => write!(f, "bricked overcards"),
        }
    }
}

fn required_fold_frequency(bet_size: u32, pot_before_bet: u32) -> f32 {
    let denom = pot_before_bet + bet_size;
    if denom == 0 { return 0.0; }
    bet_size as f32 / denom as f32
}

pub fn generate_bluff<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 5);

    let bluff_type = match rng.gen_range(0..3) {
        0 => BluffType::MissedFlushDraw,
        1 => BluffType::CappedRange,
        _ => BluffType::OvercardBrick,
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(10..=16u32), 50u32),
        DifficultyLevel::Intermediate => (rng.gen_range(8..=24), rng.gen_range(30..=80)),
        DifficultyLevel::Advanced     => (rng.gen_range(6..=40), rng.gen_range(15..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;
    let spr   = stack as f32 / pot as f32;

    let small_bet = (pot as f32 * 0.40).round() as u32;
    let large_bet = (pot as f32 * 0.75).round() as u32;
    let shove     = stack;

    let spr_bucket = if spr < 2.0 { "LowSPR" } else { "HighSPR" };
    let branch_key = match bluff_type {
        BluffType::CappedRange     => "CappedRange".to_string(),
        BluffType::MissedFlushDraw => format!("MissedFlushDraw:{}", spr_bucket),
        BluffType::OvercardBrick   => format!("OvercardBrick:{}", spr_bucket),
    };

    let hero_pos = Position::BTN;
    let hs = hand_str(hero_hand);
    let bs = board_str(&board);

    let correct_id = match bluff_type {
        BluffType::CappedRange => "A",
        _ if spr < 2.0         => "A",
        _ if spr < 4.0         => "C",
        _                      => "C",
    };

    let fold_freq_small = required_fold_frequency(small_bet, pot);
    let fold_freq_large = required_fold_frequency(large_bet, pot);
    let fold_freq_shove = required_fold_frequency(shove, pot);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Last card. You have {hs} and missed — your hand can't win at showdown. \
             Board: {bs}. Pot: {pot} chips. Stack: {stack} chips. \
             Your opponent checks to you. Options: check, bet small ({small_bet} chips), \
             bet big ({large_bet} chips), go all-in ({shove} chips). What do you do?"
        ),
        TextStyle::Technical => format!(
            "River spot. You hold {hs} ({bluff_type}) on {hero_pos}. \
             Board: {bs}. Pot: {pot} chips ({pot_bb} BB). \
             Stack: {stack} chips (SPR = {spr:.1}). Villain checks to you. \
             Bet options: small ({small_bet} chips ~40% pot), large ({large_bet} chips ~75% pot), \
             or shove ({shove} chips). What do you do?"
        ),
    };

    let check_body = if correct_id == "A" {
        format!(
            "Correct. With SPR = {spr:.1} and a {bluff_type}, \
             villain's calling range is too wide to generate sufficient fold equity. \
             Bluffing here would require villain to fold >{:.0}% of the time \
             (for a large bet), which is unrealistic.",
            fold_freq_large * 100.0
        )
    } else {
        format!(
            "Checking surrenders value. With a {bluff_type} and SPR = {spr:.1}, \
             you have no showdown value and checking guarantees a loss. \
             A well-sized bluff can generate positive EV."
        )
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct_id == "A",
            explanation: match text_style {
                TextStyle::Simple => if correct_id == "A" {
                    "Correct — check. Your opponent will call any bet you make here, so betting loses more chips than checking.".to_string()
                } else {
                    "Checking gives up — you have no chance to win at showdown, so a bet is your only way to take this pot.".to_string()
                },
                TextStyle::Technical => format!("Checking with a {bluff_type} from {hero_pos}: {check_body}"),
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Bet small".to_string(),
            is_correct: correct_id == "B",
            explanation: match text_style {
                TextStyle::Simple => if correct_id == "B" {
                    "A small bet works here — it puts just enough pressure on your opponent to fold weak hands.".to_string()
                } else {
                    "A small bet won't scare your opponent into folding. Either bet big enough to be threatening or just check.".to_string()
                },
                TextStyle::Technical => format!(
                    "Small bluff ({small_bet} chips) with {hs} ({bluff_type}): \
                     Requires villain to fold {:.1}% of the time to break even. \
                     {}",
                    fold_freq_small * 100.0,
                    if correct_id == "B" {
                        "A small bet size is appropriate here — it achieves fold equity at \
                         minimal risk and keeps you unexploitable."
                    } else {
                        "A small bluff is unlikely to fold out strong hands. Either check \
                         or bet large enough to credibly represent your value range."
                    }
                ),
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: "Bet large".to_string(),
            is_correct: correct_id == "C",
            explanation: match text_style {
                TextStyle::Simple => if correct_id == "C" {
                    "Correct — bet big! You have nothing, so your only way to win is to make your opponent fold. A big bet is the most believable and gives you the best chance they give up.".to_string()
                } else {
                    "A big bet here throws too many chips away — your opponent isn't folding. Check instead.".to_string()
                },
                TextStyle::Technical => {
                    let rationale = if correct_id == "C" {
                        format!(
                            "A 75% pot bluff applies significant pressure and is credible with a \
                             {bluff_type}. Villain must fold a realistic portion of their range, \
                             and blockers in your hand make their strong hands less likely."
                        )
                    } else {
                        "A large bluff here over-commits with no fold equity. At this SPR, \
                         villain will call too frequently for this sizing to be profitable."
                            .to_string()
                    };
                    format!(
                        "Large bluff ({large_bet} chips) with {hs} ({bluff_type}): \
                         Requires villain to fold {:.1}% of the time to break even. \
                         SPR = {spr:.1}. {rationale}",
                        fold_freq_large * 100.0,
                    )
                },
            },
        },
        AnswerOption {
            id: "D".to_string(),
            text: "All-in".to_string(),
            is_correct: false,
            explanation: match text_style {
                TextStyle::Simple => "Going all-in here is too extreme. Unless you have almost no chips left compared to the pot, a well-sized big bet does the same job at lower risk.".to_string(),
                TextStyle::Technical => format!(
                    "Shoving {shove} chips with {hs} ({bluff_type}): \
                     Requires villain to fold {:.1}% of the time. \
                     A pot-sized or overbet shove can be valid with a polarized range and \
                     nut blockers, but is generally too large here unless SPR < 1.5 \
                     and villain's range is very capped.",
                    fold_freq_shove * 100.0
                ),
            },
        },
    ];

    let players = heads_up(hero_pos, Position::BB, stack, stack);
    scenario(scenario_id, TrainingTopic::BluffSpot, branch_key,
        GameType::CashGame, hero_pos, hero_hand, board, players, pot, 0, question, answers)
}

// ═══════════════════════════════════════════════════════════════════════════════
// T10 — River Value Bet (RV-)
//
// Hero is on BTN with a made hand; villain checks the river.  Sizing:
//   - Nuts (top set / straight / flush)     → overbet (~125% pot)
//   - Strong (top two pair / second set)    → large bet (~75% pot)
//   - Medium (one pair / weak two pair)     → check (thin-value trap)
//
// This topic has 4 answer options (A-D) unlike the standard 3.
// ═══════════════════════════════════════════════════════════════════════════════

/// How strong hero's made hand is — drives value bet sizing.
#[derive(Debug, Clone, Copy)]
enum ValueStrength {
    Nuts,
    Strong,
    Medium,
}

impl std::fmt::Display for ValueStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueStrength::Nuts   => write!(f, "nutted hand (top set / straight / flush)"),
            ValueStrength::Strong => write!(f, "strong hand (top two pair / second set)"),
            ValueStrength::Medium => write!(f, "medium hand (one pair / weak two pair)"),
        }
    }
}

fn value_strength_simple(hs: ValueStrength) -> &'static str {
    match hs {
        ValueStrength::Nuts   => "very strong hand",
        ValueStrength::Strong => "strong hand",
        ValueStrength::Medium => "medium hand",
    }
}

pub fn generate_value_bet<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 5);

    let strength = match rng.gen_range(0..3) {
        0 => ValueStrength::Nuts,
        1 => ValueStrength::Strong,
        _ => ValueStrength::Medium,
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(10..=18u32), 60u32),
        DifficultyLevel::Intermediate => (rng.gen_range(8..=28), rng.gen_range(30..=80)),
        DifficultyLevel::Advanced     => (rng.gen_range(6..=40), rng.gen_range(15..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let small_bet = (pot as f32 * 0.33).round() as u32;
    let large_bet = (pot as f32 * 0.75).round() as u32;
    let overbet   = (pot as f32 * 1.25).round() as u32;

    let correct: &str = match strength {
        ValueStrength::Nuts   => "D",
        ValueStrength::Strong => "C",
        ValueStrength::Medium => "A",
    };

    let branch_key = match strength {
        ValueStrength::Nuts   => "Nuts:Overbet",
        ValueStrength::Strong => "Strong:LargeBet",
        ValueStrength::Medium => "Medium:Check",
    };

    let hero_pos = Position::BTN;
    let hs = hand_str(hero_hand);
    let bs = board_str(&board);
    let strength_simple = value_strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Last card. You have {hs} (a {strength_simple}) on the Button. \
             Board: {bs}. Pot: {pot} chips. Your opponent checked to you. \
             Options: check, bet small ({small_bet} chips), bet big ({large_bet} chips), overbet ({overbet} chips). What do you do?"
        ),
        TextStyle::Technical => format!(
            "River spot. You hold {hs} ({strength}) on Button. \
             Board: {bs}. Pot: {pot} chips ({pot_bb} BB). \
             Stack: {stack} chips. Villain checks to you. \
             Bet options: small ({small_bet} chips ~33%), \
             large ({large_bet} chips ~75%), overbet ({overbet} chips ~125%). \
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
                    ValueStrength::Medium => "Correct — check. Your hand is decent but not dominant. Betting risks giving your opponent a reason to raise and win a big pot.".to_string(),
                    _ => "Checking here loses value — you have a strong hand and your opponent will likely call a bet. Bet!".to_string(),
                },
                TextStyle::Technical => match strength {
                    ValueStrength::Medium => format!(
                        "Correct. Checking back with a {strength} on the river controls the pot. \
                         Betting risks getting check-raised by better hands (two pair, sets) and \
                         called only by hands that beat you — a classic thin-value trap. \
                         Take the free showdown."
                    ),
                    _ => format!(
                        "Checking with a {strength} surrenders significant value. Villain will \
                         rarely bet into you on the river with hands that can pay off a bet. \
                         Always bet for value when you have a strong made hand and villain checks."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Bet small ({small_bet} chips ~33%)"),
            is_correct: false,
            explanation: match text_style {
                TextStyle::Simple => "Betting too small here leaves money behind. Your hand is strong — bet bigger to win more.".to_string(),
                TextStyle::Technical => format!(
                    "A 33% pot bet with a {strength} undersizes the value. Villain's calling range \
                     is capped by the river action — they will call a larger bet just as often with \
                     hands that beat you, and fold the same weak hands. Size up to extract more EV."
                ),
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Bet large ({large_bet} chips ~75%)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    ValueStrength::Strong => "Correct — bet big! You have a strong hand and your opponent is likely to call. Get paid as much as possible.".to_string(),
                    ValueStrength::Nuts => "Going overboard on the bet size risks your opponent folding a hand that would have called a normal big bet.".to_string(),
                    ValueStrength::Medium => "Betting big here is risky when your hand isn't quite strong enough for it.".to_string(),
                },
                TextStyle::Technical => match strength {
                    ValueStrength::Strong => format!(
                        "Correct. A 75% pot value bet with a {strength} is optimal. It maximises \
                         value from villain's weaker made hands (top pair, second pair) while \
                         remaining credible — not so large that villain folds everything that \
                         can call. This is the standard value sizing on the river."
                    ),
                    ValueStrength::Nuts => format!(
                        "A 75% pot bet is good but leaves value on the table with a {strength}. \
                         Consider an overbet — your hand can credibly represent a polarised value \
                         range and villain must call off a large portion of their stack."
                    ),
                    ValueStrength::Medium => format!(
                        "Betting 75% pot with a {strength} is a risky thin value bet. You risk \
                         being called by better hands and raised off a marginal holding. \
                         Check is higher EV here."
                    ),
                },
            },
        },
        AnswerOption {
            id: "D".to_string(),
            text: format!("Overbet ({overbet} chips ~125%)"),
            is_correct: correct == "D",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    ValueStrength::Nuts => "Correct — go big! You have the strongest possible hand here. Bet as much as you can — your opponent will likely call.".to_string(),
                    _ => "Going overboard on the bet size risks your opponent folding a hand that would have called a normal big bet.".to_string(),
                },
                TextStyle::Technical => match strength {
                    ValueStrength::Nuts => format!(
                        "Correct. An overbet with a {strength} is the highest-EV play. Your hand is \
                         at the top of your range — you can represent a polarised range that includes \
                         both bluffs and the nuts. Villain cannot fold their strong hands here, and \
                         weak hands that would call 75% will also call 125%. Maximise the pot."
                    ),
                    _ => format!(
                        "Overbetting with a {strength} is too ambitious. An overbet signals a \
                         polarised range (nuts or bluff) — villain will call with better hands \
                         and fold hands you dominate. Use 75% pot sizing instead."
                    ),
                },
            },
        },
    ];

    let players = heads_up(hero_pos, Position::BB, stack, stack);
    scenario(scenario_id, TrainingTopic::RiverValueBet, branch_key,
        GameType::CashGame, hero_pos, hero_hand, board, players, pot, 0, question, answers)
}

// ═══════════════════════════════════════════════════════════════════════════════
// T14 — River Call or Fold (RF-)
//
// Villain bets into hero on the river.  Hero must decide: fold, call, or raise.
// The correct action is a function of (hand strength, bet size):
//   - Strong hand + small bet  → raise (extract value)
//   - Marginal    + standard   → call  (pot odds justify it)
//   - Weak        + large      → fold  (not enough equity)
// ═══════════════════════════════════════════════════════════════════════════════

/// Hero's hand strength when facing a river bet.
#[derive(Debug, Clone, Copy)]
enum CallerStrength {
    Strong,
    Marginal,
    Weak,
}

#[derive(Debug, Clone, Copy)]
enum BetSize {
    Small,
    Standard,
    Large,
}

impl std::fmt::Display for CallerStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallerStrength::Strong   => write!(f, "strong hand (two pair+ / top pair strong kicker)"),
            CallerStrength::Marginal => write!(f, "marginal hand (top pair weak kicker / middle pair)"),
            CallerStrength::Weak     => write!(f, "weak hand (bottom pair / missed draw)"),
        }
    }
}

impl std::fmt::Display for BetSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BetSize::Small    => write!(f, "small (~33%)"),
            BetSize::Standard => write!(f, "standard (~67%)"),
            BetSize::Large    => write!(f, "large (~pot)"),
        }
    }
}

fn caller_strength_simple(cs: CallerStrength) -> &'static str {
    match cs {
        CallerStrength::Strong   => "strong hand",
        CallerStrength::Marginal => "medium hand",
        CallerStrength::Weak     => "weak hand",
    }
}

fn bet_size_simple(bs: BetSize) -> &'static str {
    match bs {
        BetSize::Small    => "small bet",
        BetSize::Standard => "normal-sized bet",
        BetSize::Large    => "large bet",
    }
}

pub fn generate_call_or_fold<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let (hero_hand, board) = deal(rng, 5);

    let (strength, bet_size) = match rng.gen_range(0..3) {
        0 => (CallerStrength::Strong,   BetSize::Small),
        1 => (CallerStrength::Marginal, BetSize::Standard),
        _ => (CallerStrength::Weak,     BetSize::Large),
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(10..=20u32), 80u32),
        DifficultyLevel::Intermediate => (rng.gen_range(8..=28),     rng.gen_range(30..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(6..=40),     rng.gen_range(15..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let villain_bet = match bet_size {
        BetSize::Small    => (pot as f32 * 0.33).round() as u32,
        BetSize::Standard => (pot as f32 * 0.67).round() as u32,
        BetSize::Large    => pot,
    };

    let required_equity_pct =
        (villain_bet as f32 / (pot as f32 + villain_bet as f32 * 2.0) * 100.0).round() as u32;
    let raise_size = (villain_bet as f32 * 2.5).round() as u32;

    let correct: &str = match (strength, bet_size) {
        (CallerStrength::Strong,   BetSize::Small)    => "C",
        (CallerStrength::Marginal, BetSize::Standard) => "B",
        (CallerStrength::Weak,     BetSize::Large)    => "A",
        _                                              => "A",
    };

    let branch_key = match (strength, bet_size) {
        (CallerStrength::Strong,   BetSize::Small)    => "Strong:SmallBet:Raise",
        (CallerStrength::Marginal, BetSize::Standard) => "Marginal:StdBet:Call",
        (CallerStrength::Weak,     BetSize::Large)    => "Weak:LargeBet:Fold",
        _                                              => "Unknown",
    };

    let hero_pos = Position::BTN;
    let hs = hand_str(hero_hand);
    let bs = board_str(&board);
    let strength_simple = caller_strength_simple(strength);
    let bet_size_simple_label = bet_size_simple(bet_size);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Last card. You have {hs} ({strength_simple}) on the Button. \
             Board: {bs}. Pot: {pot} chips. Stack: {stack} chips. \
             Your opponent bets {villain_bet} chips ({bet_size_simple_label}) into you. What do you do?"
        ),
        TextStyle::Technical => format!(
            "River call or fold. You hold {hs} ({strength}) on the Button. \
             Board: {bs}. Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             Villain bets {villain_bet} chips ({bet_size}) into you. \
             You need ~{required_equity_pct}% equity to break even on a call. \
             What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match (strength, bet_size) {
                    (CallerStrength::Weak, BetSize::Large) =>
                        "Correct — fold. Your hand is weak and your opponent made a large bet. You don't win often enough here to make calling worth it.".to_string(),
                    _ =>
                        "Folding here gives up too easily — you have enough of a hand to call.".to_string(),
                },
                TextStyle::Technical => match (strength, bet_size) {
                    (CallerStrength::Weak, BetSize::Large) => format!(
                        "Correct. Folding a {strength} against a {bet_size} bet is right. \
                         You need ~{required_equity_pct}% equity to break even, but a {strength} \
                         is unlikely to have that against a polarised river betting range. \
                         Villain's large bet signals a strong hand or bluff — your weak hand \
                         loses to the former and gains nothing against the latter. Fold."
                    ),
                    _ => format!(
                        "Folding here surrenders too much value. Against a {bet_size} bet you \
                         need only ~{required_equity_pct}% equity — your {strength} exceeds that. \
                         Either call to realise your equity, or raise to extract more value."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({villain_bet} chips)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match (strength, bet_size) {
                    (CallerStrength::Marginal, BetSize::Standard) =>
                        "Correct — call. Your hand wins often enough at this price to make calling worthwhile.".to_string(),
                    (CallerStrength::Strong, BetSize::Small) =>
                        "Just calling here misses a chance to win more — raise with this strong hand!".to_string(),
                    _ =>
                        "Just calling here misses a chance to win more — raise with this strong hand!".to_string(),
                },
                TextStyle::Technical => match (strength, bet_size) {
                    (CallerStrength::Marginal, BetSize::Standard) => format!(
                        "Correct. Calling {villain_bet} chips against a {bet_size} bet with a \
                         {strength} is the right play. You need ~{required_equity_pct}% equity \
                         and your hand is likely ahead of villain's bluffing frequency at this \
                         sizing. Folding is too tight; raising turns a thin call into an \
                         aggressive bluff-raise that few worse hands will call."
                    ),
                    (CallerStrength::Strong, BetSize::Small) => format!(
                        "Calling with a {strength} against a {bet_size} bet is fine but leaves \
                         value behind. Villain is likely betting thin for value with hands you \
                         beat — a raise to ~{raise_size} chips extracts more EV and is credible \
                         given your strong range on the river."
                    ),
                    _ => format!(
                        "Calling {villain_bet} chips with a {strength} against a {bet_size} bet \
                         is -EV. You need ~{required_equity_pct}% equity and your hand is unlikely \
                         to have it against a polarised river bet at this sizing. Fold."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {raise_size} chips"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match (strength, bet_size) {
                    (CallerStrength::Strong, BetSize::Small) =>
                        "Correct — raise! Your opponent made a small bet and you have a strong hand. Raise to win more chips — they're likely to call.".to_string(),
                    _ =>
                        "Raising here is too aggressive for your hand strength. Just call or fold.".to_string(),
                },
                TextStyle::Technical => match (strength, bet_size) {
                    (CallerStrength::Strong, BetSize::Small) => format!(
                        "Correct. Raising to ~{raise_size} chips with a {strength} against a \
                         {bet_size} villain bet maximises value. A small river bet from villain \
                         often represents a thin value bet or a small bluff — your strong hand \
                         is ahead of much of that range. A raise to ~2.5× the bet is credible \
                         and extracts significantly more EV than a flat call. Villain will call \
                         with weaker top pairs and strong one-pair hands."
                    ),
                    _ => format!(
                        "Raising with a {strength} against a {bet_size} bet is too aggressive. \
                         A raise commits a large portion of the stack with a hand that cannot \
                         profitably call many re-raises. Only raise on the river when your hand \
                         is strong enough to comfortably stack off."
                    ),
                },
            },
        },
    ];

    let players = heads_up(hero_pos, Position::BB, stack, stack);
    scenario(scenario_id, TrainingTopic::RiverCallOrFold, branch_key,
        GameType::CashGame, hero_pos, hero_hand, board, players, pot, villain_bet, question, answers)
}
