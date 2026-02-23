# poker_drill_gen

A Rust library that generates randomised, deterministic poker training scenarios.
Feed it a topic and difficulty level — get back a hand situation, a multiple-choice
question, and a full explanation for every answer option.

---

## Features

- **9 training topics** covering every major poker decision (preflop, flop, turn, river, ICM)
- **Deterministic** — pass a seed and always get the same scenario (great for testing and sharing)
- **Self-contained** — no network, no database, no external poker solver
- **Fully explained** — every wrong answer tells you *why* it's wrong

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
poker_drill_gen = { path = "." }
```

```rust
use poker_drill_gen::{generate_training, DifficultyLevel, TrainingRequest, TrainingTopic};

let scenario = generate_training(TrainingRequest {
    topic:      TrainingTopic::PreflopDecision,
    difficulty: DifficultyLevel::Beginner,
    rng_seed:   Some(42),   // deterministic; use None for entropy
});

println!("{}", scenario.question);
for ans in &scenario.answers {
    let mark = if ans.is_correct { "✓" } else { " " };
    println!("[{mark}] {} — {}", ans.id, ans.text);
    println!("    {}", ans.explanation);
}
```

Run the built-in examples:

```bash
cargo run --example topics   # one illustrated example per topic
cargo run --example demo     # full randomised demo of all 9 topics
```

---

## Training Topics

| # | Topic | Street | Decision |
|---|-------|--------|---------|
| 1 | Preflop Decision | Preflop | Open-raise, call, or fold based on hand strength and position |
| 2 | Postflop Continuation Bet | Flop | C-bet size vs check based on board texture and range advantage |
| 3 | Pot Odds & Equity | Flop | Call or fold a draw using pot-odds math |
| 4 | Bluff Spot | River | Bluff sizing vs check-back based on SPR and archetype |
| 5 | ICM & Tournament Decision | Preflop | Push/fold with ICM pressure near bubble and final table |
| 6 | Turn Barrel Decision | Turn | Double-barrel vs check-back based on turn card type |
| 7 | Check-Raise Spot | Flop | Check-raise vs check-call vs fold OOP from the Big Blind |
| 8 | Semi-Bluff Decision | Flop | Raise vs call vs fold with a drawing hand |
| 9 | Anti-Limper Isolation | Preflop | Iso-raise vs overlimp vs fold against one or more limpers |

---

## API

```rust
// Single entry point
pub fn generate_training(request: TrainingRequest) -> TrainingScenario

// Key types
pub struct TrainingRequest {
    pub topic:      TrainingTopic,
    pub difficulty: DifficultyLevel,   // Beginner | Intermediate | Advanced
    pub rng_seed:   Option<u64>,
}

pub struct TrainingScenario {
    pub scenario_id: String,           // e.g. "PF-3A1C8F02"
    pub topic:       TrainingTopic,
    pub branch_key:  String,           // logical branch for progress tracking
    pub table_setup: TableSetup,       // cards, board, positions, stack/pot
    pub question:    String,
    pub answers:     Vec<AnswerOption>, // exactly one has is_correct: true
}
```

All types implement `serde::Serialize` / `serde::Deserialize`.

---

## Project Layout

```
src/
  lib.rs                        ← crate root and re-exports
  nt_adapter.rs                 ← to_nt_table_state() JSON adapter
  tests.rs                      ← 27 unit tests
  training_engine/
    models.rs                   ← all shared types
    deck.rs                     ← Fisher-Yates shuffled deck
    evaluator.rs                ← board texture, equity helpers, pot odds
    generator.rs                ← generate_training() dispatcher
    topics/                     ← one module per training topic
examples/
  demo.rs                       ← all 9 topics, random output
  topics.rs                     ← one illustrated example per topic
docs/
  README.md                     ← API reference
  TECHNICAL_SPEC.md             ← full reverse-engineering spec
  topics/                       ← one deep-dive per topic (poker theory + engine notes)
```

---

## Tests

```bash
cargo test
```

27 tests covering determinism, structural invariants, deck integrity, difficulty
levels, entropy mode, and per-topic sanity checks (board length, game type, hero
position, bet presence).

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `rand` | 0.8 | PRNG (`StdRng` / ChaCha12), Fisher-Yates shuffle |
| `serde` | 1 | Serialization derive macros |
| `serde_json` | 1 | JSON output for the NT adapter |

---

## Documentation

| File | Contents |
|------|---------|
| [`docs/README.md`](docs/README.md) | API reference, quick-start, glossary |
| [`docs/TECHNICAL_SPEC.md`](docs/TECHNICAL_SPEC.md) | Language-agnostic spec — data types, all 9 decision tables, invariants |
| [`docs/topics/`](docs/topics/) | Deep-dive per topic: poker theory, worked examples, engine notes |
