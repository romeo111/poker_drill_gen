//! # poker_drill_gen
//!
//! A fully offline, deterministic poker training scenario generator.
//!
//! This library generates randomised poker training scenarios across 16 topics
//! covering all four streets (preflop, flop, turn, river). Each scenario
//! includes a concrete hand situation, a multiple-choice question, and
//! per-option explanations so players understand the *why* behind each decision.
//!
//! ## How it works
//!
//! 1. Create a [`TrainingRequest`] with a topic, difficulty, optional RNG seed,
//!    and text style.
//! 2. Call [`generate_training`] — the engine shuffles a deck, deals cards,
//!    classifies the situation (hand strength, board texture, draw type, etc.),
//!    picks the correct answer based on poker strategy, and builds dynamic
//!    explanations for every option.
//! 3. The returned [`TrainingScenario`] contains the full table state, question,
//!    and answer options — ready to display in any UI.
//!
//! ## Key features
//!
//! - **Deterministic**: pass `rng_seed: Some(u64)` to reproduce the exact same
//!   scenario every time — useful for tests and progress tracking.
//! - **Two text styles**: `TextStyle::Simple` (plain English, no jargon) and
//!   `TextStyle::Technical` (SPR, EV, fold equity, c-bet, etc.).
//! - **Branch keys**: each scenario includes a `branch_key` that identifies the
//!   logical decision branch (e.g. `"OpenRaise:premium:IP"`) — stable across
//!   seeds, useful for tracking which decision types a student has mastered.
//!
//! ## Quick start
//!
//! ```rust
//! use poker_drill_gen::{
//!     generate_training, DifficultyLevel, Street, TextStyle, TrainingRequest, TrainingTopic,
//! };
//!
//! // Minimal — only topic is required (defaults: Beginner, entropy, Simple):
//! let scenario = generate_training(TrainingRequest::new(TrainingTopic::PreflopDecision));
//! println!("Q: {}", scenario.question);
//!
//! // Full control — set every field:
//! let scenario = generate_training(TrainingRequest {
//!     topic: TrainingTopic::BluffSpot.into(),
//!     difficulty: DifficultyLevel::Intermediate,
//!     rng_seed: Some(42),
//!     text_style: TextStyle::Technical,
//! });
//!
//! println!("Scenario: {}", scenario.scenario_id);
//! for ans in &scenario.answers {
//!     let mark = if ans.is_correct { "+" } else { " " };
//!     println!("[{mark}] {} — {}", ans.id, ans.text);
//! }
//!
//! // Random topic from a street:
//! let flop_drill = generate_training(TrainingRequest::new(Street::Flop));
//! println!("Random flop drill: {}", flop_drill.topic);
//! ```

pub mod training_engine;

// Convenience re-exports so callers can use `poker_drill_gen::generate_training`
// directly without reaching into `training_engine::`.
pub use training_engine::{
    generate_training, AnswerOption, DifficultyLevel, GameType,
    PlayerState, Position, Street, TableSetup, TextStyle, TopicSelector,
    TrainingRequest, TrainingScenario, TrainingTopic,
};

#[cfg(test)]
mod tests;
