// Integrated into the crate via `#[cfg(test)]` — included from lib.rs via `mod tests`.

use crate::training_engine::{
    generate_training, DifficultyLevel, TrainingRequest, TrainingTopic,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn req(topic: TrainingTopic, seed: u64) -> TrainingRequest {
    TrainingRequest {
        topic,
        difficulty: DifficultyLevel::Intermediate,
        rng_seed: Some(seed),
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn same_seed_produces_identical_scenarios() {
    for topic in all_topics() {
        let a = generate_training(req(topic, 12345));
        let b = generate_training(req(topic, 12345));
        assert_eq!(a.scenario_id, b.scenario_id, "topic {:?}", topic);
        assert_eq!(a.question,    b.question,    "topic {:?}", topic);
        assert_eq!(a.answers.len(), b.answers.len());
        for (x, y) in a.answers.iter().zip(b.answers.iter()) {
            assert_eq!(x.id,         y.id);
            assert_eq!(x.text,       y.text);
            assert_eq!(x.is_correct, y.is_correct);
        }
    }
}

#[test]
fn different_seeds_usually_differ() {
    // Not guaranteed to differ 100% of the time, but should differ across a range.
    let mut same_count = 0usize;
    for seed in 0..20u64 {
        let a = generate_training(req(TrainingTopic::PreflopDecision, seed));
        let b = generate_training(req(TrainingTopic::PreflopDecision, seed + 100));
        if a.question == b.question {
            same_count += 1;
        }
    }
    assert!(same_count < 10, "Too many identical questions across different seeds");
}

// ---------------------------------------------------------------------------
// Structural correctness
// ---------------------------------------------------------------------------

#[test]
fn every_scenario_has_exactly_one_correct_answer() {
    for topic in all_topics() {
        for seed in [1u64, 42, 9999, 0xDEAD_BEEF, 7] {
            let scenario = generate_training(req(topic, seed));
            let correct_count = scenario.answers.iter().filter(|a| a.is_correct).count();
            assert_eq!(
                correct_count, 1,
                "Expected exactly 1 correct answer for {:?} seed={}", topic, seed
            );
        }
    }
}

#[test]
fn every_scenario_has_at_least_two_answers() {
    for topic in all_topics() {
        let scenario = generate_training(req(topic, 42));
        assert!(scenario.answers.len() >= 2, "Too few answers for {:?}", topic);
    }
}

#[test]
fn every_answer_has_non_empty_explanation() {
    for topic in all_topics() {
        let scenario = generate_training(req(topic, 77));
        for ans in &scenario.answers {
            assert!(
                !ans.explanation.is_empty(),
                "Empty explanation for answer {} in {:?}", ans.id, topic
            );
        }
    }
}

#[test]
fn scenario_id_contains_topic_prefix() {
    let prefixes = [
        (TrainingTopic::PreflopDecision,          "PF-"),
        (TrainingTopic::PostflopContinuationBet,  "CB-"),
        (TrainingTopic::PotOddsAndEquity,         "PO-"),
        (TrainingTopic::BluffSpot,                "BL-"),
        (TrainingTopic::ICMAndTournamentDecision, "IC-"),
        (TrainingTopic::TurnBarrelDecision,       "TB-"),
        (TrainingTopic::CheckRaiseSpot,           "CR-"),
        (TrainingTopic::SemiBluffDecision,        "SB-"),
        (TrainingTopic::AntiLimperIsolation,      "AL-"),
    ];
    for (topic, prefix) in prefixes {
        let s = generate_training(req(topic, 1));
        assert!(
            s.scenario_id.starts_with(prefix),
            "ID '{}' doesn't start with '{}'", s.scenario_id, prefix
        );
    }
}

// ---------------------------------------------------------------------------
// Deck integrity
// ---------------------------------------------------------------------------

#[test]
fn hero_hand_cards_are_not_on_the_board() {
    for topic in all_topics() {
        for seed in [10u64, 20, 30, 40, 50] {
            let scenario = generate_training(req(topic, seed));
            let ts = &scenario.table_setup;
            for hand_card in &ts.hero_hand {
                assert!(
                    !ts.board.contains(hand_card),
                    "Hero hand card {} also appears on board in {:?} seed={}", hand_card, topic, seed
                );
            }
        }
    }
}

#[test]
fn board_cards_are_unique() {
    for topic in all_topics() {
        let scenario = generate_training(req(topic, 55));
        let board = &scenario.table_setup.board;
        let mut seen = std::collections::HashSet::new();
        for c in board {
            let key = (c.rank.0, c.suit as u8);
            assert!(seen.insert(key), "Duplicate board card: {}", c);
        }
    }
}

// ---------------------------------------------------------------------------
// branch_key
// ---------------------------------------------------------------------------

#[test]
fn every_scenario_has_non_empty_branch_key() {
    for topic in all_topics() {
        for seed in [1u64, 42, 999] {
            let s = generate_training(req(topic, seed));
            assert!(
                !s.branch_key.is_empty(),
                "Empty branch_key for {:?} seed={}", topic, seed
            );
        }
    }
}

#[test]
fn same_seed_produces_identical_branch_key() {
    for topic in all_topics() {
        let a = generate_training(req(topic, 12345));
        let b = generate_training(req(topic, 12345));
        assert_eq!(a.branch_key, b.branch_key, "branch_key not deterministic for {:?}", topic);
    }
}

// ---------------------------------------------------------------------------
// Per-topic sanity checks
// ---------------------------------------------------------------------------

#[test]
fn preflop_has_no_board_cards() {
    let s = generate_training(req(TrainingTopic::PreflopDecision, 1));
    assert!(s.table_setup.board.is_empty(), "Preflop should have no board cards");
}

#[test]
fn postflop_has_exactly_3_board_cards() {
    let s = generate_training(req(TrainingTopic::PostflopContinuationBet, 1));
    assert_eq!(s.table_setup.board.len(), 3, "Postflop should have a 3-card flop");
}

#[test]
fn pot_odds_has_a_positive_current_bet() {
    let s = generate_training(req(TrainingTopic::PotOddsAndEquity, 1));
    assert!(s.table_setup.current_bet > 0, "Pot odds scenario must have a bet to call");
}

#[test]
fn bluff_has_5_board_cards() {
    let s = generate_training(req(TrainingTopic::BluffSpot, 1));
    assert_eq!(s.table_setup.board.len(), 5, "Bluff spot should be on the river (5 cards)");
}

#[test]
fn icm_scenario_is_tournament_game_type() {
    use crate::training_engine::GameType;
    let s = generate_training(req(TrainingTopic::ICMAndTournamentDecision, 1));
    assert_eq!(s.table_setup.game_type, GameType::Tournament);
}

#[test]
fn difficulty_levels_all_work() {
    for diff in [DifficultyLevel::Beginner, DifficultyLevel::Intermediate, DifficultyLevel::Advanced] {
        let r = TrainingRequest {
            topic: TrainingTopic::PreflopDecision,
            difficulty: diff,
            rng_seed: Some(1),
        };
        let s = generate_training(r);
        assert!(!s.question.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Per-topic sanity checks (new topics)
// ---------------------------------------------------------------------------

#[test]
fn turn_barrel_has_4_board_cards() {
    for seed in [1u64, 42, 999] {
        let s = generate_training(req(TrainingTopic::TurnBarrelDecision, seed));
        assert_eq!(
            s.table_setup.board.len(), 4,
            "TurnBarrelDecision should have 3 flop + 1 turn = 4 board cards (seed {})", seed
        );
    }
}

#[test]
fn check_raise_has_3_board_cards_and_is_cash() {
    use crate::training_engine::GameType;
    for seed in [1u64, 42, 999] {
        let s = generate_training(req(TrainingTopic::CheckRaiseSpot, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "CheckRaiseSpot should have a 3-card flop (seed {})", seed
        );
        assert_eq!(s.table_setup.game_type, GameType::CashGame);
        assert_eq!(s.table_setup.hero_position, crate::training_engine::Position::BB,
            "CheckRaiseSpot hero must be BB (seed {})", seed);
    }
}

#[test]
fn semi_bluff_has_3_board_cards_and_positive_bet() {
    for seed in [1u64, 42, 999] {
        let s = generate_training(req(TrainingTopic::SemiBluffDecision, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "SemiBluffDecision should have a 3-card flop (seed {})", seed
        );
        assert!(
            s.table_setup.current_bet > 0,
            "SemiBluffDecision must have a villain bet to react to (seed {})", seed
        );
    }
}

#[test]
fn anti_limper_has_no_board_cards_and_is_cash() {
    use crate::training_engine::GameType;
    for seed in [1u64, 42, 999] {
        let s = generate_training(req(TrainingTopic::AntiLimperIsolation, seed));
        assert!(
            s.table_setup.board.is_empty(),
            "AntiLimperIsolation is preflop and must have no board cards (seed {})", seed
        );
        assert_eq!(s.table_setup.game_type, GameType::CashGame);
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn all_topics() -> [TrainingTopic; 9] {
    [
        TrainingTopic::PreflopDecision,
        TrainingTopic::PostflopContinuationBet,
        TrainingTopic::PotOddsAndEquity,
        TrainingTopic::BluffSpot,
        TrainingTopic::ICMAndTournamentDecision,
        TrainingTopic::TurnBarrelDecision,
        TrainingTopic::CheckRaiseSpot,
        TrainingTopic::SemiBluffDecision,
        TrainingTopic::AntiLimperIsolation,
    ]
}
