//! Unit tests for the `poker_drill_gen` crate.
//!
//! Included from `lib.rs` under `#[cfg(test)]`.
//!
//! # Coverage
//!
//! | Group | What is tested |
//! |-------|----------------|
//! | Determinism | Same seed → identical output; different seeds → varied output |
//! | Structural | One correct answer; ≥2 answers; non-empty explanations; ID prefixes; non-empty branch keys |
//! | Deck integrity | Hero cards absent from board; board cards unique |
//! | Per-topic | Street (board card count), game type, hero position, bet presence |
//! | Difficulty | All three levels produce valid scenarios |
//! | Entropy | `rng_seed: None` produces a valid scenario (smoke test) |

use crate::training_engine::{
    generate_training, DifficultyLevel, GameType, Position, TrainingRequest, TrainingTopic,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a deterministic `TrainingRequest` at Intermediate difficulty.
fn req(topic: TrainingTopic, seed: u64) -> TrainingRequest {
    TrainingRequest {
        topic,
        difficulty: DifficultyLevel::Intermediate,
        rng_seed: Some(seed),
    }
}

/// All nine training topics in canonical order.
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

/// Five seeds that span different RNG states.
const SEEDS: [u64; 5] = [1, 42, 999, 0xDEAD_BEEF, 7];

// ── determinism ──────────────────────────────────────────────────────────────

#[test]
fn same_seed_produces_identical_scenario() {
    for topic in all_topics() {
        let a = generate_training(req(topic, 12345));
        let b = generate_training(req(topic, 12345));
        assert_eq!(a.scenario_id, b.scenario_id, "scenario_id mismatch for {topic:?}");
        assert_eq!(a.question,    b.question,    "question mismatch for {topic:?}");
        assert_eq!(a.branch_key, b.branch_key,  "branch_key mismatch for {topic:?}");
        assert_eq!(a.answers.len(), b.answers.len(), "answer count mismatch for {topic:?}");
        for (x, y) in a.answers.iter().zip(b.answers.iter()) {
            assert_eq!(x.id,         y.id,         "answer id mismatch for {topic:?}");
            assert_eq!(x.text,       y.text,        "answer text mismatch for {topic:?}");
            assert_eq!(x.is_correct, y.is_correct,  "is_correct mismatch for {topic:?}");
        }
    }
}

#[test]
fn different_seeds_produce_varied_questions() {
    // Checks that varying the seed produces different questions across a wide
    // range. Not a hard guarantee (hash collisions are theoretically possible)
    // but holds in practice for all reasonable seed ranges.
    let mut same_count = 0usize;
    let pairs = 40u64;
    for seed in 0..pairs {
        let a = generate_training(req(TrainingTopic::PreflopDecision, seed));
        let b = generate_training(req(TrainingTopic::PreflopDecision, seed + 500));
        if a.question == b.question {
            same_count += 1;
        }
    }
    assert!(
        same_count < pairs as usize / 4,
        "Too many identical questions across different seeds ({same_count}/{pairs})"
    );
}

#[test]
fn entropy_seed_produces_a_valid_scenario() {
    // Smoke test: rng_seed: None must not panic and must satisfy all invariants.
    let s = generate_training(TrainingRequest {
        topic: TrainingTopic::PreflopDecision,
        difficulty: DifficultyLevel::Intermediate,
        rng_seed: None,
    });
    assert!(!s.scenario_id.is_empty());
    assert!(!s.question.is_empty());
    assert!(!s.branch_key.is_empty());
    let correct_count = s.answers.iter().filter(|a| a.is_correct).count();
    assert_eq!(correct_count, 1, "entropy scenario must have exactly one correct answer");
}

// ── structural invariants ─────────────────────────────────────────────────────

#[test]
fn every_scenario_has_exactly_one_correct_answer() {
    for topic in all_topics() {
        for seed in SEEDS {
            let scenario = generate_training(req(topic, seed));
            let correct = scenario.answers.iter().filter(|a| a.is_correct).count();
            assert_eq!(
                correct, 1,
                "Expected exactly 1 correct answer for {topic:?} seed={seed} \
                 (got {correct})"
            );
        }
    }
}

#[test]
fn every_scenario_has_at_least_two_answers() {
    for topic in all_topics() {
        let scenario = generate_training(req(topic, 42));
        assert!(
            scenario.answers.len() >= 2,
            "{topic:?} must have at least 2 answer options (got {})",
            scenario.answers.len()
        );
    }
}

#[test]
fn every_answer_has_non_empty_text_and_explanation() {
    for topic in all_topics() {
        let scenario = generate_training(req(topic, 77));
        for ans in &scenario.answers {
            assert!(
                !ans.text.is_empty(),
                "Empty text for answer {} in {topic:?}",
                ans.id
            );
            assert!(
                !ans.explanation.is_empty(),
                "Empty explanation for answer {} in {topic:?}",
                ans.id
            );
        }
    }
}

#[test]
fn every_scenario_id_starts_with_topic_prefix() {
    let expected_prefixes = [
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
    for (topic, prefix) in expected_prefixes {
        let s = generate_training(req(topic, 1));
        assert!(
            s.scenario_id.starts_with(prefix),
            "ID '{}' for {topic:?} does not start with expected prefix '{prefix}'",
            s.scenario_id
        );
    }
}

#[test]
fn every_scenario_has_non_empty_branch_key() {
    for topic in all_topics() {
        for seed in SEEDS {
            let s = generate_training(req(topic, seed));
            assert!(
                !s.branch_key.is_empty(),
                "Empty branch_key for {topic:?} seed={seed}"
            );
        }
    }
}

#[test]
fn branch_key_is_deterministic() {
    for topic in all_topics() {
        let a = generate_training(req(topic, 12345));
        let b = generate_training(req(topic, 12345));
        assert_eq!(
            a.branch_key, b.branch_key,
            "branch_key is not deterministic for {topic:?}"
        );
    }
}

// ── deck integrity ────────────────────────────────────────────────────────────

#[test]
fn hero_hand_cards_not_on_board() {
    for topic in all_topics() {
        for seed in [10u64, 20, 30, 40, 50] {
            let s = generate_training(req(topic, seed));
            let ts = &s.table_setup;
            for card in &ts.hero_hand {
                assert!(
                    !ts.board.contains(card),
                    "Hero card {card} is also on the board in {topic:?} seed={seed}"
                );
            }
        }
    }
}

#[test]
fn board_cards_are_unique() {
    for topic in all_topics() {
        for seed in SEEDS {
            let s = generate_training(req(topic, seed));
            let board = &s.table_setup.board;
            let mut seen = std::collections::HashSet::new();
            for card in board {
                // Use Display ("Ah", "Kc", …) as the deduplication key.
                // Avoids `suit as u8` which is explicitly disallowed (Suit has no numeric repr).
                let key = card.to_string();
                assert!(
                    seen.insert(key.clone()),
                    "Duplicate board card '{key}' in {topic:?} seed={seed}"
                );
            }
        }
    }
}

#[test]
fn hero_hand_is_always_two_cards() {
    for topic in all_topics() {
        let s = generate_training(req(topic, 1));
        assert_eq!(
            s.table_setup.hero_hand.len(), 2,
            "Hero hand for {topic:?} must always be exactly 2 cards"
        );
    }
}

// ── difficulty levels ─────────────────────────────────────────────────────────

#[test]
fn all_difficulty_levels_produce_valid_scenarios() {
    for diff in [
        DifficultyLevel::Beginner,
        DifficultyLevel::Intermediate,
        DifficultyLevel::Advanced,
    ] {
        for topic in all_topics() {
            let s = generate_training(TrainingRequest {
                topic,
                difficulty: diff,
                rng_seed: Some(1),
            });
            assert!(!s.question.is_empty(), "{topic:?} at {diff:?} produced empty question");
            let correct = s.answers.iter().filter(|a| a.is_correct).count();
            assert_eq!(correct, 1, "{topic:?} at {diff:?} must have exactly 1 correct answer");
        }
    }
}

// ── per-topic sanity checks ───────────────────────────────────────────────────

#[test]
fn preflop_decision_has_no_board_cards() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::PreflopDecision, seed));
        assert!(
            s.table_setup.board.is_empty(),
            "PreflopDecision must have no board cards (seed={seed})"
        );
    }
}

#[test]
fn postflop_cbet_has_exactly_3_board_cards() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::PostflopContinuationBet, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "PostflopContinuationBet must have a 3-card flop (seed={seed})"
        );
    }
}

#[test]
fn pot_odds_has_3_board_cards_and_positive_bet() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::PotOddsAndEquity, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "PotOddsAndEquity must be on the flop (3 board cards) (seed={seed})"
        );
        assert!(
            s.table_setup.current_bet > 0,
            "PotOddsAndEquity must have a villain bet to call (seed={seed})"
        );
    }
}

#[test]
fn bluff_spot_has_5_board_cards() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::BluffSpot, seed));
        assert_eq!(
            s.table_setup.board.len(), 5,
            "BluffSpot must be on the river (5 board cards) (seed={seed})"
        );
    }
}

#[test]
fn icm_scenario_is_tournament_type() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::ICMAndTournamentDecision, seed));
        assert_eq!(
            s.table_setup.game_type,
            GameType::Tournament,
            "ICMAndTournamentDecision must be GameType::Tournament (seed={seed})"
        );
    }
}

#[test]
fn turn_barrel_has_4_board_cards() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::TurnBarrelDecision, seed));
        assert_eq!(
            s.table_setup.board.len(), 4,
            "TurnBarrelDecision must have 3 flop + 1 turn = 4 board cards (seed={seed})"
        );
    }
}

#[test]
fn check_raise_has_3_board_cards_positive_bet_and_bb_hero() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::CheckRaiseSpot, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "CheckRaiseSpot must have a 3-card flop (seed={seed})"
        );
        assert_eq!(
            s.table_setup.game_type,
            GameType::CashGame,
            "CheckRaiseSpot must be a cash game (seed={seed})"
        );
        assert_eq!(
            s.table_setup.hero_position,
            Position::BB,
            "CheckRaiseSpot hero must be in the Big Blind (seed={seed})"
        );
        assert!(
            s.table_setup.current_bet > 0,
            "CheckRaiseSpot must have a villain bet to react to (seed={seed})"
        );
    }
}

#[test]
fn semi_bluff_has_3_board_cards_and_positive_bet() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::SemiBluffDecision, seed));
        assert_eq!(
            s.table_setup.board.len(), 3,
            "SemiBluffDecision must have a 3-card flop (seed={seed})"
        );
        assert!(
            s.table_setup.current_bet > 0,
            "SemiBluffDecision must have a villain bet to react to (seed={seed})"
        );
    }
}

#[test]
fn anti_limper_has_no_board_cards_and_is_cash() {
    for seed in SEEDS {
        let s = generate_training(req(TrainingTopic::AntiLimperIsolation, seed));
        assert!(
            s.table_setup.board.is_empty(),
            "AntiLimperIsolation is preflop and must have no board cards (seed={seed})"
        );
        assert_eq!(
            s.table_setup.game_type,
            GameType::CashGame,
            "AntiLimperIsolation must be a cash game (seed={seed})"
        );
    }
}
