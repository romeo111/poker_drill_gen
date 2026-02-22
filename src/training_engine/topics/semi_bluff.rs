use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{has_flush_draw, has_straight_draw},
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawType {
    ComboDraw,
    FlushDraw,
    OESD,
    GutShot,
}

impl std::fmt::Display for DrawType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawType::ComboDraw => write!(f, "combo draw (flush + straight)"),
            DrawType::FlushDraw => write!(f, "flush draw"),
            DrawType::OESD      => write!(f, "open-ended straight draw"),
            DrawType::GutShot   => write!(f, "gutshot straight draw"),
        }
    }
}

fn draw_equity_flop(dt: DrawType) -> f32 {
    match dt {
        DrawType::ComboDraw => 0.54,
        DrawType::FlushDraw => 0.35,
        DrawType::OESD      => 0.32,
        DrawType::GutShot   => 0.17,
    }
}

fn classify_draw(board: &[Card]) -> DrawType {
    let flush = has_flush_draw(board);
    let straight = has_straight_draw(board);
    match (flush, straight) {
        (true, true)  => DrawType::ComboDraw,
        (true, false) => DrawType::FlushDraw,
        (false, true) => DrawType::OESD,
        _             => DrawType::GutShot,
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

    let draw_type = classify_draw(&board);

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (60u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(35..=120), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    // Villain bets 50–75% pot
    let villain_bet_pct: u32 = rng.gen_range(50..=75);
    let villain_bet = (pot * villain_bet_pct / 100).max(bb);
    let raise_size = villain_bet * 5 / 2; // 2.5× semi-bluff raise

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

    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let pos_str = format!("{}", hero_pos);
    let equity = draw_equity_flop(draw_type);
    let position_label = if hero_is_ip { "in position" } else { "out of position" };

    // Correct answer (single ID):
    // ComboDraw         → "C" (Raise — near-favourite, maximise pressure)
    // FlushDraw + IP    → "B" (Call — realise equity in position)
    // OESD + stack ≥ 40 → "C" (Raise — fold equity + semi-bluff)
    // GutShot           → "A" (Fold — insufficient equity)
    // FlushDraw + OOP   → "B" (Call — can't raise without positional advantage)
    let correct: &str = match draw_type {
        DrawType::ComboDraw                         => "C",
        DrawType::OESD if stack_bb >= 40            => "C",
        DrawType::FlushDraw | DrawType::OESD        => "B",
        DrawType::GutShot                           => "A",
    };

    let question = format!(
        "You hold {hand_str} and have a {draw_type} on the flop {board_str}. \
         You are {position_label} ({pos_str}, {stack_bb} BB deep). \
         Villain bets {villain_bet} chips ({villain_bet_pct}% pot). \
         Pot is {pot} chips ({pot_bb} BB). \
         Your {draw_type} has ~{:.0}% equity. What do you do?",
        equity * 100.0
    );

    // --- Explanations ---

    let fold_exp = if matches!(draw_type, DrawType::GutShot) {
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
            "Folding a {draw_type} (~{:.0}% equity) is too tight here. You have enough \
             equity to continue — either by calling to realise it, or raising as a \
             semi-bluff when conditions are right.",
            equity * 100.0
        )
    };

    let call_exp = match (draw_type, hero_is_ip, correct) {
        (DrawType::FlushDraw, true, "B") => format!(
            "Correct. Calling with a {draw_type} (~{:.0}% equity) from {pos_str} (IP) is \
             the best play. You have position to control the pot on future streets — check \
             back or bet when you hit, give up cheaply when you miss. Raising risks bloating \
             the pot without the positional advantage needed to navigate it well.",
            equity * 100.0
        ),
        (DrawType::FlushDraw, false, "B") | (DrawType::OESD, _, "B") => format!(
            "Correct. Calling with a {draw_type} (~{:.0}% equity) {position_label} is correct \
             here. Your stack depth ({stack_bb} BB) and/or position make a semi-bluff raise \
             suboptimal — calling lets you realise equity without bloating the pot OOP or \
             risking a re-raise at shallow depth.",
            equity * 100.0
        ),
        _ => format!(
            "Calling is an option but not the highest-EV line here. With a {draw_type} \
             (~{:.0}% equity) {position_label}, a semi-bluff raise to {raise_size} chips \
             adds fold equity on top of your draw equity, making raising more profitable.",
            equity * 100.0
        ),
    };

    let raise_exp = match (draw_type, hero_is_ip, correct) {
        (DrawType::ComboDraw, _, "C") => format!(
            "Correct. Raising to {raise_size} chips (2.5× villain's {villain_bet}) with a \
             {draw_type} on {board_str} is the highest-EV play. Your combo draw has ~54% \
             equity — you are a slight favourite! Raising wins the pot outright when villain \
             folds (~40% of the time) and builds a large pot when villain calls into your \
             equity edge. Never just call with a combo draw when you can apply maximum pressure."
        ),
        (DrawType::OESD, _, "C") => format!(
            "Correct. Raising to {raise_size} chips (2.5× villain's {villain_bet}) with an \
             {draw_type} at {stack_bb} BB depth is correct. Your OESD has ~32% equity plus \
             significant fold equity: villain must fold hands like top pair to avoid getting \
             stacked. At {stack_bb} BB the semi-bluff raise sets up a profitable shove on \
             the turn or a clean check when you miss.",
        ),
        _ => format!(
            "Raising to {raise_size} chips as a semi-bluff with a {draw_type} \
             {position_label} is too aggressive here. You risk building a large pot \
             without sufficient equity to back it up. Calling is the stronger line."
        ),
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
            text: "Call (passive)".to_string(),
            is_correct: correct == "B",
            explanation: call_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {} chips (semi-bluff, 2.5× bet)", raise_size),
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
