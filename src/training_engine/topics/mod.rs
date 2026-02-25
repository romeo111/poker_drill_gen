//! Topic generators grouped by street.
//!
//! Each module contains all the topic generators for one street of play.
//! Every public function follows the same signature:
//!
//! ```ignore
//! pub fn generate_<name><R: Rng>(
//!     rng: &mut R,
//!     difficulty: DifficultyLevel,
//!     scenario_id: String,
//!     text_style: TextStyle,
//! ) -> TrainingScenario
//! ```
//!
//! The generator dispatches to these via `generator.rs`.

/// T1 (PF-), T5 (IC-), T9 (AL-), T11 (SQ-), T12 (BD-)
pub mod preflop;
/// T2 (CB-), T3 (PO-), T7 (CR-), T8 (SB-), T13 (3B-)
pub mod flop;
/// T6 (TB-), T15 (PB-), T16 (DC-)
pub mod turn;
/// T4 (BL-), T10 (RV-), T14 (RF-)
pub mod river;
