use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{
        combo_draw_equity, flush_draw_equity, has_flush_draw, has_straight_draw,
        oesd_equity, required_equity,
    },
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

#[derive(Debug, Clone, Copy)]
enum DrawType {
    FlushDraw,
    OpenEndedStraight,
    ComboDraw,
    GutShot,
}

impl std::fmt::Display for DrawType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawType::FlushDraw          => write!(f, "flush draw"),
            DrawType::OpenEndedStraight  => write!(f, "open-ended straight draw"),
            DrawType::ComboDraw          => write!(f, "combo draw (flush + straight draw)"),
            DrawType::GutShot            => write!(f, "gutshot straight draw"),
        }
    }
}

fn draw_type_simple(dt: DrawType) -> &'static str {
    match dt {
        DrawType::FlushDraw         => "flush draw (you need one more card of the same suit to make a flush)",
        DrawType::OpenEndedStraight => "straight draw (you can complete a straight on either end)",
        DrawType::ComboDraw         => "two-way draw (flush or straight possible)",
        DrawType::GutShot           => "inside straight draw (only one card completes your straight)",
    }
}

fn hero_equity(draw: DrawType, streets: u8) -> f32 {
    match draw {
        DrawType::FlushDraw         => flush_draw_equity(streets),
        DrawType::OpenEndedStraight => oesd_equity(streets),
        DrawType::ComboDraw         => combo_draw_equity(streets),
        DrawType::GutShot           => if streets == 2 { 0.17 } else { 0.09 },
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
    let board: Vec<Card> = deck.deal_n(3);

    // Determine draw type from the actual board (best effort) or assign randomly
    let flush = has_flush_draw(&board);
    let straight = has_straight_draw(&board);
    let draw_type = match (flush, straight) {
        (true, true)  => DrawType::ComboDraw,
        (true, false) => DrawType::FlushDraw,
        (false, true) => DrawType::OpenEndedStraight,
        _             => DrawType::GutShot,
    };

    let bb = 2u32;
    let (pot_bb, bet_pct) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(8..=12u32), 0.50f32),
        DifficultyLevel::Intermediate => (rng.gen_range(6..=20), rng.gen_range_f32(0.33..=1.0)),
        DifficultyLevel::Advanced     => (rng.gen_range(4..=30), rng.gen_range_f32(0.25..=1.5)),
    };
    let pot = pot_bb * bb;
    let bet = (pot as f32 * bet_pct).round() as u32;
    let streets_remaining: u8 = 2; // flop scenario, two streets to come

    let req_eq = required_equity(bet, pot);
    let actual_eq = hero_equity(draw_type, streets_remaining);
    let should_call = actual_eq >= req_eq;

    let draw_name = match draw_type {
        DrawType::FlushDraw         => "FlushDraw",
        DrawType::OpenEndedStraight => "OESD",
        DrawType::ComboDraw         => "ComboDraw",
        DrawType::GutShot           => "GutShot",
    };
    let branch_key = format!("{}:{}", draw_name, if should_call { "Call" } else { "Fold" });

    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let hero_pos = Position::BB;

    let draw_type_label = format!("{}", draw_type);
    let draw_type_simple_label = draw_type_simple(draw_type);

    let question = match text_style {
        TextStyle::Simple => format!(
            "You have {hand_str} and are chasing a {draw_type_simple_label} after the first three cards: {board_str}. \
             Pot: {pot} chips. Your opponent bet {bet} chips. Do you call or fold?"
        ),
        TextStyle::Technical => format!(
            "You hold {hand_str} and have a {draw_type_label} on the flop {board_str}. \
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

// Helper: generate a random f32 in a range via Rng
trait RngF32Ext {
    fn gen_range_f32(&mut self, range: std::ops::RangeInclusive<f32>) -> f32;
}
impl<R: Rng> RngF32Ext for R {
    fn gen_range_f32(&mut self, range: std::ops::RangeInclusive<f32>) -> f32 {
        let lo = *range.start();
        let hi = *range.end();
        lo + self.gen::<f32>() * (hi - lo)
    }
}
