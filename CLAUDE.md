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
Dependencies: `rand = "0.8"`, `serde = { version = "1", features = ["derive"] }`.

**Public API:**
```rust
pub fn generate_training(request: TrainingRequest) -> TrainingScenario
```
- `TrainingRequest::new(topic)` — minimal constructor, accepts `TrainingTopic` or `Street`
- Only `topic` is required; `difficulty` (Beginner), `rng_seed` (None), `text_style` (Simple) have defaults
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
    evaluator.rs                  ← board_texture, pot-odds math, draw equity, hand classification, suit_index, DrawType
    helpers.rs                    ← shared builder functions (deal, hand_str, board_str, styled, answer, heads_up, scenario)
    generator.rs                  ← generate_training() dispatch + make_scenario_id()
    topics/
      mod.rs
      preflop.rs                  ← PF-, IC-, AL-, SQ-, BD- (5 preflop topics)
      flop.rs                     ← CB-, PO-, CR-, SB-, 3B- (5 flop topics)
      turn.rs                     ← TB-, PB-, DC- (3 turn topics)
      river.rs                    ← BL-, RV-, RF- (3 river topics)
examples/
  demo.rs                         ← TextStyle comparison + all 16 topics
  topics.rs                       ← one illustrated example per topic
docs/
  README.md                       ← API reference, architecture, glossary
  how_it_works.md                 ← beginner-friendly site copy
  topics/                         ← one .md per topic (01_preflop_decision.md … 16_delayed_cbet.md)
```

---

## Key Design Conventions
- Topics are grouped by street into 4 files (preflop.rs, flop.rs, turn.rs, river.rs).
  Each public function follows: `pub fn generate_<name><R: Rng>(rng, difficulty, scenario_id, text_style) -> TrainingScenario`
- Shared helpers in `helpers.rs` eliminate boilerplate: `deal()`, `hand_str()`, `board_str()`, `styled()`, `answer()`, `heads_up()`, `scenario()`
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
| 10 | `RiverValueBet` | `RV-` | River | Value bet sizing vs check |
| 11 | `SqueezePlay` | `SQ-` | Preflop | Squeeze vs call vs fold |
| 12 | `BigBlindDefense` | `BD-` | Preflop | 3-bet vs call vs fold from BB |
| 13 | `ThreeBetPotCbet` | `3B-` | Flop | C-bet sizing in 3-bet pots |
| 14 | `RiverCallOrFold` | `RF-` | River | Call vs fold vs raise facing river bet |
| 15 | `TurnProbeBet` | `PB-` | Turn | Probe bet sizing OOP after check-through |
| 16 | `DelayedCbet` | `DC-` | Turn | Delayed c-bet sizing IP after flop check-back |

---

## Hand Classification (5-category)
Defined in `evaluator.rs` (`HandCategory` enum + `classify_hand()`); used by `preflop.rs`:
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
