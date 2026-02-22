pub mod deck;
pub mod evaluator;
pub mod generator;
pub mod models;
pub mod topics;

// Re-export the public API surface.
pub use generator::generate_training;
pub use models::{
    AnswerOption, DifficultyLevel, GameType, PlayerState,
    Position, TableSetup, TrainingRequest, TrainingScenario, TrainingTopic,
};
