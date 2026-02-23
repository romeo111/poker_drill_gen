use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

/// Hero's flop hand strength in a multiway pot.
#[derive(Debug, Clone, Copy)]
enum MultiStrength {
    Strong,  // Set, two pair, overpair — bet large for value and protection
    TopPair, // Top pair (good kicker) — bet small, protect without overcommitting
    Weak,    // Middle pair or worse — check; multiway equity is too vulnerable
}

impl std::fmt::Display for MultiStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiStrength::Strong  => write!(f, "strong (set / two pair / overpair)"),
            MultiStrength::TopPair => write!(f, "top pair (good kicker)"),
            MultiStrength::Weak    => write!(f, "weak (middle pair or worse)"),
        }
    }
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(3);

    let strength = match rng.gen_range(0..3) {
        0 => MultiStrength::Strong,
        1 => MultiStrength::TopPair,
        _ => MultiStrength::Weak,
    };

    // Number of opponents (multiway = 2+)
    let opponents: u8 = match difficulty {
        DifficultyLevel::Beginner     => 2,
        DifficultyLevel::Intermediate => rng.gen_range(2..=3),
        DifficultyLevel::Advanced     => rng.gen_range(2..=4),
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(8..=16u32), 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(6..=20),    rng.gen_range(50..=120)),
        DifficultyLevel::Advanced     => (rng.gen_range(4..=30),    rng.gen_range(20..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let small_bet = (pot as f32 * 0.33).round() as u32;
    let large_bet = (pot as f32 * 0.67).round() as u32;

    // Correct action in a multiway pot:
    // Strong   → Bet large (~67%): multiple draws multiply with each opponent;
    //            charge them all immediately and protect your equity.
    // TopPair  → Bet small (~33%): extract thin value while keeping the pot manageable
    //            if you encounter resistance from multiple players.
    // Weak     → Check: in multiway pots the probability of at least one opponent
    //            having a better hand increases sharply; don't bloat the pot with weak equity.
    let correct: &str = match strength {
        MultiStrength::Strong  => "C",
        MultiStrength::TopPair => "B",
        MultiStrength::Weak    => "A",
    };

    let branch_key = match strength {
        MultiStrength::Strong  => "Strong:BetLarge",
        MultiStrength::TopPair => "TopPair:BetSmall",
        MultiStrength::Weak    => "Weak:Check",
    }.to_string();

    let hero_pos  = Position::CO;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let opp_str   = if opponents == 1 {
        "1 opponent".to_string()
    } else {
        format!("{opponents} opponents")
    };

    let question = format!(
        "Multiway flop. You hold {hand_str} ({strength}) in the Cutoff. \
         Board: {board_str}. Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
         You are first to act against {opp_str}. \
         Bet options: small ({small_bet} chips ~33%), large ({large_bet} chips ~67%). \
         What do you do?"
    );

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct == "A",
            explanation: match strength {
                MultiStrength::Weak => format!(
                    "Correct. Checking a {strength} in a multiway pot is right. In multiway \
                     pots, the probability that at least one opponent has a strong hand rises \
                     significantly with each extra player. Betting a {strength} risks being \
                     called or raised by better hands with minimal protection value. Check and \
                     re-evaluate on the turn."
                ),
                _ => format!(
                    "Checking a {strength} in a multiway pot is too passive. More opponents \
                     mean more draws and speculative hands — you need to charge them to see \
                     more cards. Checking gives a free card to {opp_str} and reduces your \
                     protection against improving hands. Bet to build the pot and deny equity."
                ),
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Bet small ({small_bet} chips ~33%)"),
            is_correct: correct == "B",
            explanation: match strength {
                MultiStrength::TopPair => format!(
                    "Correct. A small bet (~33% pot) with {strength} in a multiway pot is the \
                     right approach. Top pair with a good kicker has value but is vulnerable \
                     to draws and stronger made hands. A small bet extracts thin value, applies \
                     some protection, and keeps the pot manageable if you encounter resistance \
                     from {opp_str}."
                ),
                MultiStrength::Strong => format!(
                    "A small bet with {strength} in a multiway pot undersizes the protection \
                     needed. With {opp_str} in the hand, draw combinations multiply and you \
                     need to charge them appropriately. A larger bet (~67%) protects your equity \
                     and builds a pot worth winning."
                ),
                MultiStrength::Weak => format!(
                    "Betting small with a {strength} into {opp_str} is a marginal bluff with \
                     poor equity. Multiway bluffs have lower success rates because all opponents \
                     must fold. Check and control the pot."
                ),
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Bet large ({large_bet} chips ~67%)"),
            is_correct: correct == "C",
            explanation: match strength {
                MultiStrength::Strong => format!(
                    "Correct. A large bet (~67% pot) with {strength} against {opp_str} is the \
                     highest-EV play. In multiway pots, draw combinations multiply with each \
                     opponent — sets and two pair need to charge draws immediately. A larger \
                     bet denies equity, protects your made hand, and builds a significant pot \
                     that you are heavily favoured to win."
                ),
                MultiStrength::TopPair => format!(
                    "A large bet with {strength} against {opp_str} over-commits to a \
                     vulnerable holding. Top pair faces real reverse-implied odds in multiway \
                     pots — at least one opponent may already have two pair or better. A small \
                     bet (~33%) achieves the same protection at much lower risk."
                ),
                MultiStrength::Weak => format!(
                    "Betting large with a {strength} into {opp_str} is a costly bluff. \
                     Multiway pot bluffs require all opponents to fold, which rarely happens \
                     with a large bet at the flop stage. Check instead."
                ),
            },
        },
    ];

    // Build player list: hero + opponents at various positions
    let mut players = vec![
        PlayerState { seat: 2, position: hero_pos,      stack, is_hero: true,  is_active: true },
        PlayerState { seat: 3, position: Position::BTN, stack, is_hero: false, is_active: true },
    ];
    if opponents >= 2 {
        players.push(PlayerState {
            seat: 4, position: Position::BB, stack, is_hero: false, is_active: true,
        });
    }
    if opponents >= 3 {
        players.push(PlayerState {
            seat: 5, position: Position::SB, stack, is_hero: false, is_active: true,
        });
    }
    if opponents >= 4 {
        players.push(PlayerState {
            seat: 1, position: Position::HJ, stack, is_hero: false, is_active: true,
        });
    }

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::MultiwayPot,
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
