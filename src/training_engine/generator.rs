//! Single entry point for scenario generation.
//!
//! `generate_training()` is the only public function in the crate.  It:
//!
//! 1. Creates a deterministic or entropy-based RNG from the request seed.
//! 2. Generates a unique scenario ID (2-letter prefix + 8-hex-digit suffix).
//! 3. Dispatches to the correct topic generator based on `TrainingTopic`.
//!
//! The RNG is consumed by `make_scenario_id` first (one `next_u32` call),
//! then passed into the topic generator.  This ordering is load-bearing —
//! changing it would break determinism tests.

use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::RngCore;

use crate::training_engine::{
    models::{TopicSelector, TrainingRequest, TrainingScenario, TrainingTopic},
    topics,
};

/// Generate a unique scenario ID: `"{PREFIX}-{8 hex digits}"`.
///
/// Consumes one `next_u32()` call from the RNG.  The 2-letter prefix
/// identifies the topic (e.g. "PF" for PreflopDecision, "CB" for c-bet).
fn make_scenario_id(topic: TrainingTopic, rng: &mut impl RngCore) -> String {
    let prefix = match topic {
        TrainingTopic::PreflopDecision          => "PF",
        TrainingTopic::PostflopContinuationBet  => "CB",
        TrainingTopic::PotOddsAndEquity         => "PO",
        TrainingTopic::BluffSpot                => "BL",
        TrainingTopic::ICMAndTournamentDecision => "IC",
        TrainingTopic::TurnBarrelDecision       => "TB",
        TrainingTopic::CheckRaiseSpot           => "CR",
        TrainingTopic::SemiBluffDecision        => "SB",
        TrainingTopic::AntiLimperIsolation      => "AL",
        TrainingTopic::RiverValueBet            => "RV",
        TrainingTopic::SqueezePlay              => "SQ",
        TrainingTopic::BigBlindDefense          => "BD",
        TrainingTopic::ThreeBetPotCbet          => "3B",
        TrainingTopic::RiverCallOrFold          => "RF",
        TrainingTopic::TurnProbeBet             => "PB",
        TrainingTopic::DelayedCbet              => "DC",
    };
    format!("{}-{:08X}", prefix, rng.next_u32())
}

/// Resolve a [`TopicSelector`] to a concrete [`TrainingTopic`].
///
/// For `TopicSelector::Topic(t)` this is a no-op.  For
/// `TopicSelector::Street(s)` the RNG picks a random topic from that street.
fn resolve_topic(selector: TopicSelector, rng: &mut impl Rng) -> TrainingTopic {
    match selector {
        TopicSelector::Topic(t) => t,
        TopicSelector::Street(s) => {
            let topics = s.topics();
            topics[rng.gen_range(0..topics.len())]
        }
    }
}

/// Generate a complete poker training scenario.
///
/// This is the crate's single public entry point.  Pass a [`TrainingRequest`]
/// with a topic (or street), difficulty, optional seed, and text style.
/// Returns a fully-built [`TrainingScenario`] ready for display.
///
/// When `topic` is a `TopicSelector::Street`, the engine uses the RNG to
/// pick a random topic from that street — still fully deterministic when
/// a seed is provided.
///
/// The 16 topics are dispatched to 4 street-grouped modules:
/// - `topics::preflop` — T1, T5, T9, T11, T12
/// - `topics::flop`    — T2, T3, T7, T8, T13
/// - `topics::turn`    — T6, T15, T16
/// - `topics::river`   — T4, T10, T14
pub fn generate_training(request: TrainingRequest) -> TrainingScenario {
    let mut rng: StdRng = match request.rng_seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None       => StdRng::from_entropy(),
    };

    // Resolve street selector to a concrete topic (consumes RNG for Street mode).
    let topic = resolve_topic(request.topic, &mut rng);

    let scenario_id = make_scenario_id(topic, &mut rng);
    let ts = request.text_style;

    match topic {
        // Preflop topics
        TrainingTopic::PreflopDecision =>
            topics::preflop::generate(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::ICMAndTournamentDecision =>
            topics::preflop::generate_icm(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::AntiLimperIsolation =>
            topics::preflop::generate_anti_limper(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::SqueezePlay =>
            topics::preflop::generate_squeeze(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::BigBlindDefense =>
            topics::preflop::generate_bb_defense(&mut rng, request.difficulty, scenario_id, ts),

        // Flop topics
        TrainingTopic::PostflopContinuationBet =>
            topics::flop::generate_cbet(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::PotOddsAndEquity =>
            topics::flop::generate_pot_odds(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::CheckRaiseSpot =>
            topics::flop::generate_check_raise(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::SemiBluffDecision =>
            topics::flop::generate_semi_bluff(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::ThreeBetPotCbet =>
            topics::flop::generate_3bet_cbet(&mut rng, request.difficulty, scenario_id, ts),

        // Turn topics
        TrainingTopic::TurnBarrelDecision =>
            topics::turn::generate_barrel(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::TurnProbeBet =>
            topics::turn::generate_probe(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::DelayedCbet =>
            topics::turn::generate_delayed_cbet(&mut rng, request.difficulty, scenario_id, ts),

        // River topics
        TrainingTopic::BluffSpot =>
            topics::river::generate_bluff(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::RiverValueBet =>
            topics::river::generate_value_bet(&mut rng, request.difficulty, scenario_id, ts),
        TrainingTopic::RiverCallOrFold =>
            topics::river::generate_call_or_fold(&mut rng, request.difficulty, scenario_id, ts),
    }
}
