# poker_drill_gen — Documentation

`poker_drill_gen` is a Rust library that generates randomised poker training scenarios.
Each scenario presents a concrete hand situation, a multiple-choice question, and
a per-option explanation so players understand the *why* behind each decision.

---

## Quick Start

```rust
use poker_drill_gen::{
    generate_training, DifficultyLevel, Street, TextStyle, TrainingRequest, TrainingTopic,
};

// Minimal — only topic is required (defaults: Beginner, entropy, Simple):
let scenario = generate_training(TrainingRequest::new(TrainingTopic::PreflopDecision));
println!("{}", scenario.question);

// Random topic from a street (minimal):
let flop_drill = generate_training(TrainingRequest::new(Street::Flop));
println!("Got: {}", flop_drill.topic); // e.g. "Check-Raise Spot"

// Full control — set every field explicitly:
let scenario = generate_training(TrainingRequest {
    topic:      TrainingTopic::BluffSpot.into(),
    difficulty: DifficultyLevel::Intermediate,
    rng_seed:   Some(42), // deterministic; use None for entropy
    text_style: TextStyle::Technical,
});

for ans in &scenario.answers {
    let mark = if ans.is_correct { "+" } else { " " };
    println!("[{mark}] {} — {}", ans.id, ans.text);
    println!("    {}", ans.explanation);
}
```

---

## Architecture

```
src/
  lib.rs                          crate root — re-exports the public API
  tests.rs                        49 unit tests (determinism, invariants, per-topic, street selector)
  training_engine/
    mod.rs                        module declarations + re-exports
    models.rs                     all shared types (Card, Position, TrainingScenario, ...)
    deck.rs                       52-card deck, Fisher-Yates shuffle, deterministic dealing
    evaluator.rs                  board texture, draw classification, pot odds, hand strength
    helpers.rs                    shared builders (deal, hand_str, board_str, answer, scenario)
    generator.rs                  generate_training() — single entry point, dispatches to topics
    topics/
      mod.rs
      preflop.rs                  T1 PF-, T5 IC-, T9 AL-, T11 SQ-, T12 BD-  (5 topics)
      flop.rs                     T2 CB-, T3 PO-, T7 CR-, T8 SB-, T13 3B-   (5 topics)
      turn.rs                     T6 TB-, T15 PB-, T16 DC-                   (3 topics)
      river.rs                    T4 BL-, T10 RV-, T14 RF-                   (3 topics)
```

**How a scenario is generated:**

1. `generate_training()` in `generator.rs` creates a deterministic RNG from the
   request seed, generates a unique scenario ID, and dispatches to the correct
   topic generator.
2. The topic generator shuffles a deck, deals hero cards and the board, classifies
   the situation (hand strength, board texture, draw type, etc.), determines the
   correct answer based on poker strategy, and builds dynamic explanations for
   every option.
3. Shared helpers in `helpers.rs` handle the boilerplate: dealing, string
   formatting, answer construction, and final scenario assembly.
4. Analysis primitives in `evaluator.rs` (board texture, draw equity, hand
   classification) are shared across all topics — never duplicated.

---

## API Reference

### `generate_training(request: TrainingRequest) -> TrainingScenario`

The single public entry point. Accepts a `TrainingRequest` and returns a fully-built
`TrainingScenario`.

**`TrainingRequest` fields**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `topic` | `TopicSelector` | *(required)* | What to drill — a specific `TrainingTopic` or a `Street` (use `.into()`) |
| `difficulty` | `DifficultyLevel` | `Beginner` | `Beginner`, `Intermediate`, or `Advanced` |
| `rng_seed` | `Option<u64>` | `None` | `Some(seed)` for deterministic output; `None` for entropy |
| `text_style` | `TextStyle` | `Simple` | `Simple` (plain English) or `Technical` (poker jargon) |

Only `topic` is required. Use `TrainingRequest::new(topic)` for the shortest form — it
accepts both `TrainingTopic` and `Street` directly (no `.into()` needed).

**`TopicSelector` variants**

| Variant | Example | Behaviour |
|---------|---------|-----------|
| `Topic(TrainingTopic)` | `TrainingTopic::BluffSpot.into()` | Generate that exact topic |
| `Street(Street)` | `Street::Flop.into()` | Pick a random topic from the street |

**`Street` variants**: `Preflop`, `Flop`, `Turn`, `River`

**`TrainingScenario` fields**

| Field | Type | Description |
|-------|------|-------------|
| `scenario_id` | `String` | Unique ID with topic prefix, e.g. `"PF-3A1C8F02"` |
| `topic` | `TrainingTopic` | The topic this scenario belongs to |
| `branch_key` | `String` | Logical decision branch, stable across seeds — use for progress tracking |
| `table_setup` | `TableSetup` | Cards, board, positions, stack/pot sizes |
| `question` | `String` | The question posed to the player |
| `answers` | `Vec<AnswerOption>` | All answer choices (exactly one has `is_correct: true`) |

**Invariants guaranteed by the engine**

- Exactly **one** `AnswerOption` has `is_correct: true` per scenario.
- Hero hole cards never appear on the board.
- Board cards are always unique (no duplicates).
- The same `rng_seed` always produces the same scenario (deterministic).

---

## Training Topics

Use `Street::X.into()` to get a random topic from a street, or `TrainingTopic::X.into()`
for an exact topic.

| Street | # | Topic | Enum Variant | ID Prefix |
|--------|---|-------|-------------|-----------|
| **Preflop** | 1 | [Preflop Decision](topics/01_preflop_decision.md) | `PreflopDecision` | `PF-` |
| | 5 | [ICM & Tournament Decision](topics/05_icm_tournament_decision.md) | `ICMAndTournamentDecision` | `IC-` |
| | 9 | [Anti-Limper Isolation](topics/09_anti_limper_isolation.md) | `AntiLimperIsolation` | `AL-` |
| | 11 | [Squeeze Play](topics/11_squeeze_play.md) | `SqueezePlay` | `SQ-` |
| | 12 | [Big Blind Defense](topics/12_big_blind_defense.md) | `BigBlindDefense` | `BD-` |
| **Flop** | 2 | [Postflop Continuation Bet](topics/02_postflop_continuation_bet.md) | `PostflopContinuationBet` | `CB-` |
| | 3 | [Pot Odds & Equity](topics/03_pot_odds_and_equity.md) | `PotOddsAndEquity` | `PO-` |
| | 7 | [Check-Raise Spot](topics/07_check_raise_spot.md) | `CheckRaiseSpot` | `CR-` |
| | 8 | [Semi-Bluff Decision](topics/08_semi_bluff_decision.md) | `SemiBluffDecision` | `SB-` |
| | 13 | [3-Bet Pot C-Bet](topics/13_three_bet_pot_cbet.md) | `ThreeBetPotCbet` | `3B-` |
| **Turn** | 6 | [Turn Barrel Decision](topics/06_turn_barrel_decision.md) | `TurnBarrelDecision` | `TB-` |
| | 15 | [Turn Probe Bet](topics/15_turn_probe_bet.md) | `TurnProbeBet` | `PB-` |
| | 16 | [Delayed C-Bet](topics/16_delayed_cbet.md) | `DelayedCbet` | `DC-` |
| **River** | 4 | [Bluff Spot](topics/04_bluff_spot.md) | `BluffSpot` | `BL-` |
| | 10 | [River Value Bet](topics/10_river_value_bet.md) | `RiverValueBet` | `RV-` |
| | 14 | [River Call or Fold](topics/14_river_call_or_fold.md) | `RiverCallOrFold` | `RF-` |


---

## Difficulty Levels

| Level | Stack depth range | Bet sizes | Who it's for |
|-------|------------------|-----------|-------------|
| `Beginner` | Fixed ~100 BB | Narrow, predictable | New players learning the fundamentals |
| `Intermediate` | 40–150 BB | Moderate variance | Players comfortable with basics |
| `Advanced` | 15–300 BB | Full variance | Players studying edge cases and deep/short-stack play |

---

## `branch_key` — Progress Tracking

Every scenario includes a `branch_key` that identifies the logical decision branch
regardless of which specific cards were dealt. Example values:

```
OpenRaise:premium:IP     ← preflop open, premium hand, in-position
FacingOpen:marginal:OOP  ← facing a raise, marginal hand, out-of-position
Dry:RangeAdv             ← c-bet spot, dry board, hero has range advantage
FlushDraw:Call           ← pot-odds spot, flush draw, call is correct
```

Use `branch_key` to track which decision types a student has mastered. The key is
stable across different seeds — you can always regenerate a specific branch for
targeted practice.

---

## Text Style

The `text_style` field on `TrainingRequest` controls the language used in the `question` string
and all `AnswerOption.explanation` strings. The game logic — which answer is correct, what cards
are dealt, bet sizes — is identical in both modes. Only the wording changes.

| Style | Audience | Description |
|-------|----------|-------------|
| `TextStyle::Simple` | Beginners | Plain English with no poker jargon. Concepts like SPR become "stack relative to pot". **This is the default.** |
| `TextStyle::Technical` | Experienced players | Standard poker terminology: SPR, EV, fold equity, c-bet, range advantage, GTO, pot odds, etc. |

**Example — Simple mode:**

```
Q: You're on the button with a missed flush draw and checked to on the river.
   The pot is 20 chips. Do you bluff or give up?

A: Bet large (15 chips) — With a busted draw and a lot of chips behind, your
   opponent can't be sure you missed. Betting large forces them to fold often enough.
```

**Example — Technical mode:**

```
Q: BTN, missed FD, checked to on river. Pot 20, SPR 4.2. Bluff or check back?

A: Bet large (15, ~75% pot) — With high fold equity, sufficient SPR, and a
   polarised range, a large barrel generates positive EV. Villain cannot
   profitably call without a strong hand given the sizing.
```

Both examples represent the same hand situation — only the text style differs.

---

## Glossary

| Term | Definition |
|------|-----------|
| **BB** | Big blind — the base unit for bet sizing in cash games |
| **c-bet** | Continuation bet — a bet on the flop by the preflop aggressor |
| **Equity** | Probability of winning the pot at showdown |
| **EV** | Expected value — long-run average profit of an action |
| **Fold equity** | Extra EV earned when a bet forces villain to fold hands they would otherwise win |
| **ICM** | Independent Chip Model — converts tournament chips to prize-money equity |
| **IP** | In position — acts last postflop |
| **OESD** | Open-ended straight draw — eight outs to complete a straight on either end |
| **OOP** | Out of position — acts first postflop |
| **Pot odds** | Ratio of the call amount to the total pot after calling |
| **SPR** | Stack-to-pot ratio: remaining stack ÷ pot |
