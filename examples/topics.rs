//! One illustrated example for every training topic.
//!
//! Run with:
//!   cargo run --example topics
//!
//! Each block shows:
//!   • What the topic teaches
//!   • A concrete scenario (fixed seed → always the same cards)
//!   • All answer options — correct one marked with ✓
//!   • The full explanation for every choice

use poker_drill_gen::{generate_training, DifficultyLevel, TextStyle, TrainingRequest, TrainingTopic};

// ── topic metadata ────────────────────────────────────────────────────────────

struct TopicMeta {
    topic: TrainingTopic,
    seed: u64,
    teaches: &'static str,
}

fn topics() -> Vec<TopicMeta> {
    vec![
        TopicMeta {
            topic: TrainingTopic::PreflopDecision,
            seed: 1001,
            teaches: "When to open-raise, call, or fold before the flop based on \
                      hand strength, position, and stack depth.",
        },
        TopicMeta {
            topic: TrainingTopic::PostflopContinuationBet,
            seed: 2002,
            teaches: "How to size a continuation bet (c-bet) on the flop: small on \
                      dry boards with range advantage, large on wet boards to charge draws.",
        },
        TopicMeta {
            topic: TrainingTopic::PotOddsAndEquity,
            seed: 3003,
            teaches: "Whether to call a bet with a drawing hand by comparing the \
                      pot odds offered to your equity (outs × rule-of-2/4).",
        },
        TopicMeta {
            topic: TrainingTopic::BluffSpot,
            seed: 4004,
            teaches: "Picking river spots to bluff: when the board favours your range, \
                      villain is capped, and a well-sized bet can fold enough equity.",
        },
        TopicMeta {
            topic: TrainingTopic::ICMAndTournamentDecision,
            seed: 5005,
            teaches: "Adjusting preflop push/fold thresholds in tournaments where \
                      chip EV ≠ prize equity (ICM pressure near the bubble or final table).",
        },
        TopicMeta {
            topic: TrainingTopic::TurnBarrelDecision,
            seed: 6006,
            teaches: "Whether to fire a second barrel on the turn: double-barrel \
                      when the turn card improves your range or equity; check back when it does not.",
        },
        TopicMeta {
            topic: TrainingTopic::CheckRaiseSpot,
            seed: 7007,
            teaches: "Playing out-of-position on the flop: check-raise strong hands \
                      and combo draws to build the pot and deny equity; check-call medium holdings.",
        },
        TopicMeta {
            topic: TrainingTopic::SemiBluffDecision,
            seed: 8008,
            teaches: "Raising with a draw (flush draw, OESD) as a semi-bluff to win \
                      the pot immediately or with a made hand on the next street.",
        },
        TopicMeta {
            topic: TrainingTopic::AntiLimperIsolation,
            seed: 9009,
            teaches: "Isolating a preflop limper with a larger raise to play heads-up \
                      in position, rather than overlimping into a multiway pot.",
        },
        TopicMeta {
            topic: TrainingTopic::RiverValueBet,
            seed: 1010,
            teaches: "Sizing river value bets by hand strength: overbet with the nuts, \
                      large bet with a strong hand, check with a medium hand to avoid \
                      the thin-value trap.",
        },
        TopicMeta {
            topic: TrainingTopic::SqueezePlay,
            seed: 1111,
            teaches: "Preflop squeeze spots: 3-bet with premium hands to isolate, call \
                      with speculative hands for implied odds, fold with weak hands that \
                      cannot profit from a large pot.",
        },
        TopicMeta {
            topic: TrainingTopic::BigBlindDefense,
            seed: 1212,
            teaches: "Defending the Big Blind against a single raise: 3-bet strong hands \
                      for value, call playable hands exploiting the BB pot-odds discount, \
                      fold off-suit trash even with the discount.",
        },
        TopicMeta {
            topic: TrainingTopic::ThreeBetPotCbet,
            seed: 1313,
            teaches: "C-betting on the flop in a 3-bet pot: use small sizing on dry boards, \
                      large sizing on wet boards to charge draws, and check back weak hands \
                      given the low stack-to-pot ratio.",
        },
        TopicMeta {
            topic: TrainingTopic::RiverCallOrFold,
            seed: 1414,
            teaches: "Facing a river bet: calculate required equity vs the bet size, raise \
                      for value with strong hands against small bets, call marginal hands at \
                      standard odds, and fold weak hands at large bet sizes.",
        },
        TopicMeta {
            topic: TrainingTopic::TurnProbeBet,
            seed: 1515,
            teaches: "Probing the turn OOP after the flop is checked through: exploit \
                      the Button's capped range with a large probe on strong hands, a small \
                      probe on medium hands, and check with weak hands.",
        },
        TopicMeta {
            topic: TrainingTopic::DelayedCbet,
            seed: 1616,
            teaches: "Delayed c-bet on the turn after checking back the flop in position: \
                      bet medium with strong hands for value, small with medium hands on \
                      blank turns, and check weak hands or medium hands on scare cards.",
        },
    ]
}

// ── display helpers ───────────────────────────────────────────────────────────

fn divider(ch: char, n: usize) { println!("{}", ch.to_string().repeat(n)); }

fn print_example(meta: &TopicMeta) {
    let scenario = generate_training(TrainingRequest {
        topic: meta.topic,
        difficulty: DifficultyLevel::Beginner,
        rng_seed: Some(meta.seed),
        text_style: TextStyle::Simple,
    });

    let ts = &scenario.table_setup;

    divider('═', 66);
    println!("  TOPIC : {}", scenario.topic);
    println!("  GAME  : {}   ID: {}   Branch: {}",
        ts.game_type, scenario.scenario_id, scenario.branch_key);
    divider('─', 66);

    // ── What this topic teaches ──
    println!();
    println!("  WHAT THIS TEACHES");
    // Wrap at ~60 chars
    let words: Vec<&str> = meta.teaches.split_whitespace().collect();
    let mut line = String::from("    ");
    for word in &words {
        if line.len() + word.len() + 1 > 64 {
            println!("{line}");
            line = format!("    {word}");
        } else {
            if line.len() > 4 { line.push(' '); }
            line.push_str(word);
        }
    }
    if !line.trim().is_empty() { println!("{line}"); }

    // ── Situation ──
    println!();
    println!("  SITUATION");
    let hand_str: Vec<String> = ts.hero_hand.iter().map(|c| c.to_string()).collect();
    println!("    Hero:     {}  ({})", hand_str.join(" "), ts.hero_position);
    if !ts.board.is_empty() {
        let board_str: Vec<String> = ts.board.iter().map(|c| c.to_string()).collect();
        println!("    Board:    {}", board_str.join(" "));
    }
    println!("    Pot:      {} chips   Bet to call: {} chips",
        ts.pot_size, ts.current_bet);

    // ── Question ──
    println!();
    println!("  QUESTION");
    // Simple word-wrap at ~62 chars
    let q_words: Vec<&str> = scenario.question.split_whitespace().collect();
    let mut qline = String::from("    ");
    for word in &q_words {
        if qline.len() + word.len() + 1 > 66 {
            println!("{qline}");
            qline = format!("    {word}");
        } else {
            if qline.len() > 4 { qline.push(' '); }
            qline.push_str(word);
        }
    }
    if !qline.trim().is_empty() { println!("{qline}"); }

    // ── Answers ──
    println!();
    println!("  ANSWERS");
    for ans in &scenario.answers {
        let marker = if ans.is_correct { " ✓ CORRECT" } else { "          " };
        println!();
        println!("    [{}]{} — {}", ans.id, marker, ans.text);
        // Indent explanation, wrap at ~60
        let exp_words: Vec<&str> = ans.explanation.split_whitespace().collect();
        let mut eline = String::from("        ");
        for word in &exp_words {
            if eline.len() + word.len() + 1 > 68 {
                println!("{eline}");
                eline = format!("        {word}");
            } else {
                if eline.len() > 8 { eline.push(' '); }
                eline.push_str(word);
            }
        }
        if !eline.trim().is_empty() { println!("{eline}"); }
    }

    println!();
}

// ── entry point ───────────────────────────────────────────────────────────────

fn main() {
    println!();
    println!("  POKER DRILL GENERATOR — One example per training topic");
    println!("  Difficulty: Beginner   Seeds: fixed (deterministic)");
    println!();

    for meta in &topics() {
        print_example(meta);
    }

    divider('═', 66);
    println!("  16 topics shown.  Run 'cargo run --example demo' for the");
    println!("  full randomised demo.");
    divider('═', 66);
    println!();
}
