use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

/// Which blocker scenario the hero is in.
#[derive(Debug, Clone, Copy)]
enum BluffType {
    MissedFlushDraw,   // Hero has a busted draw — no showdown value
    CappedRange,       // Hero checked the turn and can't represent the nuts
    OvercardBrick,     // Hero has two overcards that bricked
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
    // To break even: fold_freq * pot >= (1-fold_freq) * bet
    // fold_freq = bet / (pot + bet)
    let denom = pot_before_bet + bet_size;
    if denom == 0 { return 0.0; }
    bet_size as f32 / denom as f32
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    // River: 5 board cards
    let board: Vec<Card> = deck.deal_n(5);

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
    let pot    = pot_bb * bb;
    let stack  = stack_bb * bb;
    let spr    = stack as f32 / pot as f32;

    let small_bet  = (pot as f32 * 0.40).round() as u32;
    let large_bet  = (pot as f32 * 0.75).round() as u32;
    let shove      = stack.min(stack); // all-in

    let spr_bucket = if spr < 2.0 { "LowSPR" } else { "HighSPR" };
    let branch_key = match bluff_type {
        BluffType::CappedRange     => "CappedRange".to_string(),
        BluffType::MissedFlushDraw => format!("MissedFlushDraw:{}", spr_bucket),
        BluffType::OvercardBrick   => format!("OvercardBrick:{}", spr_bucket),
    };

    let hero_pos = Position::BTN;
    let pos_str = format!("{}", hero_pos);
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");

    // Determine correct action:
    // - High SPR + missed draw → large bluff (polarized range applies pressure)
    // - Low SPR → shove or check (no fold equity at low SPR)
    // - Capped range → check (can't credibly represent the nuts)
    let correct_id = match bluff_type {
        BluffType::CappedRange => "A",
        _ if spr < 2.0         => "A", // no fold equity, just check
        _ if spr < 4.0         => "C", // modest bluff
        _                      => "C", // large bluff on average
    };

    let fold_freq_small = required_fold_frequency(small_bet, pot);
    let fold_freq_large = required_fold_frequency(large_bet, pot);
    let fold_freq_shove = required_fold_frequency(shove, pot);

    let question = format!(
        "River spot. You hold {hand_str} ({bluff_type}) on {pos_str}. \
         Board: {board_str}. Pot: {pot} chips ({pot_bb} BB). \
         Stack: {stack} chips (SPR = {spr:.1}). Villain checks to you. \
         What do you do?"
    );

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
            text: "Check (give up)".to_string(),
            is_correct: correct_id == "A",
            explanation: format!("Checking with a {bluff_type} from {pos_str}: {check_body}"),
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Small bluff ({:.0}% pot = {} chips)", 40.0, small_bet),
            is_correct: correct_id == "B",
            explanation: format!(
                "Small bluff ({small_bet} chips) with {hand_str} ({bluff_type}): \
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
        AnswerOption {
            id: "C".to_string(),
            text: format!("Large bluff ({:.0}% pot = {} chips)", 75.0, large_bet),
            is_correct: correct_id == "C",
            explanation: format!(
                "Large bluff ({large_bet} chips) with {hand_str} ({bluff_type}): \
                 Requires villain to fold {:.1}% of the time to break even. \
                 SPR = {spr:.1}. {}",
                fold_freq_large * 100.0,
                if correct_id == "C" {
                    "A 75% pot bluff applies significant pressure and is credible with a \
                     {bluff_type}. Villain must fold a realistic portion of their range, \
                     and blockers in your hand make their strong hands less likely."
                } else {
                    "A large bluff here over-commits with no fold equity. At this SPR, \
                     villain will call too frequently for this sizing to be profitable."
                }
            ),
        },
        AnswerOption {
            id: "D".to_string(),
            text: format!("All-in shove ({} chips)", shove),
            is_correct: false,
            explanation: format!(
                "Shoving {shove} chips with {hand_str} ({bluff_type}): \
                 Requires villain to fold {:.1}% of the time. \
                 A pot-sized or overbet shove can be valid with a polarized range and \
                 nut blockers, but is generally too large here unless SPR < 1.5 \
                 and villain's range is very capped.",
                fold_freq_shove * 100.0
            ),
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1, position: Position::BB, stack, is_hero: false, is_active: true,
        },
        PlayerState {
            seat: 2, position: hero_pos, stack, is_hero: true, is_active: true,
        },
    ];

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
        topic: TrainingTopic::BluffSpot,
        branch_key,
        table_setup,
        question,
        answers,
    }
}
