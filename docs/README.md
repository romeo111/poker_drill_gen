# poker_drill_gen — Documentation

`poker_drill_gen` is a Rust library that generates randomised poker training scenarios.
Each scenario presents a concrete hand situation, a multiple-choice question, and
a per-option explanation so players understand the *why* behind each decision.

---

## Quick Start

```rust
use poker_drill_gen::{generate_training, DifficultyLevel, TrainingRequest, TrainingTopic};

let scenario = generate_training(TrainingRequest {
    topic:      TrainingTopic::PreflopDecision,
    difficulty: DifficultyLevel::Beginner,
    rng_seed:   Some(42), // deterministic; use None for entropy
});

println!("{}", scenario.question);
for ans in &scenario.answers {
    let mark = if ans.is_correct { "✓" } else { " " };
    println!("[{mark}] {} — {}", ans.id, ans.text);
    println!("    {}", ans.explanation);
}
```

---

## API Reference

### `generate_training(request: TrainingRequest) -> TrainingScenario`

The single public entry point. Accepts a `TrainingRequest` and returns a fully-built
`TrainingScenario`.

**`TrainingRequest` fields**

| Field | Type | Description |
|-------|------|-------------|
| `topic` | `TrainingTopic` | Which of the 15 topics to generate |
| `difficulty` | `DifficultyLevel` | `Beginner`, `Intermediate`, or `Advanced` |
| `rng_seed` | `Option<u64>` | `Some(seed)` for deterministic output; `None` for entropy |

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

| # | Topic | Street | Enum Variant | Scenario ID Prefix |
|---|-------|--------|-------------|--------------------|
| 1 | [Preflop Decision](topics/01_preflop_decision.md) | Preflop | `PreflopDecision` | `PF-` |
| 2 | [Postflop Continuation Bet](topics/02_postflop_continuation_bet.md) | Flop | `PostflopContinuationBet` | `CB-` |
| 3 | [Pot Odds & Equity](topics/03_pot_odds_and_equity.md) | Flop | `PotOddsAndEquity` | `PO-` |
| 4 | [Bluff Spot](topics/04_bluff_spot.md) | River | `BluffSpot` | `BL-` |
| 5 | [ICM & Tournament Decision](topics/05_icm_tournament_decision.md) | Preflop | `ICMAndTournamentDecision` | `IC-` |
| 6 | [Turn Barrel Decision](topics/06_turn_barrel_decision.md) | Turn | `TurnBarrelDecision` | `TB-` |
| 7 | [Check-Raise Spot](topics/07_check_raise_spot.md) | Flop | `CheckRaiseSpot` | `CR-` |
| 8 | [Semi-Bluff Decision](topics/08_semi_bluff_decision.md) | Flop | `SemiBluffDecision` | `SB-` |
| 9 | [Anti-Limper Isolation](topics/09_anti_limper_isolation.md) | Preflop | `AntiLimperIsolation` | `AL-` |
| 10 | [River Value Bet](topics/10_river_value_bet.md) | River | `RiverValueBet` | `RV-` |
| 11 | [Squeeze Play](topics/11_squeeze_play.md) | Preflop | `SqueezePlay` | `SQ-` |
| 12 | [Big Blind Defense](topics/12_big_blind_defense.md) | Preflop | `BigBlindDefense` | `BD-` |
| 13 | [3-Bet Pot C-Bet](topics/13_three_bet_pot_cbet.md) | Flop | `ThreeBetPotCbet` | `3B-` |
| 14 | [River Call or Fold](topics/14_river_call_or_fold.md) | River | `RiverCallOrFold` | `RF-` |
| 15 | [Turn Probe Bet](topics/15_turn_probe_bet.md) | Turn | `TurnProbeBet` | `PB-` |


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
