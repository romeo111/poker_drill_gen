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

/// Core dispatch: routes to the correct topic module.
pub fn generate_training(request: TrainingRequest) -> TrainingScenario {
    let mut rng: StdRng = match request.rng_seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None       => StdRng::from_entropy(),
    };

    let scenario_id = make_scenario_id(request.topic, &mut rng);
    let ts = request.text_style;

    match request.topic {
        TrainingTopic::PreflopDecision =>
            topics::preflop::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::PostflopContinuationBet =>
            topics::postflop::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::PotOddsAndEquity =>
            topics::pot_odds::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::BluffSpot =>
            topics::bluff::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::ICMAndTournamentDecision =>
            topics::icm::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::TurnBarrelDecision =>
            topics::turn_barrel::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::CheckRaiseSpot =>
            topics::check_raise::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::SemiBluffDecision =>
            topics::semi_bluff::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::AntiLimperIsolation =>
            topics::anti_limper::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::RiverValueBet =>
            topics::river_value_bet::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::SqueezePlay =>
            topics::squeeze_play::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::BigBlindDefense =>
            topics::big_blind_defense::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::ThreeBetPotCbet =>
            topics::three_bet_pot_cbet::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::RiverCallOrFold =>
            topics::river_call_or_fold::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::TurnProbeBet =>
            topics::turn_probe_bet::generate(&mut rng, request.difficulty, scenario_id, ts),

        TrainingTopic::DelayedCbet =>
            topics::delayed_cbet::generate(&mut rng, request.difficulty, scenario_id, ts),
    }
}
