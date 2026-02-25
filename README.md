# poker_drill_gen

A Rust library that generates poker training scenarios — randomised by default,
fully deterministic with a seed. Feed it a topic (or a street) and get back a
hand situation, a multiple-choice question, and an explanation for every answer
option.

## Usage

```rust
use poker_drill_gen::{
    generate_training, DifficultyLevel, Street, TextStyle, TrainingRequest, TrainingTopic,
};

// By topic — generate a specific drill:
let s = generate_training(TrainingRequest::new(TrainingTopic::BluffSpot));

// By street — engine picks a random topic from that street:
let s = generate_training(TrainingRequest::new(Street::Flop));

// Full control — set every parameter:
let s = generate_training(TrainingRequest {
    topic:      TrainingTopic::PotOddsAndEquity.into(),
    difficulty: DifficultyLevel::Advanced,
    rng_seed:   Some(42),
    text_style: TextStyle::Technical,
});
```

## Parameters

Only `topic` is required. Everything else has a default.

| Field | Type | Default | When omitted |
|-------|------|---------|-------------|
| `topic` | `TrainingTopic` or `Street` | *(required)* | — |
| `difficulty` | `DifficultyLevel` | `Beginner` | Fixed ~100 BB stacks, narrow bet sizes |
| `rng_seed` | `Option<u64>` | `None` | Cards, positions, and stacks are randomised from entropy |
| `text_style` | `TextStyle` | `Simple` | Plain English, no poker jargon |

**What gets randomised** when you only pass a topic: hero cards, board cards,
hero position, villain stacks, pot/bet sizes, and (for street mode) which topic
within the street is picked. Pass `rng_seed: Some(n)` to freeze all of that.

## Output

`generate_training()` returns a `TrainingScenario`:

| Field | What it is |
|-------|-----------|
| `scenario_id` | Unique ID with topic prefix, e.g. `"PF-3A1C8F02"` |
| `topic` | Which topic was generated |
| `branch_key` | Decision branch (stable across seeds) — use for progress tracking |
| `table_setup` | Hero hand, board, positions, stacks, pot |
| `question` | The question posed to the player |
| `answers` | Answer choices — exactly one has `is_correct: true` |

## 16 Topics across 4 Streets

| Street | Topics |
|--------|--------|
| **Preflop** | Preflop Decision, ICM & Tournament, Anti-Limper Isolation, Squeeze Play, Big Blind Defense |
| **Flop** | Continuation Bet, Pot Odds & Equity, Check-Raise Spot, Semi-Bluff, 3-Bet Pot C-Bet |
| **Turn** | Turn Barrel, Turn Probe Bet, Delayed C-Bet |
| **River** | Bluff Spot, River Value Bet, River Call or Fold |

## Examples

```bash
cargo run --example demo     # all 16 topics + street selector
cargo run --example topics   # one illustrated example per topic
```

## Docs

- [`docs/README.md`](docs/README.md) — API reference
- [`docs/topics/`](docs/topics/) — poker theory + engine notes per topic
