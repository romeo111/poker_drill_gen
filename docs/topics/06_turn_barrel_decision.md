# Topic 6 — Turn Barrel Decision

**Enum variant:** `TrainingTopic::TurnBarrelDecision`
**Scenario ID prefix:** `TB-`
**Street:** Turn
**Difficulty range:** Beginner → Advanced

---

## Core Principle

You raised preflop, c-bet the flop, and villain called. Now the turn card has arrived.
The question: do you **fire a second barrel** (double-barrel), or do you **check back**?

This is one of the highest-leverage postflop decisions in poker because:
- The pot is already substantial from the flop c-bet.
- Your story must be consistent with a two-street betting range.
- The turn card itself dramatically changes the calculus — some cards help you, some hurt.

---

## The Three Turn Card Types

The engine classifies each turn card into one of three categories:

| Category | Description | Example |
|----------|-------------|---------|
| **Blank** | Low card (≤ 9) that doesn't complete obvious draws | 2♠ on K♥ 7♣ 4♦ board |
| **ScareBroadway** | Ten or higher — hits the preflop raiser's late-position range hard | Q♦ on J♠ 8♥ 3♣ board |
| **DrawComplete** | Card that fills a flush or straight draw from the flop | 9♥ completing a flush on K♥ T♥ 2♣ |

---

## Decision Matrix

| Turn card type | Flop texture | Correct action |
|----------------|-------------|----------------|
| DrawComplete | Any | **Check** — draws completed, villain's hand improved |
| ScareBroadway | Any | **Bet ~80% pot** — scare card hits your range harder |
| Blank | Wet/Semi-Wet | **Bet ~50% pot** — charge remaining draws |
| Blank | Dry | **Check** — no value barreling air on dry board |

---

## Why Check When Draws Complete?

When the turn completes a flush or straight, villain's check-call range from the flop is
enriched with now-made hands. The caller who had a flush draw now has a flush. The
caller who had open-ended straight draw may have a straight.

Your bluff equity collapses:
- Draws that called the flop are now made hands.
- Villain's check-call range is now heavily weighted toward strong holdings.
- A barrel into completed draws has very low fold equity and risks being check-raised
  by value hands.

**Exception:** If you have the nut flush or a straight yourself, you can bet for value.
But the engine trains the default "you missed" scenario.

---

## Why Bet Big on ScareBroadway Cards?

A late-position preflop range (CO, BTN) is rich in Broadway cards: AK, AQ, AJ, AT, KQ,
KJ, QJ, and pocket pairs from TT to AA.

When a Queen, King, Jack, or Ten hits the turn:
- Your range contains many combinations of top pair, two pair, set, or strong draws.
- Villain's flop-calling range (often Jx, middle pairs, draws) is relatively capped
  against a new high card.
- A large turn bet is **range-credible** — villain cannot tell you don't have AQ, KQ,
  or a set.

**Size:** ~80% pot, because you want to apply maximum pressure when the card "hit" you.

---

## Why Bet Medium on Blanks (Wet Board)?

On a wet flop (T♦ 9♣ 6♥), villain's calling range contains many draws: 87, J8, Q8, flush
draws. A blank turn like 2♠ doesn't help those hands but doesn't hurt them either.

**Charging draws:** With one street to come (the river), your opponent's flush draw drops
from ~35% equity to ~20%. A 50% pot bet makes calling mathematically incorrect:

```
Required equity to call 50% pot = 50 / 150 = 33.3%
Flush draw equity (1 street) = ~20%
20% < 33.3% → fold is correct for villain
```

This is the turn's **denying equity** function: force villain to pay the wrong price
before the river eliminates the draw entirely.

---

## Why Check Blanks on Dry Boards?

On a dry board (K♠ 7♦ 2♣) with no draws, a blank turn like 3♥ changes nothing. Villain
called the flop with something — a pair, maybe two pair, maybe a slow-played hand. None
of those fold to a second barrel.

Without draws to charge or new equity in your range, a turn barrel is "air-on-air"
bluffing: you risk chips without improving fold equity. Checking back:
- Controls the pot size.
- Keeps you on a check-behind line that can sometimes win at showdown.
- Preserves chips for a more credible river bet if the river improves your hand.

---

## Worked Examples

### Example A — DrawComplete Turn: Check
**Hand:** A♦ J♣
**Flop:** 8♥ 7♥ 3♠ (semi-wet — heart flush draw)
**Turn:** K♥ (third heart — **DrawComplete**)
**Pot:** 120 chips. Stack: 300 chips.

**Decision: Check**
The K♥ completes the flush draw. Any caller with 9♥ 5♥, Q♥ T♥, or similar now has a
flush. Villain's check-calling range is now polarised toward made hands. Your AJ has no
pair and no draw — checking is mandatory.

---

### Example B — ScareBroadway Turn: Bet 80%
**Hand:** Q♠ T♦
**Flop:** J♥ 6♣ 4♦ (dry)
**Turn:** Q♦ (ScareBroadway — improves your hand too!)
**Pot:** 100 chips. Stack: 400 chips.

**Decision: Bet 80% pot (80 chips)**
The Q♦ gives you top pair and is a scary Broadway overcard. From the Button your range
includes QQ, QJ, AQ — this card "hit" your story. Villain's J6, 66, 44 (slow plays) are
now behind. A large barrel extracts value from JJ, J6, and forces out 88, 77 type hands.

---

### Example C — Blank Turn, Wet Flop: Bet 50%
**Hand:** A♣ K♠
**Flop:** T♥ 9♥ 5♣ (wet — straight draw + backdoor flush)
**Turn:** 2♦ (Blank)
**Pot:** 140 chips. Stack: 500 chips.

**Decision: Bet 50% pot (70 chips)**
Villain called the flop with draws: J8, Q8, T8, flush draws. A blank 2♦ doesn't complete
anything but doesn't fold those draws either. Charge them for the river card. With one
card to come, flush/straight draws are roughly 20% — they can't call a 50% pot bet
profitably.

---

### Example D — Blank Turn, Dry Flop: Check
**Hand:** K♥ Q♦
**Flop:** K♠ 7♣ 2♦ (dry)
**Turn:** 4♥ (Blank — dry board remains dry)
**Pot:** 100 chips. Stack: 350 chips.

**Decision: Check**
You have top pair top kicker, but the dry flop c-bet already extracted value. The 4♥ is
a complete blank with no draws. Villain's calling range (K9, 77, KT type hands) doesn't
fold to a second barrel here. Check back, protect stack, and bet the river if you can
represent strength.

---

## Balancing Your Barrel Range

Good players don't barrel every hand on good cards or check every hand on bad cards.
A balanced barrel range contains both **value hands** (betting for profit) and **bluffs**
(betting to fold out villain's equity). The ratio varies by board:

- ~2:1 value-to-bluff ratio on most turn barrels.
- Bluffs should have some equity (combo draws, backdoor draws, blockers) — not just
  nothing.

If you barrel with zero equity and zero blockers, you're burning money.

---

## Common Mistakes

1. **Auto-barreling every turn** — many players c-bet the flop and barrel the turn
   regardless of card. This is highly exploitable.
2. **Checking draws-completed turns in panic** — if you have a value hand, the draw
   completing may actually help you too (your flush beats villain's lower flush).
3. **Small barrels on scare cards** — if you're representing a strong range, size up.
   A 33% pot bet on a Broadway scare card is unconvincing.
4. **Ignoring position** — these decisions apply to IP play. OOP turn decisions are
   different and involve donk-bet and check-raise dynamics.

---

## Engine Modelling Notes

- Hero is always IP (BTN or CO) having c-bet the flop.
- Board = 3 flop cards + 1 turn card; `board` field contains all 4 cards.
- Turn type classified by: flush-complete (3 same-suit), straight-complete (4 cards
  spanning ≤ 4 rank units), Broadway (rank ≥ 10), or Blank.
- Answers: Check, Bet ~50% pot, Bet ~80% pot.
- SPR context provided in question text for advanced reasoning.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [2 — Postflop C-bet](02_postflop_continuation_bet.md) | The flop bet that precedes this turn decision |
| [4 — Bluff Spot](04_bluff_spot.md) | River continuation after a turn check-back or missed barrel |
| [7 — Check-Raise Spot](07_check_raise_spot.md) | Villain OOP may check-raise a turn barrel; know both sides |
