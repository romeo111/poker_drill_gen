//! Full randomised demo of all 16 training topics.
//!
//! Run with: cargo run --example demo
//!
//! # TextStyle
//!
//! Every `TrainingRequest` includes a `text_style` field that controls the
//! language used in the question and answer explanations:
//!
//! - `TextStyle::Simple`    — plain English, no poker jargon (the default)
//! - `TextStyle::Technical` — standard poker terminology (SPR, EV, fold equity, c-bet, …)
//!
//! The game logic (correct answer, cards dealt, bet sizes) is identical in
//! both modes.  This demo shows both styles side-by-side for BluffSpot, then
//! runs all 16 topics in Simple mode.

use poker_drill_gen::{
    generate_training, DifficultyLevel, TextStyle, TrainingRequest, TrainingTopic,
};

fn print_scenario(topic: TrainingTopic, seed: u64, style: TextStyle) {
    let scenario = generate_training(TrainingRequest {
        topic,
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
    // ── TextStyle comparison: same hand, two styles ───────────────────────────
    println!();
    println!("══ TextStyle comparison: BluffSpot seed=4004 ══");
    println!();
    print_scenario(TrainingTopic::BluffSpot, 4004, TextStyle::Simple);
    print_scenario(TrainingTopic::BluffSpot, 4004, TextStyle::Technical);

    // ── All 16 topics in Simple mode ─────────────────────────────────────────
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
}
