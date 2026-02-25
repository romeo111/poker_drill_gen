# poker_drill_gen

Rust library that generates randomised poker training scenarios (6-max tables).
Each scenario includes a hand situation, a multiple-choice question, and
per-option explanations.

---

## Quick Start

```rust
use poker_drill_gen::{
    generate_training, DifficultyLevel, Street, TextStyle, TrainingRequest, TrainingTopic,
};

// Minimal — only topic is required:
let s = generate_training(TrainingRequest::new(TrainingTopic::BluffSpot));

// By street — engine picks a random topic from that street:
let s = generate_training(TrainingRequest::new(Street::Flop));

// Full control:
let s = generate_training(TrainingRequest {
    topic:      TrainingTopic::PotOddsAndEquity.into(),
    difficulty: DifficultyLevel::Advanced,
    rng_seed:   Some(42),           // deterministic; None = entropy
    text_style: TextStyle::Technical, // default: Simple
});
```

---

## Input — `TrainingRequest`

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `topic` | `TopicSelector` | *(required)* | `TrainingTopic::X.into()` or `Street::X.into()` |
| `difficulty` | `DifficultyLevel` | `Beginner` | `Beginner` / `Intermediate` / `Advanced` |
| `rng_seed` | `Option<u64>` | `None` | Fixed seed = deterministic output |
| `text_style` | `TextStyle` | `Simple` | `Simple` (plain English) / `Technical` (poker jargon) |

`TrainingRequest::new(topic)` accepts `TrainingTopic` or `Street` directly — no `.into()` needed.

**Topic vs Street:**

| Input | Example | What happens |
|-------|---------|--------------|
| Specific topic | `TrainingTopic::BluffSpot` | Generates that exact topic |
| Street | `Street::Flop` | Picks a random topic from the street |

---

## Output — `TrainingScenario`

| Field | Type | Description |
|-------|------|-------------|
| `scenario_id` | `String` | Unique ID with prefix, e.g. `"PF-3A1C8F02"` |
| `topic` | `TrainingTopic` | Which topic was generated |
| `branch_key` | `String` | Decision branch — stable across seeds, use for progress tracking |
| `table_setup` | `TableSetup` | Hero hand, board, positions, stacks, pot |
| `question` | `String` | The question posed to the player |
| `answers` | `Vec<AnswerOption>` | Choices (exactly one has `is_correct: true`) |

---

## 16 Topics by Street

| Street | Topic | Enum Variant | Prefix |
|--------|-------|-------------|--------|
| **Preflop** | Preflop Decision | `PreflopDecision` | `PF-` |
| | ICM & Tournament | `ICMAndTournamentDecision` | `IC-` |
| | Anti-Limper Isolation | `AntiLimperIsolation` | `AL-` |
| | Squeeze Play | `SqueezePlay` | `SQ-` |
| | Big Blind Defense | `BigBlindDefense` | `BD-` |
| **Flop** | Continuation Bet | `PostflopContinuationBet` | `CB-` |
| | Pot Odds & Equity | `PotOddsAndEquity` | `PO-` |
| | Check-Raise Spot | `CheckRaiseSpot` | `CR-` |
| | Semi-Bluff Decision | `SemiBluffDecision` | `SB-` |
| | 3-Bet Pot C-Bet | `ThreeBetPotCbet` | `3B-` |
| **Turn** | Turn Barrel | `TurnBarrelDecision` | `TB-` |
| | Turn Probe Bet | `TurnProbeBet` | `PB-` |
| | Delayed C-Bet | `DelayedCbet` | `DC-` |
| **River** | Bluff Spot | `BluffSpot` | `BL-` |
| | River Value Bet | `RiverValueBet` | `RV-` |
| | River Call or Fold | `RiverCallOrFold` | `RF-` |

Streets: `Preflop`, `Flop`, `Turn`, `River`

---

## Difficulty

| Level | Stacks | Who it's for |
|-------|--------|-------------|
| `Beginner` | ~100 BB | New players |
| `Intermediate` | 40–150 BB | Comfortable with basics |
| `Advanced` | 15–300 BB | Edge cases, short/deep stack play |

---

## Text Style

Controls question and explanation wording. Game logic (correct answer, cards, sizing) is identical.

**Simple** (default) — plain English, no jargon:
```
Q: You're on the button with a missed flush draw. The pot is 20 chips. Bluff or give up?
A: Bet large (15 chips) — Betting large forces them to fold often enough.
```

**Technical** — standard poker terminology:
```
Q: BTN, missed FD, checked to on river. Pot 20, SPR 4.2. Bluff or check back?
A: Bet large (15, ~75% pot) — High fold equity, sufficient SPR, positive EV.
```

---

## Branch Key

Identifies the logical decision type, stable across seeds:

```
OpenRaise:premium:IP     ← preflop open, premium hand, in-position
Dry:RangeAdv             ← c-bet spot, dry board, range advantage
FlushDraw:Call           ← pot-odds, flush draw, call is correct
```

Use for progress tracking — regenerate any branch for targeted practice.

---

## Guarantees

- Exactly one correct answer per scenario
- Hero cards never on the board
- Board cards are unique
- Same `rng_seed` = same output (deterministic)
