# poker_drill_gen — Project Instructions

## Build & Test
```bash
cargo build
cargo test
cargo run --example demo
```

> Note: `cargo` must be available in PATH. On Windows with Claude Code, invoke via bash shell.

---

## Project Overview
Rust library crate that generates randomized poker training scenarios.
Single external dependency: `rand = "0.8"`.

**Public API:**
```rust
pub fn generate_training(request: TrainingRequest) -> TrainingScenario
```
- `rng_seed: Some(u64)` → deterministic output (used by tests)
- `rng_seed: None` → entropy-based

---

## Module Layout
```
src/
  lib.rs                          ← crate root; re-exports; mod tests
  tests.rs                        ← all unit tests (#[cfg(test)])
  training_engine/
    mod.rs                        ← pub re-exports + sub-mod declarations
    models.rs                     ← all shared types (Card, Position, TrainingScenario, …)
    deck.rs                       ← Deck struct + Fisher-Yates shuffle
    evaluator.rs                  ← board_texture, pot-odds math, draw equity helpers
    generator.rs                  ← generate_training() dispatch + make_scenario_id()
    topics/
      mod.rs
      preflop.rs        (PF-)
      postflop.rs       (CB-)
      pot_odds.rs       (PO-)
      bluff.rs          (BL-)
      icm.rs            (IC-)
      turn_barrel.rs    (TB-)
      check_raise.rs    (CR-)
      semi_bluff.rs     (SB-)
      anti_limper.rs    (AL-)
examples/
  demo.rs
docs/
  README.md
  topics/  ← one .md per topic (01_preflop_decision.md … 09_anti_limper_isolation.md)
```

---

## Key Design Conventions
- Each topic module has a single public function:
  `pub fn generate<R: Rng>(rng: &mut R, difficulty: DifficultyLevel, scenario_id: String) -> TrainingScenario`
- **Single correct answer invariant:** use a `correct: &str` ID (`"A"`, `"B"`, or `"C"`) and match it to `AnswerOption.is_correct`. Never mark multiple answers correct.
- Explanations are dynamically formatted strings — not static templates.
- `board_texture()` in `evaluator.rs` drives c-bet sizing choices in postflop topics.
- `Suit` has no numeric repr — use `suit_index(s: Suit) -> usize` with an explicit `match` to convert to array index (not `s as usize`).

---

## Training Topics
| # | Enum Variant | ID Prefix | Street | Core Decision |
|---|---|---|---|---|
| 1 | `PreflopDecision` | `PF-` | Preflop | Open-raise vs fold |
| 2 | `PostflopContinuationBet` | `CB-` | Flop | C-bet size vs check |
| 3 | `PotOddsAndEquity` | `PO-` | Flop/Turn | Call vs fold with a draw |
| 4 | `BluffSpot` | `BL-` | River | Bluff vs check/fold |
| 5 | `ICMAndTournamentDecision` | `IC-` | Preflop | Push/fold vs ICM pressure |
| 6 | `TurnBarrelDecision` | `TB-` | Turn | Double-barrel vs check back |
| 7 | `CheckRaiseSpot` | `CR-` | Flop | Check-raise vs check-call vs fold (OOP) |
| 8 | `SemiBluffDecision` | `SB-` | Flop | Raise vs call vs fold with draw |
| 9 | `AntiLimperIsolation` | `AL-` | Preflop | Iso-raise vs overlimp vs fold |

---

## Hand Classification (5-category)
Used in `preflop.rs` and inlined in `anti_limper.rs`:
- **Premium**: AA, KK, QQ, AKs
- **Strong**: JJ, TT, AQo, AKo, AQs
- **Playable**: 99–77, AJs, KQs, suited connectors
- **Marginal**: 66–22, KJo, weak aces
- **Trash**: everything else

---

## Test Coverage (tests.rs)
- Determinism: same seed → identical scenario
- One-correct-answer invariant: exactly 1 `is_correct == true` per scenario
- Deck integrity: hero hand cards not on board; board cards unique
- Per-topic sanity: board card count, game type, hero position, bet presence

To add a new topic: extend `all_topics()` in `tests.rs` and add a per-topic sanity test.
