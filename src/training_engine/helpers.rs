//! Shared builder functions that eliminate boilerplate across topic generators.
//!
//! Every topic generator assembles the same pieces: deal cards, format strings,
//! build answer options, create player lists, and construct the final scenario.
//! These helpers centralise that work so topic files focus on poker logic only.
//!
//! ## RNG ordering
//!
//! `deal()` shuffles the deck and draws hero + board in a fixed order.  This is
//! correct for most topics, but topics with pre-deck RNG calls (T1 preflop,
//! T5 ICM, T6 turn barrel) use `Deck::new_shuffled()` directly to preserve
//! their specific RNG call sequence — changing the order would break
//! determinism tests.

use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::*,
};

/// Deal hero hand (2 cards) + board cards from a freshly shuffled deck.
///
/// This is the standard deal sequence used by most topics.  Returns
/// `(hero_hand, board)` where both are guaranteed disjoint and unique.
pub fn deal<R: Rng>(rng: &mut R, board_cards: usize) -> ([Card; 2], Vec<Card>) {
    let mut deck = Deck::new_shuffled(rng);
    let hand = [deck.deal(), deck.deal()];
    let board = deck.deal_n(board_cards);
    (hand, board)
}

/// Format hero hand as string (e.g. "AcKs").
pub fn hand_str(hand: [Card; 2]) -> String {
    format!("{}{}", hand[0], hand[1])
}

/// Format board as space-separated string (e.g. "Ac Ks 7h").
pub fn board_str(board: &[Card]) -> String {
    board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
}

/// Pick the right wording based on the active text style.
///
/// `Simple` returns beginner-friendly English; `Technical` returns poker jargon.
/// Game logic (correct answer, cards, bet sizes) is identical in both modes.
pub fn styled(ts: TextStyle, simple: String, technical: String) -> String {
    match ts {
        TextStyle::Simple => simple,
        TextStyle::Technical => technical,
    }
}

/// Build one answer option.
///
/// `is_correct` is set automatically by comparing `id == correct`.
/// The explanation is chosen by `TextStyle` via `styled()`.
pub fn answer(
    id: &str, text: impl Into<String>, correct: &str,
    ts: TextStyle, simple: String, tech: String,
) -> AnswerOption {
    AnswerOption {
        id: id.to_string(),
        text: text.into(),
        is_correct: id == correct,
        explanation: styled(ts, simple, tech),
    }
}

/// Build a standard 2-player heads-up setup.
pub fn heads_up(
    hero_pos: Position, villain_pos: Position,
    hero_stack: u32, villain_stack: u32,
) -> Vec<PlayerState> {
    vec![
        PlayerState { seat: 1, position: villain_pos, stack: villain_stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos, stack: hero_stack, is_hero: true, is_active: true },
    ]
}

/// Assemble the final [`TrainingScenario`] from all its parts.
///
/// This is the last call in every topic generator — it bundles hero hand,
/// board, players, pot, question, and answers into the output struct.
pub fn scenario(
    id: String, topic: TrainingTopic, branch_key: impl Into<String>,
    game_type: GameType, hero_pos: Position, hero_hand: [Card; 2],
    board: Vec<Card>, players: Vec<PlayerState>,
    pot: u32, bet: u32, question: String, answers: Vec<AnswerOption>,
) -> TrainingScenario {
    TrainingScenario {
        scenario_id: id,
        topic,
        branch_key: branch_key.into(),
        table_setup: TableSetup {
            game_type,
            hero_position: hero_pos,
            hero_hand,
            board,
            players,
            pot_size: pot,
            current_bet: bet,
        },
        question,
        answers,
    }
}
