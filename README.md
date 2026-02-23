# poker_drill_gen

A Rust library that generates randomised, deterministic poker training scenarios.
Feed it a topic and difficulty level — get back a hand situation, a multiple-choice
question, and a full explanation for every answer option.

---

## Features

- **15 training topics** covering every major poker decision (preflop, flop, turn, river, ICM)
- **Deterministic** — pass a seed and always get the same scenario (great for testing and sharing)
- **Self-contained** — no network, no database, no external poker solver
- **Fully explained** — every wrong answer tells you *why* it's wrong
- **Two text styles** — `Simple` (plain English, no jargon) or `Technical` (SPR, EV, fold equity, c-bet, etc.)

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
poker_drill_gen = { path = "." }
```

```rust
use poker_drill_gen::{generate_training, DifficultyLevel, TextStyle, TrainingRequest, TrainingTopic};

let scenario = generate_training(TrainingRequest {
    topic:      TrainingTopic::PreflopDecision,
    difficulty: DifficultyLevel::Beginner,
    rng_seed:   Some(42),   // deterministic; use None for entropy
    text_style: TextStyle::Simple,  // plain English (default); or TextStyle::Technical
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
cargo run --example demo     # full randomised demo of all 15 topics
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
| 10 | River Value Bet | River | Overbet / standard bet / check based on hand strength against a checked-to villain |
| 11 | Squeeze Play | Preflop | Squeeze 3-bet vs call vs fold facing an open + callers |
| 12 | Big Blind Defense | Preflop | 3-bet vs call vs fold from the BB against a single raise |
| 13 | 3-Bet Pot C-Bet | Flop | Small vs large vs check c-bet in a 3-bet pot based on board texture |
| 14 | River Call or Fold | River | Call vs fold vs value-raise facing a villain river bet |
| 15 | Turn Probe Bet | Turn | Probe large vs small vs check OOP on the turn after a checked flop |


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
    pub text_style: TextStyle,         // Simple (default) | Technical
}

pub enum TextStyle {
    Simple,    // plain English, no poker jargon — DEFAULT
    Technical, // standard poker terminology (SPR, EV, fold equity, c-bet, etc.)
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

## Text Style

The `text_style` field on `TrainingRequest` controls the language used in the `question` and
all `AnswerOption.explanation` strings. The game logic — which answer is correct, what cards
are dealt, bet sizes — is identical in both modes. Only the text changes.

| Style | Audience | Description |
|-------|----------|-------------|
| `TextStyle::Simple` | Beginners | Plain English with no poker jargon. Concepts are explained in everyday language. **This is the default.** |
| `TextStyle::Technical` | Experienced players | Standard poker terminology: SPR, EV, fold equity, c-bet, range advantage, GTO, etc. |

```rust
// Beginner-friendly (default)
let s = generate_training(TrainingRequest {
    topic:      TrainingTopic::BluffSpot,
    difficulty: DifficultyLevel::Beginner,
    rng_seed:   Some(42),
    text_style: TextStyle::Simple,   // or Default::default()
});

// Pro mode
let s = generate_training(TrainingRequest {
    topic:      TrainingTopic::BluffSpot,
    difficulty: DifficultyLevel::Advanced,
    rng_seed:   None,
    text_style: TextStyle::Technical,
});
```

---

## Project Layout

```
src/
  lib.rs                        ← crate root and re-exports
  nt_adapter.rs                 ← to_nt_table_state() JSON adapter
  tests.rs                      ← 33 unit tests
  training_engine/
    models.rs                   ← all shared types
    deck.rs                     ← Fisher-Yates shuffled deck
    evaluator.rs                ← board texture, equity helpers, pot odds
    generator.rs                ← generate_training() dispatcher
    topics/                     ← one module per training topic
examples/
  demo.rs                       ← all 15 topics, random output
  topics.rs                     ← one illustrated example per topic
docs/
  README.md                     ← API reference
  TECHNICAL_SPEC.md             ← full reverse-engineering spec
  topics/                       ← one deep-dive per topic (poker theory + engine notes)
```

---

## Scenario Space

**~1.6 trillion unique scenarios** across all 15 topics (raw card combinations × parameter variance).

| Topic group | Dominant factor | ≈ Unique scenarios |
|-------------|----------------|-------------------|
| T4, T10, T14 (river topics) | C(52,2) × C(50,5) × scenario params | ~500B each |
| T6, T15 (turn topics) | C(52,2) × C(50,4) × positions × stack/pot | ~60B each |
| T2, T3, T7, T8, T13 (flop topics) | C(52,2) × C(50,3) × parameters | ~2–5B each |
| T1, T5, T9, T11, T12 (preflop topics) | C(52,2) × position/stack/stage params | ~1–6M each |

The three river topics together account for ~90% of the total due to C(50,5) ≈ 2.1M board combinations.
From a *strategy* perspective the engine covers ~100 meaningfully distinct situations,
captured by the `branch_key` field.

**Board card distribution** (assuming uniform topic selection across all 15 topics):

| Street | Topics | Count | Share |
|--------|--------|-------|-------|
| Preflop (0 cards) | T1, T5, T9, T11, T12 | 5 | 33% |
| Flop (3 cards) | T2, T3, T7, T8, T13 | 5 | 33% |
| Turn (4 cards) | T6, T15 | 2 | 13% |
| River (5 cards) | T4, T10, T14 | 3 | 20% |

With 15 topics the distribution is balanced: preflop, flop, and river each get equal
or near-equal representation. To shift the balance, weight topic selection in the
request rather than picking uniformly.

---

## Tests

```bash
cargo test
```

36 tests covering determinism, structural invariants, deck integrity, difficulty
levels, entropy mode, per-topic sanity checks (board length, game type, hero
position, bet presence), and TextStyle behaviour (non-empty text, Simple vs
Technical produce different output, correct answer is unaffected by style).

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
| [`docs/TECHNICAL_SPEC.md`](docs/TECHNICAL_SPEC.md) | Language-agnostic spec — data types, all 15 decision tables, invariants |
| [`docs/topics/`](docs/topics/) | Deep-dive per topic: poker theory, worked examples, engine notes |
