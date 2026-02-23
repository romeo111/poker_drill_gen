# poker_drill_gen — Technical Specification

> **Purpose:** This document is a complete, language-agnostic specification of the
> `poker_drill_gen` engine. A developer with no access to the existing source code
> should be able to re-implement a fully compatible system from this document alone.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Data Types](#2-data-types)
3. [Module Structure](#3-module-structure)
4. [Core Subsystems](#4-core-subsystems)
   - 4.1 RNG and Scenario ID
   - 4.2 Deck
   - 4.3 Board Texture
   - 4.4 Equity Constants
   - 4.5 Pot Odds
5. [Topic Generators — Decision Logic](#5-topic-generators--decision-logic)
   - T1 Preflop Decision
   - T2 Postflop Continuation Bet
   - T3 Pot Odds & Equity
   - T4 Bluff Spot
   - T5 ICM & Tournament Decision
   - T6 Turn Barrel Decision
   - T7 Check-Raise Spot
   - T8 Semi-Bluff Decision
   - T9 Anti-Limper Isolation
   - T10 River Value Bet
   - T11 Squeeze Play
   - T12 Big Blind Defense
   - T13 3-Bet Pot C-Bet
   - T14 River Call or Fold
   - T15 Turn Probe Bet
   - T16 Multiway Pot
6. [Hard Invariants](#6-hard-invariants)
7. [branch\_key Catalogue](#7-branch_key-catalogue)
8. [Test Requirements](#8-test-requirements)

---

## 1. System Overview

The engine exposes one function:

```
generate_training(request: TrainingRequest) -> TrainingScenario
```

It:
1. Seeds a PRNG (deterministic or entropy-based).
2. Draws a scenario ID from the PRNG.
3. Deals cards from a Fisher-Yates shuffled 52-card deck.
4. Applies topic-specific decision logic to determine the single correct answer.
5. Returns a fully-populated `TrainingScenario`.

The engine is **stateless** — no global state, no I/O, no network calls.

---

## 2. Data Types

### 2.1 Card Primitives

```
Rank: u8 in [2..=14]   (14 = Ace, 13 = King, 12 = Queen, 11 = Jack, 10 = Ten)
Rank symbol: 2..9 → "2".."9", 10 → "T", 11 → "J", 12 → "Q", 13 → "K", 14 → "A"

Suit: enum { Clubs, Diamonds, Hearts, Spades }
Suit display: Clubs → "c", Diamonds → "d", Hearts → "h", Spades → "s"

Card: { rank: Rank, suit: Suit }
Card display: "{rank_symbol}{suit}" → e.g. "Ah", "Tc", "2s"

⚠ Suit has NO numeric representation. Never cast Suit to an integer.
   To index into a 4-element array use an explicit match:
   Clubs=0, Diamonds=1, Hearts=2, Spades=3
```

### 2.2 Enums

```
GameType:
  CashGame | Tournament
  Display: "Cash Game" | "Tournament"

Position:
  UTG | UTG1 | UTG2 | LJ | HJ | CO | BTN | SB | BB
  Display: "UTG" | "UTG+1" | "UTG+2" | "Lojack" | "Hijack" | "Cutoff" | "Button"
           | "Small Blind" | "Big Blind"
  is_late(): true when CO or BTN

DifficultyLevel:
  Beginner | Intermediate | Advanced

TrainingTopic:
  PreflopDecision          → prefix "PF"
  PostflopContinuationBet  → prefix "CB"
  PotOddsAndEquity         → prefix "PO"
  BluffSpot                → prefix "BL"
  ICMAndTournamentDecision → prefix "IC"
  TurnBarrelDecision       → prefix "TB"
  CheckRaiseSpot           → prefix "CR"
  SemiBluffDecision        → prefix "SB"
  AntiLimperIsolation      → prefix "AL"
```

### 2.3 Structs

```
PlayerState {
  seat:      u8        -- 1-indexed seat number
  position:  Position
  stack:     u32       -- chip count
  is_hero:   bool
  is_active: bool
}

TableSetup {
  game_type:      GameType
  hero_position:  Position
  hero_hand:      [Card; 2]
  board:          Vec<Card>   -- 0 (preflop), 3 (flop), 4 (turn), 5 (river)
  players:        Vec<PlayerState>
  pot_size:       u32
  current_bet:    u32         -- 0 if villain checked; >0 if villain bet
}

AnswerOption {
  id:          String      -- "A", "B", "C", or "D"
  text:        String      -- short label (e.g. "Fold", "Raise to 3 BB")
  is_correct:  bool
  explanation: String      -- full-sentence explanation, never empty
}

TrainingRequest {
  topic:      TrainingTopic
  difficulty: DifficultyLevel
  rng_seed:   Option<u64>   -- Some → deterministic; None → entropy
}

TrainingScenario {
  scenario_id: String       -- e.g. "PF-3A1C8F02"
  topic:       TrainingTopic
  branch_key:  String       -- logical decision branch (see §7)
  table_setup: TableSetup
  question:    String
  answers:     Vec<AnswerOption>
}
```

All types must support JSON serialization/deserialization.

---

## 3. Module Structure

```
engine/
  mod.rs          -- re-exports; declares submodules
  models.rs       -- all shared types (§2)
  deck.rs         -- Deck struct (§4.2)
  evaluator.rs    -- board_texture, equity helpers, pot odds (§4.3–4.5)
  generator.rs    -- generate_training() dispatcher; make_scenario_id()
  topics/
    mod.rs
    preflop.rs    -- T1
    postflop.rs   -- T2
    pot_odds.rs   -- T3
    bluff.rs      -- T4
    icm.rs        -- T5
    turn_barrel.rs -- T6
    check_raise.rs -- T7
    semi_bluff.rs  -- T8
    anti_limper.rs -- T9
```

Each topic module exposes exactly one function:

```
generate(rng, difficulty, scenario_id) -> TrainingScenario
```

---

## 4. Core Subsystems

### 4.1 RNG and Scenario ID

**PRNG algorithm:** `rand::StdRng` (ChaCha12-based, from the `rand 0.8` crate).

```
if rng_seed == Some(seed):
    rng = StdRng::seed_from_u64(seed)
else:
    rng = StdRng::from_entropy()
```

**Scenario ID format:**

```
"{PREFIX}-{:08X}"
```

Where `{:08X}` is the next `u32` from the RNG, formatted as 8 uppercase hex digits.
The ID is consumed from the RNG **before** any other RNG usage (cards, parameters).

```
scenario_id = make_id(topic, rng)   ← consumes 1 u32 from rng
# then deal cards, sample parameters, etc.
```

### 4.2 Deck

Standard 52-card deck. Construction order: for each suit in
`[Clubs, Diamonds, Hearts, Spades]`, ranks `2..=14` in order.

**Shuffle:** Fisher-Yates (Knuth shuffle), iterating from index `len-1` down to `1`.
For each `i`, swap `cards[i]` with `cards[rng.gen_range(0..=i)]`.

**Deal:** Sequential; cursor advances by 1 per card. Panics if deck is exhausted.

### 4.3 Board Texture

```
has_flush_draw(board):
  any suit appears ≥ 2 times in board cards

has_straight_draw(board):
  collect unique ranks, sort ascending
  any consecutive pair in sorted ranks differs by ≤ 2
  (i.e. ranks[i+1] - ranks[i] <= 2)

board_texture(board):
  flush = has_flush_draw(board)
  straight = has_straight_draw(board)
  if flush AND straight  → Wet
  if flush OR straight   → SemiWet
  else                   → Dry
```

### 4.4 Equity Constants

Fixed approximate equities (Rule of 4 and 2):

| Draw type | 2 streets (flop) | 1 street (turn) |
|-----------|-----------------|-----------------|
| Combo draw (flush + straight) | 0.54 | 0.30 |
| Flush draw | 0.35 | 0.20 |
| Open-ended straight draw (OESD) | 0.32 | 0.17 |
| Gutshot | 0.17 | 0.09 |

### 4.5 Pot Odds

```
required_equity(call_amount, pot_before_call):
  return call_amount / (pot_before_call + call_amount)

required_fold_frequency(bet_size, pot_before_bet):
  return bet_size / (pot_before_bet + bet_size)
```

Both return a `f32` in `[0.0, 1.0]`. Return `0.0` if denominator is zero.

---

## 5. Topic Generators — Decision Logic

Each generator follows this pattern:
1. Deal hero hand (2 cards) and board (0–5 cards) from a freshly shuffled deck.
2. Sample stack/pot sizes based on difficulty.
3. Classify the situation (hand category, board texture, draw type, etc.).
4. Determine `correct` answer ID with a single-match decision table.
5. Build `branch_key` (deterministic from classification, not from RNG).
6. Build `question` string and one `AnswerOption` per choice.
7. Assemble and return `TrainingScenario`.

---

### T1 — Preflop Decision

**Game type:** CashGame
**Board cards:** 0
**Players:** All positions for 6-max or 9-max (randomly chosen)

#### Hand Classification (5 categories)

Input: two cards sorted descending by rank. `r1 >= r2`, `suited = (suit1 == suit2)`.

```
Pairs:
  r1 in {14,13,12} → Premium
  r1 in {11,10}    → Strong
  r1 in {7,8,9}    → Playable
  r1 <= 6          → Marginal

Non-pairs (r1, r2, suited):
  (14, 13, true)         → Premium
  (14, 13, false)        → Strong
  (14, 12, any)          → Strong
  (14, 11, true)         → Playable
  (14, r≥9, true)        → Playable
  (13, 12, true)         → Playable
  (13, 12, false)        → Marginal
  r1≥9 AND r1-r2 <= 1, suited → Playable   (suited connectors/one-gappers)
  r1 <= 9                → Trash
  anything else          → Marginal
```

#### Spot Selection (uniform 1-of-3)

```
0 → OpenRaise
1 → FacingOpen
2 → ThreeBetPot
```

#### Stack Sampling

```
Beginner:     80–120 BB (uniform)
Intermediate: 40–150 BB
Advanced:     15–300 BB
```

#### Correct Answer

```
OpenRaise:
  should_raise = (Premium OR Strong)
              OR (is_late(pos) AND (Playable OR Marginal))
  correct = "B" (Raise) if should_raise else "A" (Fold)
  "C" (Call/Limp) is always wrong

FacingOpen:
  Premium         → "C" (3-bet)
  Strong          → "C" (3-bet)
  Playable + late → "C" (3-bet)
  Playable + !late → "B" (call)
  Marginal        → "A" (fold)
  Trash + late AND stack ≥ 25 BB → "C" (3-bet bluff)
  Trash otherwise → "A" (fold)

ThreeBetPot:
  Premium   → "C" (4-bet)
  Strong    → "B" (call)
  Playable  → "B" (call)
  Marginal  → "A" (fold)
  Trash     → "A" (fold)
```

#### Sizing

```
open_size = 3 BB if stack ≥ 40 BB else 2 BB
raiser_size (FacingOpen) = same formula as open_size
three_bet = raiser_size * 3
four_bet = three_bet * 3
```

#### branch_key

```
OpenRaise:   "OpenRaise:{cat}:{IP|OOP}"     e.g. "OpenRaise:premium:IP"
FacingOpen:  "FacingOpen:{cat}:{IP|OOP}"
ThreeBetPot: "ThreeBetPot:{cat}"
```

---

### T2 — Postflop Continuation Bet

**Game type:** CashGame
**Board cards:** 3 (flop only)
**Hero position:** BTN or CO (50/50)
**Villain position:** BB

#### Range Advantage Flag

```
hero_has_range_adv = hero_pos.is_late() AND min(board_ranks) <= 8
```

#### Correct Answer

```
(Dry, range_adv=true)  → "B" (Bet small ~33% pot)
(Dry, range_adv=false) → "A" (Check)
(SemiWet)              → "C" (Bet large ~75% pot)
(Wet)                  → "C" (Bet large ~75% pot)
```

Answer "D" (Overbet ~125% pot) is always wrong.

#### Sizing in question text

```
small = pot / 3
large = pot * 3 / 4
overbet = pot * 5 / 4
```

#### Stack/Pot Sampling

```
Beginner:     stack=100 BB, pot=8–14 BB
Intermediate: stack=60–130 BB, pot=6–20 BB
Advanced:     stack=20–200 BB, pot=4–30 BB
```

All values in BB; multiply by `bb=2` for chips.

#### branch_key

```
(Dry, true)   → "Dry:RangeAdv"
(Dry, false)  → "Dry:NoRangeAdv"
(SemiWet, _)  → "SemiWet"
(Wet, _)      → "Wet"
```

---

### T3 — Pot Odds & Equity

**Game type:** CashGame
**Board cards:** 3 (flop)
**Hero position:** BB
**Villain position:** BTN
**Streets remaining:** 2 (always flop scenario)

#### Draw Classification (from actual board)

```
flush = has_flush_draw(board)
straight = has_straight_draw(board)
(true, true)   → ComboDraw
(true, false)  → FlushDraw
(false, true)  → OpenEndedStraight (OESD)
(false, false) → GutShot
```

#### Equity Used

Use the §4.4 constants with `streets_remaining=2`.
GutShot: `0.17` on flop.

#### Bet Size Sampling

```
Beginner:     pot=8–12 BB, bet_pct=0.50 (fixed)
Intermediate: pot=6–20 BB, bet_pct=uniform [0.33, 1.0]
Advanced:     pot=4–30 BB, bet_pct=uniform [0.25, 1.5]
bet_chips = round(pot * bet_pct)
```

#### Correct Answer

```
req = required_equity(bet_chips, pot)
actual = hero_equity(draw_type, streets=2)
should_call = actual >= req
"A" (Call) if should_call else "B" (Fold)
```

#### branch_key

```
"{DrawName}:{Call|Fold}"
DrawName: FlushDraw | OESD | ComboDraw | GutShot
e.g. "FlushDraw:Call", "GutShot:Fold"
```

---

### T4 — Bluff Spot

**Game type:** CashGame
**Board cards:** 5 (river)
**Hero position:** BTN (always)
**Villain position:** BB
**current_bet:** 0 (villain checked)

#### Bluff Archetype (uniform 1-of-3)

```
0 → MissedFlushDraw
1 → CappedRange
2 → OvercardBrick
```

#### SPR

```
spr = stack / pot
spr_bucket = "LowSPR" if spr < 2.0 else "HighSPR"
```

#### Bet Sizes

```
small_bet = round(pot * 0.40)
large_bet = round(pot * 0.75)
shove     = stack
```

Answer "D" (All-in shove) is always wrong.

#### Correct Answer

```
CappedRange         → "A" (Check — can't credibly represent nuts)
spr < 2.0           → "A" (Check — no fold equity)
otherwise           → "C" (Bet large ~75%)
```

"B" (Bet small) is never correct in the current engine.

#### Stack/Pot Sampling

```
Beginner:     pot=10–16 BB, stack=50 BB
Intermediate: pot=8–24 BB, stack=30–80 BB
Advanced:     pot=6–40 BB, stack=15–150 BB
```

#### branch_key

```
CappedRange:      "CappedRange"
MissedFlushDraw:  "MissedFlushDraw:{spr_bucket}"
OvercardBrick:    "OvercardBrick:{spr_bucket}"
```

---

### T5 — ICM & Tournament Decision

**Game type:** Tournament
**Board cards:** 0
**Hero position:** BTN (always)
**Villain position:** BB
**Answers:** A = "All-in" (shove), B = "Fold" (only 2 answers)
**Chip unit:** `bb = 100` (tournament chips; 100 = 1 BB)

#### Tournament Stage (uniform 1-of-4)

```
0 → EarlyLevels
1 → MiddleStages
2 → Bubble
3 → FinalTable
```

#### Push Threshold (stack BB ≤ threshold → shove)

```
EarlyLevels:  20
MiddleStages: 15
Bubble:       10
FinalTable:   12
```

#### ICM Risk Premium (displayed in explanation)

```
EarlyLevels:  3%
MiddleStages: 8%
Bubble:       20%
FinalTable:   15%
```

#### Stack Sampling (hero_stack_bb)

```
Beginner:     6–18 BB
Intermediate: 4–25 BB
Advanced:     3–30 BB
villain_stack_bb: 20–60 BB (any difficulty)
```

#### Players Remaining

```
EarlyLevels:  60–120
MiddleStages: 25–60
Bubble:       10–18
FinalTable:   3–9
```

#### Paid Spots

```
paid_spots = ceil(players_remaining * 0.15)
```

#### Correct Answer

```
should_push = hero_stack_bb <= push_threshold_bb(stage)
"A" (All-in) if should_push else "B" (Fold)
```

#### Pot Size in TableSetup

```
pot = bb + bb/2    (= 150 chips = 1.5 BB; represents SB+BB estimate)
```

#### branch_key

```
"{EarlyLevels|Middle|Bubble|FinalTable}:{Push|Fold}"
e.g. "Bubble:Fold", "FinalTable:Push"
```

Stage label in branch_key: `Early` | `Middle` | `Bubble` | `FinalTable`

---

### T6 — Turn Barrel Decision

**Game type:** CashGame
**Board cards:** 4 (3 flop + 1 turn; all 4 in `board` field)
**Hero position:** BTN or CO (50/50)
**Villain position:** BB
**current_bet:** 0 (villain checked)

#### Turn Card Classification

```
classify_turn(flop, turn_card):
  1. Flush complete:
     count = flop cards sharing suit with turn_card
     if count >= 2 → DrawComplete

  2. Straight complete:
     ranks = [flop ranks..., turn_card rank], sorted, deduplicated
     for each window of 4 consecutive ranks:
       if ranks[3] - ranks[0] <= 4 → DrawComplete

  3. Broadway: turn_card.rank >= 10 → ScareBroadway

  4. else → Blank
```

#### Flop Texture

Used only for `Blank` turn: classify the original 3-card flop with `board_texture()`.

#### Correct Answer

```
DrawComplete               → "A" (Check)
ScareBroadway              → "C" (Bet large ~80% pot)
Blank + (Wet or SemiWet)   → "B" (Bet medium ~50% pot)
Blank + Dry                → "A" (Check)
```

#### Bet Sizes in Question Text

```
medium = pot / 2
large  = pot * 4 / 5
```

#### Stack/Pot Sampling

```
Beginner:     stack=100 BB, pot=14–22 BB
Intermediate: stack=50–130 BB, pot=10–28 BB
Advanced:     stack=25–200 BB, pot=8–40 BB
```

#### branch_key

```
DrawComplete    → "DrawComplete"
ScareBroadway   → "ScareBroadway"
Blank + Wet/Semi → "Blank:Wet"
Blank + Dry     → "Blank:Dry"
```

---

### T7 — Check-Raise Spot

**Game type:** CashGame
**Board cards:** 3 (flop)
**Hero position:** BB (always, OOP)
**Villain position:** BTN
**current_bet:** villain_bet (always > 0)

#### Board Favour Classification

```
rank_sum = sum of all 3 board card ranks
BBFavorable if rank_sum <= 20
IPFavorable if rank_sum > 20
```

#### Hand Interaction Classification

Priority order:

```
1. Check flush draw: hero card shares suit with a board card AND board has flush draw
   (i.e. board already has 2+ same-suit cards; hero card adds to same suit)
2. Check straight draw: board has straight draw AND any hero card rank is within 3
   of any board card rank
3. If flush OR straight → Draw
4. Else if any hero card rank matches any board card rank → Strong
5. Else → Weak
```

#### Combo Draw Flag

```
combo = hero_has_flush_draw(hand, board) AND hero_has_straight_draw(hand, board)
```

#### Villain Bet Size

```
villain_bet = floor(pot * uniform(50..=70) / 100)
villain_bet = max(villain_bet, bb)
cr_size = villain_bet * 5 / 2   (2.5×)
```

#### Correct Answer

```
(BBFavorable, Strong)       → "C" (Check-raise)
(any, Draw) AND combo=true  → "C" (Check-raise semi-bluff)
(IPFavorable, Weak)         → "A" (Fold)
all other combinations      → "B" (Check-call)
```

#### branch_key

```
(BBFavorable, Strong, _)       → "BBFav:Strong"
(BBFavorable, Draw, combo)     → "BBFav:ComboDraw"
(BBFavorable, Draw, !combo)    → "BBFav:Draw"
(BBFavorable, Weak, _)         → "BBFav:Weak"
(IPFavorable, Strong, _)       → "IPFav:Strong"
(IPFavorable, Draw, combo)     → "IPFav:ComboDraw"
(IPFavorable, Draw, !combo)    → "IPFav:Draw"
(IPFavorable, Weak, _)         → "IPFav:Weak"
```

---

### T8 — Semi-Bluff Decision

**Game type:** CashGame
**Board cards:** 3 (flop)
**current_bet:** villain_bet (> 0)

#### Hero Position

```
50% chance: BTN (IP)
50% chance: BB (OOP)
Villain: CO if hero is BTN; BB if hero is CO
```

#### Draw Classification (from board)

```
flush = has_flush_draw(board)
straight = has_straight_draw(board)
(true, true)   → ComboDraw
(true, false)  → FlushDraw
(false, true)  → OESD
(false, false) → GutShot
```

#### Equity (flop, §4.4)

```
ComboDraw: 0.54
FlushDraw: 0.35
OESD:      0.32
GutShot:   0.17
```

#### Villain Bet / Raise Size

```
villain_bet_pct: uniform 50–75
villain_bet = floor(pot * villain_bet_pct / 100), minimum bb
raise_size = villain_bet * 5 / 2   (2.5×)
```

#### Correct Answer

```
ComboDraw                       → "C" (Raise)
OESD AND stack_bb >= 40         → "C" (Raise)
FlushDraw (any position)        → "B" (Call)
OESD AND stack_bb < 40          → "B" (Call)
GutShot                         → "A" (Fold)
```

Note: position does not change the correct answer in this engine — all flush draws
call, all gutshots fold, all combo draws raise.

#### Stack/Pot Sampling

```
Beginner:     stack=60 BB, pot=8–14 BB
Intermediate: stack=35–120 BB, pot=6–20 BB
Advanced:     stack=20–200 BB, pot=4–30 BB
```

#### branch_key

```
ComboDraw              → "ComboDraw"
FlushDraw              → "FlushDraw"
OESD + stack >= 40     → "OESD:Deep"
OESD + stack < 40      → "OESD:Short"
GutShot                → "GutShot"
```

---

### T9 — Anti-Limper Isolation

**Game type:** CashGame
**Board cards:** 0
**current_bet:** bb (the limp amount)

#### Hero Position (uniform 1-of-3)

```
0 → CO
1 → BTN
2 → SB
```

#### Limper Count

```
uniform 1–3
```

#### Pot Size

```
pot = bb + (bb / 2) + (bb * limper_count)
    = 2 + 1 + (2 * limper_count)
```

#### Iso-Raise Size

```
1 limper → 4 BB (= 8 chips at bb=2)
2 limpers → 5 BB
3+ limpers → 6 BB
```

#### Hand Classification

Same 5-category logic as T1 (see §T1 Hand Classification).

#### In-Position Flag

```
ip = (hero_pos == CO OR hero_pos == BTN)
```

#### Correct Answer

```
Premium OR Strong         → "C" (Iso-raise)
Playable AND ip           → "C" (Iso-raise)
Playable AND NOT ip (SB)  → "B" (Overlimp / call)
Marginal OR Trash         → "A" (Fold)
```

#### Stack Sampling

```
Beginner:     60–120 BB
Intermediate: 30–150 BB
Advanced:     15–200 BB
```

#### branch_key

```
Premium   → "Premium"
Strong    → "Strong"
Playable + ip   → "Playable:IP"
Playable + !ip  → "Playable:OOP"
Marginal  → "Marginal"
Trash     → "Trash"
```

---

### T10 River Value Bet (`RV-`)

**Street:** River (5 board cards).
**Hero position:** BTN. **Villain position:** BB.

#### Enum

```
HandStrength: Nuts | Strong | Medium
```

#### Scenario Parameters

```
Beginner:     pot 10–18 BB, stack 60 BB
Intermediate: pot 8–28 BB,  stack 30–80 BB
Advanced:     pot 6–40 BB,  stack 15–150 BB
```

Bet sizings: `small = pot × 0.33`, `large = pot × 0.75`, `overbet = pot × 1.25`.

#### Decision Logic

```
Nuts   → "D" (Overbet, ~125% pot)
Strong → "C" (Large bet, ~75% pot)
Medium → "A" (Check)
```

#### Answer Options

```
A  Check                    — correct for Medium
B  Bet small (~33% pot)     — always wrong
C  Bet large (~75% pot)     — correct for Strong
D  Overbet (~125% pot)      — correct for Nuts
```

`current_bet = 0` (villain checked to hero).

#### branch_key

```
Nuts   → "Nuts:Overbet"
Strong → "Strong:LargeBet"
Medium → "Medium:Check"
```

---

### T11 Squeeze Play (`SQ-`)

**Street:** Preflop (no board cards).
**Hero position:** BTN. **Opener position:** UTG.

#### Enum

```
HoleStrength: Premium | Speculative | Weak
```

#### Scenario Parameters

```
callers:
  Beginner:     1
  Intermediate: 1–2
  Advanced:     1–3

open size:
  Beginner:     3 BB
  Intermediate: 2–4 BB
  Advanced:     2–5 BB

stack:
  Beginner:     100 BB
  Intermediate: 60–120 BB
  Advanced:     25–150 BB
```

`pot_bb = open_bb + callers × open_bb + 1`
`squeeze_bb = open_bb × 3 + callers × open_bb`

#### Decision Logic

```
Premium     → "C" (Squeeze)
Speculative → "B" (Call)
Weak        → "A" (Fold)
```

#### Answer Options

```
A  Fold               — correct for Weak
B  Call (open_bb)     — correct for Speculative
C  Squeeze (~squeeze) — correct for Premium
```

`current_bet = open_bb × bb` (hero faces the open raise).

#### branch_key

```
Premium     → "Premium:Squeeze"
Speculative → "Speculative:Call"
Weak        → "Weak:Fold"
```

---

### T12 Big Blind Defense (`BD-`)

**Street:** Preflop (no board cards).
**Hero position:** BB. **Villain position:** UTG | CO | BTN (random).

#### Enum

```
DefenseStrength: Strong | Playable | Weak
```

#### Scenario Parameters

```
raise_bb:
  Beginner:     3 BB
  Intermediate: 2–4 BB
  Advanced:     2–5 BB

stack:
  Beginner:     100 BB
  Intermediate: 60–120 BB
  Advanced:     25–150 BB
```

`pot_bb = raise_bb + 1` (BB already posted)
`three_bet_bb = raise_bb × 3 + 1`

#### Decision Logic

```
Strong   → "C" (3-bet)
Playable → "B" (Call)
Weak     → "A" (Fold)
```

#### Answer Options

```
A  Fold                  — correct for Weak
B  Call (raise_bb)       — correct for Playable
C  3-bet (~three_bet_bb) — correct for Strong
```

`current_bet = raise_bb × bb`.

#### branch_key

```
Strong   → "Strong:ThreeBet"
Playable → "Playable:Call"
Weak     → "Weak:Fold"
```

---

### T13 3-Bet Pot C-Bet (`3B-`)

**Street:** Flop (3 board cards).
**Hero position:** BTN (the 3-better). **Villain position:** BB.

#### Enums

```
BoardTexture: Dry | Wet
FlopStrength: Strong | Weak
```

#### Scenario Parameters

```
pot_bb (3-bet pot is larger):
  Beginner:     10–14 BB
  Intermediate: 8–18 BB
  Advanced:     6–22 BB

stack:
  Beginner:     100 BB
  Intermediate: 50–100 BB
  Advanced:     30–150 BB
```

`small_bet = pot × 0.33`, `large_bet = pot × 0.67`.

#### Decision Logic

```
(Dry,  Strong) → "B" (Small c-bet ~33%)
(Wet,  Strong) → "C" (Large c-bet ~67%)
(Dry,  Weak)   → "A" (Check)
(Wet,  Weak)   → "A" (Check)
```

#### Answer Options

```
A  Check back         — correct for (any, Weak)
B  C-bet small (~33%) — correct for (Dry, Strong)
C  C-bet large (~67%) — correct for (Wet, Strong)
```

`current_bet = 0` (villain checks to hero).

#### branch_key

```
(Dry, Strong) → "Dry:Strong:SmallCbet"
(Wet, Strong) → "Wet:Strong:LargeCbet"
(Dry, Weak)   → "Dry:Weak:Check"
(Wet, Weak)   → "Wet:Weak:Check"
```

---

### T14 River Call or Fold (`RF-`)

**Street:** River (5 board cards).
**Hero position:** BTN. **Villain position:** BB (bets into hero).

#### Enums

```
HandStrength: Strong | Marginal | Weak
BetSize:      Small | Standard | Large
```

#### Scenario Parameters

```
pot_bb:
  Beginner:     10–20 BB
  Intermediate: 8–28 BB
  Advanced:     6–40 BB

stack:
  Beginner:     80 BB
  Intermediate: 30–100 BB
  Advanced:     15–150 BB
```

Villain bet amounts:
```
Small    = pot × 0.33
Standard = pot × 0.67
Large    = pot × 1.00
```

Required equity formula:
```
required_equity = villain_bet / (pot + 2 × villain_bet)
```

#### Paired Scenario Selection

```
rng.gen_range(0..3):
  0 → (Strong,   Small)    — raise for value
  1 → (Marginal, Standard) — call
  2 → (Weak,     Large)    — fold
```

#### Decision Logic

```
(Strong,   Small)    → "C" (Raise to ~2.5× villain_bet)
(Marginal, Standard) → "B" (Call)
(Weak,     Large)    → "A" (Fold)
```

#### Answer Options

```
A  Fold                  — correct for (Weak, Large)
B  Call (villain_bet)    — correct for (Marginal, Standard)
C  Raise to ~2.5× bet    — correct for (Strong, Small)
```

`current_bet = villain_bet`.

#### branch_key

```
(Strong,   Small)    → "Strong:SmallBet:Raise"
(Marginal, Standard) → "Marginal:StdBet:Call"
(Weak,     Large)    → "Weak:LargeBet:Fold"
```

---

### T15 Turn Probe Bet (`PB-`)

**Street:** Turn (4 board cards: flop + turn).
**Hero position:** BB (OOP). **Villain position:** BTN.

#### Enum

```
ProbeStrength: Strong | Medium | Weak
```

#### Scenario Parameters

```
pot_bb:
  Beginner:     6–14 BB
  Intermediate: 4–20 BB
  Advanced:     4–30 BB

stack:
  Beginner:     80 BB
  Intermediate: 40–100 BB
  Advanced:     20–150 BB
```

`small_probe = pot × 0.40`, `large_probe = pot × 0.70`.

#### Decision Logic

```
Strong → "C" (Probe large ~70%)
Medium → "B" (Probe small ~40%)
Weak   → "A" (Check)
```

#### Answer Options

```
A  Check                         — correct for Weak
B  Probe small (~40% pot)        — correct for Medium
C  Probe large (~70% pot)        — correct for Strong
```

`current_bet = 0` (hero acts first on the turn; flop was checked through).

#### branch_key

```
Strong → "Strong:ProbeLarge"
Medium → "Medium:ProbeSmall"
Weak   → "Weak:Check"
```

---

### T16 Multiway Pot (`MW-`)

**Street:** Flop (3 board cards).
**Hero position:** CO. **Opponents:** BTN + BB (+ SB, HJ at higher opponent counts).

#### Enum

```
MultiStrength: Strong | TopPair | Weak
```

#### Scenario Parameters

```
opponents:
  Beginner:     2
  Intermediate: 2–3
  Advanced:     2–4

pot_bb:
  Beginner:     8–16 BB
  Intermediate: 6–20 BB
  Advanced:     4–30 BB

stack:
  Beginner:     100 BB
  Intermediate: 50–120 BB
  Advanced:     20–150 BB
```

`small_bet = pot × 0.33`, `large_bet = pot × 0.67`.

#### Decision Logic

```
Strong  → "C" (Bet large ~67%)
TopPair → "B" (Bet small ~33%)
Weak    → "A" (Check)
```

#### Answer Options

```
A  Check               — correct for Weak
B  Bet small (~33%)    — correct for TopPair
C  Bet large (~67%)    — correct for Strong
```

`current_bet = 0` (hero acts first).

#### branch_key

```
Strong  → "Strong:BetLarge"
TopPair → "TopPair:BetSmall"
Weak    → "Weak:Check"
```

---

## 6. Hard Invariants

These must be true for every generated scenario, enforced by tests:

| # | Invariant |
|---|-----------|
| 1 | Exactly **one** `AnswerOption` has `is_correct = true` per scenario. |
| 2 | Hero hole cards **never appear** in the board (cards are distinct). |
| 3 | All board cards are **unique** (no duplicate cards). |
| 4 | Same `rng_seed` value always produces **identical** output (deterministic). |
| 5 | `scenario_id` starts with the topic prefix (e.g. `"PF-"`). |
| 6 | `branch_key` is non-empty. |
| 7 | Every `AnswerOption` has non-empty `text` and non-empty `explanation`. |
| 8 | Hero hand is always exactly **2 cards**. |
| 9 | Board card count matches the street: 0 (preflop), 3 (flop), 4 (turn), 5 (river). |

---

## 7. branch\_key Catalogue

| Topic | Possible branch_key values |
|-------|---------------------------|
| T1 Preflop | `OpenRaise:{cat}:{IP\|OOP}`, `FacingOpen:{cat}:{IP\|OOP}`, `ThreeBetPot:{cat}` |
| T2 C-bet | `Dry:RangeAdv`, `Dry:NoRangeAdv`, `SemiWet`, `Wet` |
| T3 Pot Odds | `{DrawName}:{Call\|Fold}` where DrawName ∈ {FlushDraw, OESD, ComboDraw, GutShot} |
| T4 Bluff | `CappedRange`, `MissedFlushDraw:{LowSPR\|HighSPR}`, `OvercardBrick:{LowSPR\|HighSPR}` |
| T5 ICM | `{Early\|Middle\|Bubble\|FinalTable}:{Push\|Fold}` |
| T6 Turn Barrel | `DrawComplete`, `ScareBroadway`, `Blank:Wet`, `Blank:Dry` |
| T7 Check-Raise | `{BBFav\|IPFav}:{Strong\|ComboDraw\|Draw\|Weak}` |
| T8 Semi-Bluff | `ComboDraw`, `FlushDraw`, `OESD:{Deep\|Short}`, `GutShot` |
| T9 Anti-Limper | `Premium`, `Strong`, `Playable:{IP\|OOP}`, `Marginal`, `Trash` |
| T10 River Value Bet | `Nuts:Overbet`, `Strong:LargeBet`, `Medium:Check` |
| T11 Squeeze Play | `Premium:Squeeze`, `Speculative:Call`, `Weak:Fold` |
| T12 BB Defense | `Strong:ThreeBet`, `Playable:Call`, `Weak:Fold` |
| T13 3B Pot C-Bet | `Dry:Strong:SmallCbet`, `Wet:Strong:LargeCbet`, `Dry:Weak:Check`, `Wet:Weak:Check` |
| T14 River Call/Fold | `Strong:SmallBet:Raise`, `Marginal:StdBet:Call`, `Weak:LargeBet:Fold` |
| T15 Turn Probe Bet | `Strong:ProbeLarge`, `Medium:ProbeSmall`, `Weak:Check` |
| T16 Multiway Pot | `Strong:BetLarge`, `TopPair:BetSmall`, `Weak:Check` |

`{cat}` in T1/T9: `premium` | `strong` | `playable` | `marginal` | `trash` (lowercase)

---

## 8. Test Requirements

A conforming implementation must pass tests in all of these groups:

### Determinism

- Same `rng_seed` → identical `scenario_id`, `question`, `branch_key`, answer IDs, `is_correct` flags.
- Different seeds across a range of 40+ pairs → fewer than 25% identical questions for any single topic.
- `rng_seed: None` (entropy) → valid scenario satisfying all structural invariants.

### Structural Invariants (all topics, multiple seeds)

- Exactly 1 correct answer per scenario.
- At least 2 answer options per scenario.
- Non-empty `text` and `explanation` on every answer option.
- `scenario_id` starts with correct prefix.
- `branch_key` non-empty and deterministic.
- Hero hand length = 2.
- All 3 difficulty levels produce valid scenarios.

### Deck Integrity (all topics, multiple seeds)

- Hero hand cards not in board.
- Board cards unique (no duplicates).

### Per-Topic (each topic, multiple seeds)

| Topic | Required board length | Required current_bet | Required game_type | Required hero_position |
|-------|-----------------------|----------------------|--------------------|-----------------------|
| T1 | 0 | — | CashGame | any |
| T2 | 3 | — | CashGame | BTN or CO |
| T3 | 3 | > 0 | CashGame | BB |
| T4 | 5 | — | CashGame | BTN |
| T5 | 0 | — | Tournament | BTN |
| T6 | 4 | — | CashGame | BTN or CO |
| T7 | 3 | > 0 | CashGame | BB |
| T8 | 3 | > 0 | CashGame | BTN or BB |
| T9 | 0 | — | CashGame | CO, BTN, or SB |
| T10 | 5 | 0 | CashGame | BTN |
| T11 | 0 | > 0 | CashGame | BTN |
| T12 | 0 | > 0 | CashGame | BB |
| T13 | 3 | 0 | CashGame | BTN |
| T14 | 5 | > 0 | CashGame | BTN |
| T15 | 4 | 0 | CashGame | BB |
| T16 | 3 | 0 | CashGame | CO |
