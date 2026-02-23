use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

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

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(3);

    let texture  = if rng.gen_bool(0.5) { FlopTexture::Dry } else { FlopTexture::Wet };
    let fstrength = if rng.gen_bool(0.5) { FlopStrength::Strong } else { FlopStrength::Weak };

    let bb = 2u32;
    // 3-bet pots are bigger: pre-flop pot is typically 7–11 BB
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
    // Dry + Strong → small c-bet (~33%): dry boards miss villain's range; a small probe
    //               is enough to extract value and start building toward a commitment
    //               given the low SPR.
    // Wet + Strong → large c-bet (~67%): charge draws, deny equity, commit the stack.
    // Any + Weak   → check: no equity; in a low-SPR pot any bet is a large commitment
    //               that is hard to fold to a raise.
    let (correct, branch_key): (&str, &str) = match (texture, fstrength) {
        (FlopTexture::Dry, FlopStrength::Strong) => ("B", "Dry:Strong:SmallCbet"),
        (FlopTexture::Wet, FlopStrength::Strong) => ("C", "Wet:Strong:LargeCbet"),
        (FlopTexture::Dry, FlopStrength::Weak)   => ("A", "Dry:Weak:Check"),
        (FlopTexture::Wet, FlopStrength::Weak)   => ("A", "Wet:Weak:Check"),
    };
    let branch_key = branch_key.to_string();

    let hero_pos  = Position::BTN;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");

    let question = match text_style {
        TextStyle::Simple => format!(
            "You re-raised before the flop and your opponent called. First three cards: {board_str}. \
             You have {hand_str} on the Button. Pot: {pot} chips. Stack: {stack} chips. \
             Your opponent checked to you. What do you do?"
        ),
        TextStyle::Technical => format!(
            "3-bet pot c-bet. You hold {hand_str} ({fstrength}) on the Button (the 3-better). \
             Villain called your 3-bet from the Big Blind. Board: {board_str} ({texture}). \
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
