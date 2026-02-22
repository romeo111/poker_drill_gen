//! # poker_drill_gen
//!
//! A fully offline, deterministic poker training scenario generator.
//!
//! ## Quick start
//!
//! ```rust
//! use poker_drill_gen::training_engine::{
//!     generate_training, DifficultyLevel, TrainingRequest, TrainingTopic,
//! };
//!
//! let scenario = generate_training(TrainingRequest {
//!     topic: TrainingTopic::PreflopDecision,
//!     difficulty: DifficultyLevel::Intermediate,
//!     rng_seed: Some(42),
//! });
//!
//! println!("Scenario: {}", scenario.scenario_id);
//! println!("Q: {}", scenario.question);
//! for ans in &scenario.answers {
//!     println!("[{}] {} — correct={}", ans.id, ans.text, ans.is_correct);
//! }
//! ```

pub mod training_engine;
pub mod nt_adapter;

// Convenience re-exports at crate root.
pub use training_engine::{
    generate_training, AnswerOption, DifficultyLevel, GameType,
    PlayerState, Position, TableSetup, TrainingRequest,
    TrainingScenario, TrainingTopic,
};
pub use nt_adapter::to_nt_table_state;

#[cfg(test)]
mod tests;
