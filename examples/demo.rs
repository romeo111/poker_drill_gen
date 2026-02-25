//! Full demo of all 16 training topics.
//!
//! Run with: `cargo run --example demo`
//!
//! This example shows how `poker_drill_gen` works end to end:
//!
//! 1. **TextStyle comparison** — the same BluffSpot hand is generated twice
//!    (same seed = same cards) in Simple and Technical mode, showing how the
//!    wording changes while the game logic stays identical.
//!
//! 2. **All 16 topics** — one scenario per topic in Simple mode with fixed
//!    seeds, so the output is deterministic and reproducible.
//!
//! ## Key concepts demonstrated
//!
//! - `TrainingRequest::new(topic)` — minimal one-argument constructor; accepts
//!   `TrainingTopic` or `Street` directly. Defaults: Beginner, entropy, Simple.
//! - `rng_seed: Some(u64)` makes the output fully deterministic.
//! - `TextStyle::Simple` uses plain English; `TextStyle::Technical` uses
//!   poker jargon (SPR, EV, fold equity, c-bet, etc.).
//! - The correct answer, cards dealt, and bet sizes are the same in both modes.
//! - Each scenario includes a `branch_key` for progress tracking.

use poker_drill_gen::{
    generate_training, DifficultyLevel, Street, TextStyle, TrainingRequest, TrainingTopic,
};

/// Generate and pretty-print one scenario.
///
/// Shows: topic, game type, text style, scenario ID, branch key, hero hand,
/// board, pot, question, and all answers with explanations.
fn print_scenario(topic: TrainingTopic, seed: u64, style: TextStyle) {
    let scenario = generate_training(TrainingRequest {
        topic: topic.into(),
        difficulty: DifficultyLevel::Intermediate,
        rng_seed: Some(seed),
        text_style: style,
    });

    let ts = &scenario.table_setup;
    let style_label = match style {
        TextStyle::Simple    => "Simple",
        TextStyle::Technical => "Technical",
    };
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [{} — {}]  Style: {}  ID: {}  Branch: {}",
        scenario.topic, ts.game_type, style_label, scenario.scenario_id, scenario.branch_key);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Hero:  {}{}  Position: {}",
        ts.hero_hand[0], ts.hero_hand[1], ts.hero_position);
    if !ts.board.is_empty() {
        let board: Vec<String> = ts.board.iter().map(|c| c.to_string()).collect();
        println!("  Board: {}", board.join(" "));
    }
    println!("  Pot: {}  Bet to call: {}", ts.pot_size, ts.current_bet);
    println!();
    println!("  Q: {}", scenario.question);
    println!();
    for ans in &scenario.answers {
        let marker = if ans.is_correct { "✓" } else { " " };
        println!("  [{}] {marker} {}", ans.id, ans.text);
        // Indent explanation
        for line in ans.explanation.lines() {
            println!("       {}", line);
        }
        println!();
    }
}

fn main() {
    // ── Minimal API ────────────────────────────────────────────────────────
    // TrainingRequest::new() only requires a topic — everything else defaults.
    // Accepts TrainingTopic or Street directly (no .into() needed).
    println!();
    println!("══ Minimal API: TrainingRequest::new() ══");
    println!();
    let s1 = generate_training(TrainingRequest::new(TrainingTopic::PreflopDecision));
    println!("  Specific topic:  {}  ID: {}", s1.topic, s1.scenario_id);
    let s2 = generate_training(TrainingRequest::new(Street::Flop));
    println!("  Random from Flop: {}  ID: {}", s2.topic, s2.scenario_id);
    println!();

    // ── TextStyle comparison ─────────────────────────────────────────────────
    // Same topic + same seed = same cards, same correct answer.
    // Only the wording changes between Simple and Technical.
    println!();
    println!("══ TextStyle comparison: BluffSpot seed=4004 ══");
    println!();
    print_scenario(TrainingTopic::BluffSpot, 4004, TextStyle::Simple);
    print_scenario(TrainingTopic::BluffSpot, 4004, TextStyle::Technical);

    // ── All 16 topics ────────────────────────────────────────────────────────
    // One scenario per topic, fixed seed for reproducible output.
    // Topics are ordered by their internal number (T1–T16).
    println!();
    println!("══ All 16 topics (Simple mode) ══");
    println!();

    let topics = [
        (TrainingTopic::PreflopDecision,          1001u64),
        (TrainingTopic::PostflopContinuationBet,  2002),
        (TrainingTopic::PotOddsAndEquity,         3003),
        (TrainingTopic::BluffSpot,                4004),
        (TrainingTopic::ICMAndTournamentDecision, 5005),
        (TrainingTopic::TurnBarrelDecision,       6006),
        (TrainingTopic::CheckRaiseSpot,           7007),
        (TrainingTopic::SemiBluffDecision,        8008),
        (TrainingTopic::AntiLimperIsolation,      9009),
        (TrainingTopic::RiverValueBet,            1010),
        (TrainingTopic::SqueezePlay,              1111),
        (TrainingTopic::BigBlindDefense,          1212),
        (TrainingTopic::ThreeBetPotCbet,          1313),
        (TrainingTopic::RiverCallOrFold,          1414),
        (TrainingTopic::TurnProbeBet,             1515),
        (TrainingTopic::DelayedCbet,              1616),
    ];

    for (topic, seed) in topics {
        print_scenario(topic, seed, TextStyle::Simple);
    }

    // ── Street selector ──────────────────────────────────────────────────────
    // Instead of a specific topic, pass a Street to get a random topic from
    // that street.  Same seed → same topic + same scenario (deterministic).
    println!();
    println!("══ Street selector: one random drill per street ══");
    println!();

    for (street, seed) in [
        (Street::Preflop, 7001u64),
        (Street::Flop,    7002),
        (Street::Turn,    7003),
        (Street::River,   7004),
    ] {
        let scenario = generate_training(TrainingRequest {
            topic: street.into(),
            difficulty: DifficultyLevel::Intermediate,
            rng_seed: Some(seed),
            text_style: TextStyle::Simple,
        });
        println!("  Street: {street}  →  Topic picked: {}  ID: {}",
            scenario.topic, scenario.scenario_id);
        println!("  Q: {}", scenario.question);
        println!();
    }
}
