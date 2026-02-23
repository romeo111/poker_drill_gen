use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

/// How strong the hero's made hand is on the river.
#[derive(Debug, Clone, Copy)]
enum HandStrength {
    Nuts,       // Full house, flush, straight, or top set
    Strong,     // Top two pair, second set, strong one pair
    Medium,     // Middle or bottom pair, weak two pair
}

impl std::fmt::Display for HandStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandStrength::Nuts   => write!(f, "nutted hand (top set / straight / flush)"),
            HandStrength::Strong => write!(f, "strong hand (top two pair / second set)"),
            HandStrength::Medium => write!(f, "medium hand (one pair / weak two pair)"),
        }
    }
}

fn hand_strength_simple(hs: HandStrength) -> &'static str {
    match hs {
        HandStrength::Nuts   => "very strong hand",
        HandStrength::Strong => "strong hand",
        HandStrength::Medium => "medium hand",
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
    let board: Vec<Card> = deck.deal_n(5);

    let strength = match rng.gen_range(0..3) {
        0 => HandStrength::Nuts,
        1 => HandStrength::Strong,
        _ => HandStrength::Medium,
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

    // Correct action:
    // Nuts   → overbet (polarised, maximises value against wide calling range)
    // Strong → large bet (75% pot extracts value, credible sizing)
    // Medium → check (pot control; thin value bets often get called by better hands)
    let correct: &str = match strength {
        HandStrength::Nuts   => "D",
        HandStrength::Strong => "C",
        HandStrength::Medium => "A",
    };

    let branch_key = match strength {
        HandStrength::Nuts   => "Nuts:Overbet",
        HandStrength::Strong => "Strong:LargeBet",
        HandStrength::Medium => "Medium:Check",
    }.to_string();

    let hero_pos = Position::BTN;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let strength_simple = hand_strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Last card. You have {hand_str} (a {strength_simple}) on the Button. \
             Board: {board_str}. Pot: {pot} chips. Your opponent checked to you. \
             Options: check, bet small ({small_bet} chips), bet big ({large_bet} chips), overbet ({overbet} chips). What do you do?"
        ),
        TextStyle::Technical => format!(
            "River spot. You hold {hand_str} ({strength}) on Button. \
             Board: {board_str}. Pot: {pot} chips ({pot_bb} BB). \
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
                    HandStrength::Medium => format!(
                        "Correct — check. Your hand is decent but not dominant. Betting risks giving your opponent a reason to raise and win a big pot."
                    ),
                    _ => format!(
                        "Checking here loses value — you have a strong hand and your opponent will likely call a bet. Bet!"
                    ),
                },
                TextStyle::Technical => match strength {
                    HandStrength::Medium => format!(
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
                TextStyle::Simple => match strength {
                    HandStrength::Medium => format!(
                        "Betting too small here leaves money behind. Your hand is strong — bet bigger to win more."
                    ),
                    _ => format!(
                        "Betting too small here leaves money behind. Your hand is strong — bet bigger to win more."
                    ),
                },
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
                    HandStrength::Strong => format!(
                        "Correct — bet big! You have a strong hand and your opponent is likely to call. Get paid as much as possible."
                    ),
                    HandStrength::Nuts => format!(
                        "Going overboard on the bet size risks your opponent folding a hand that would have called a normal big bet."
                    ),
                    HandStrength::Medium => format!(
                        "Betting big here is risky when your hand isn't quite strong enough for it."
                    ),
                },
                TextStyle::Technical => match strength {
                    HandStrength::Strong => format!(
                        "Correct. A 75% pot value bet with a {strength} is optimal. It maximises \
                         value from villain's weaker made hands (top pair, second pair) while \
                         remaining credible — not so large that villain folds everything that \
                         can call. This is the standard value sizing on the river."
                    ),
                    HandStrength::Nuts => format!(
                        "A 75% pot bet is good but leaves value on the table with a {strength}. \
                         Consider an overbet — your hand can credibly represent a polarised value \
                         range and villain must call off a large portion of their stack."
                    ),
                    HandStrength::Medium => format!(
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
                    HandStrength::Nuts => format!(
                        "Correct — go big! You have the strongest possible hand here. Bet as much as you can — your opponent will likely call."
                    ),
                    _ => format!(
                        "Going overboard on the bet size risks your opponent folding a hand that would have called a normal big bet."
                    ),
                },
                TextStyle::Technical => match strength {
                    HandStrength::Nuts => format!(
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

    let players = vec![
        PlayerState { seat: 1, position: Position::BB, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos, stack, is_hero: true, is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::RiverValueBet,
        branch_key,
        table_setup: TableSetup {
            game_type: GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board,
            players,
            pot_size: pot,
            current_bet: 0,
        },
        question,
        answers,
    }
}
