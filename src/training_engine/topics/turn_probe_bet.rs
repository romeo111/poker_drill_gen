use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

/// Hero's hand strength on the turn in an OOP probe spot.
#[derive(Debug, Clone, Copy)]
enum ProbeStrength {
    Strong, // Top pair+, strong draw — probe large to build pot and charge draws
    Medium, // Middle pair, weak draw — probe small as semi-bluff or thin value
    Weak,   // Bottom pair, air — check; no equity to justify betting OOP
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

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(4); // three flop cards + turn card

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

    // Correct action:
    // Strong → Probe large (~70%): build pot, charge draws, collect value from one-pair hands.
    // Medium → Probe small (~40%): semi-bluff or thin value at low risk; fold-able if raised.
    // Weak   → Check: no equity; probing into Button (IP) who checked back risks a raise
    //          with no equity to fall back on.
    let correct: &str = match strength {
        ProbeStrength::Strong => "C",
        ProbeStrength::Medium => "B",
        ProbeStrength::Weak   => "A",
    };

    let branch_key = match strength {
        ProbeStrength::Strong => "Strong:ProbeLarge",
        ProbeStrength::Medium => "Medium:ProbeSmall",
        ProbeStrength::Weak   => "Weak:Check",
    }.to_string();

    let hero_pos  = Position::BB;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");

    let strength_simple = probe_strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Both players checked after the first three cards. Fourth card: {board_str}. \
             You have {hand_str} ({strength_simple}) in the Big Blind (you act first). \
             Pot: {pot} chips. Stack: {stack} chips. \
             Options: check, bet small ({small_probe} chips), bet big ({large_probe} chips). What do you do?"
        ),
        TextStyle::Technical => format!(
            "Turn probe spot. You hold {hand_str} ({strength}) in the Big Blind (OOP). \
             The flop was checked through by both players. Board (flop + turn): {board_str}. \
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
                    ProbeStrength::Weak => format!(
                        "Correct — check. Your hand is weak and your opponent didn't bet on the flop — no reason to bet now."
                    ),
                    _ => format!(
                        "Checking here misses an opportunity. Your hand is strong enough to bet and take the pot."
                    ),
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
                    ProbeStrength::Medium => format!(
                        "Correct — bet small. Your hand is decent but not great. A small bet tests the water and may win the pot without risking too much."
                    ),
                    ProbeStrength::Strong => format!(
                        "A small bet doesn't do enough here — bet bigger to put real pressure on, or just check."
                    ),
                    ProbeStrength::Weak => format!(
                        "A small bet doesn't do enough here — bet bigger to put real pressure on, or just check."
                    ),
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
                    ProbeStrength::Strong => format!(
                        "Correct — bet big! You have a strong hand and the Button didn't bet after the flop (a sign of weakness). Take the pot now with a big bet."
                    ),
                    ProbeStrength::Medium => format!(
                        "Betting big here is too aggressive for your hand strength. Bet small or check."
                    ),
                    ProbeStrength::Weak => format!(
                        "Betting big here is too aggressive for your hand strength. Bet small or check."
                    ),
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

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::TurnProbeBet,
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
