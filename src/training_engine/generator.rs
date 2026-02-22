use rand::{rngs::StdRng, SeedableRng};
use rand::RngCore;

use crate::training_engine::{
    models::{TrainingRequest, TrainingScenario, TrainingTopic},
    topics,
};

/// Generate a unique scenario ID from topic + seed.
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
    };
    format!("{}-{:08X}", prefix, rng.next_u32())
}

/// Core dispatch: routes to the correct topic module.
pub fn generate_training(request: TrainingRequest) -> TrainingScenario {
    let mut rng: StdRng = match request.rng_seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None       => StdRng::from_entropy(),
    };

    let scenario_id = make_scenario_id(request.topic, &mut rng);

    match request.topic {
        TrainingTopic::PreflopDecision =>
            topics::preflop::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::PostflopContinuationBet =>
            topics::postflop::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::PotOddsAndEquity =>
            topics::pot_odds::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::BluffSpot =>
            topics::bluff::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::ICMAndTournamentDecision =>
            topics::icm::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::TurnBarrelDecision =>
            topics::turn_barrel::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::CheckRaiseSpot =>
            topics::check_raise::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::SemiBluffDecision =>
            topics::semi_bluff::generate(&mut rng, request.difficulty, scenario_id),

        TrainingTopic::AntiLimperIsolation =>
            topics::anti_limper::generate(&mut rng, request.difficulty, scenario_id),
    }
}
