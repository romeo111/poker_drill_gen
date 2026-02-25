//! Core training engine — scenario generation, card handling, and poker analysis.
//!
//! ## Module overview
//!
//! | Module      | Purpose |
//! |-------------|---------|
//! | `models`    | All shared types: cards, positions, request/response structs |
//! | `deck`      | 52-card deck with Fisher-Yates shuffle and deterministic dealing |
//! | `evaluator` | Board texture, draw classification, pot-odds math, hand strength |
//! | `helpers`   | Shared builder functions that eliminate boilerplate across topics |
//! | `generator` | Single entry point `generate_training()` — dispatches to topics |
//! | `topics`    | 16 topic generators grouped by street (preflop, flop, turn, river) |

pub mod deck;
pub mod evaluator;
pub mod generator;
pub mod helpers;
pub mod models;
pub mod topics;

// Re-export the public API surface so callers can use
// `training_engine::generate_training` without reaching into sub-modules.
pub use generator::generate_training;
pub use models::{
    AnswerOption, DifficultyLevel, GameType, PlayerState,
    Position, Street, TableSetup, TextStyle, TopicSelector, TrainingRequest,
    TrainingScenario, TrainingTopic,
};
