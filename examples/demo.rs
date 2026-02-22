//! Run with: cargo run --example demo

use poker_drill_gen::{
    generate_training, DifficultyLevel, TrainingRequest, TrainingTopic,
};

fn print_scenario(topic: TrainingTopic, seed: u64) {
    let scenario = generate_training(TrainingRequest {
        topic,
        difficulty: DifficultyLevel::Intermediate,
        rng_seed: Some(seed),
    });

    let ts = &scenario.table_setup;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [{} — {}]  ID: {}  Branch: {}",
        scenario.topic, ts.game_type, scenario.scenario_id, scenario.branch_key);
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
    ];

    for (topic, seed) in topics {
        print_scenario(topic, seed);
    }
}
