# Topic 10 — River Value Bet

**Enum variant:** `TrainingTopic::RiverValueBet`
**Scenario ID prefix:** `RV-`
**Street:** River (5 board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **value bet** is a bet made with the intention of being called by a worse hand. On the
river your hand is fixed — there are no more draws to improve or fear. The only question
is: how much can you extract from villain's calling range?

The three decisions are:
1. **Check** — no value in betting; you control the pot and take a free showdown.
2. **Bet standard** — extract value from the hands that call; a credible sizing.
3. **Overbet** — squeeze maximum EV from a polar top-of-range hand.

---

## Value Bet Sizing

The optimal size depends on how strong your hand is relative to villain's calling range:

| Hand strength | Sizing | Rationale |
|---------------|--------|-----------|
| Nuts (top set, flush, straight) | 125%+ pot (overbet) | Polarised range — bluffs justify large size; villain cannot fold strong one-pair hands |
| Strong (top two pair, second set) | 75% pot | Extracts value from top pair and weaker two pair; remains credible without over-pricing |
| Medium (one pair, weak two pair) | Check | Thin value bets often called by better hands; check-raise risk; take the free showdown |

---

## The Thin Value Trap

Betting a medium hand for value is one of the most common mistakes. Consider:

- You hold top pair medium kicker.
- If you bet, villain calls with: top pair better kicker (you lose), sets (you lose),
  two pair (you lose), and occasionally worse one pair (you win).
- If you check, villain bets with hands that beat you — but you only call them off
  rather than inflating the pot.

In most river spots, the expected value of checking a one-pair hand exceeds the EV of
a thin value bet once you account for the check-raise threat.

---

## Polarised Overbets

An overbet makes strategic sense when your range includes both **the nuts** and
**bluffs** — a polarised range. If villain cannot differentiate value from bluff,
they must call off a large bet or fold, giving you maximum EV either way.

Conditions for a profitable overbet:
1. Your hand is at the top of your range (fully captured the board).
2. Villain's range is capped (they cannot have the nuts, e.g., they called preflop OOP).
3. Your bluffing frequency is balanced (roughly 1 bluff per 2.5 value overbets at 125% pot).

---

## Worked Examples

### Example A — Nuts: Overbet
**Hand:** A♣ K♣ (nut flush)
**Board:** Q♣ 8♣ 4♠ J♦ 2♣
**Position:** Button. Pot: 200 chips. Stack: 600 chips.

**Decision: Overbet (125% pot, 250 chips)**
Villain called your flop and turn bets — they have a pair, two pair, or a smaller flush.
They cannot fold a strong hand here. Maximise the pot with a bet they cannot ignore.

---

### Example B — Strong: Large bet
**Hand:** Q♠ J♠ (top two pair)
**Board:** Q♦ J♣ 7♥ 3♠ 2♦
**Position:** Button. Pot: 160 chips. Stack: 500 chips.

**Decision: Bet 75% pot (120 chips)**
Villain likely has a Q or a J. A 75% bet extracts value from their one-pair hands while
remaining believable. Overbetting risks pricing out all but their strongest holdings.

---

### Example C — Medium: Check
**Hand:** A♦ 8♥ (top pair weak kicker)
**Board:** A♣ K♠ 9♥ 5♦ 2♣
**Position:** Button. Pot: 140 chips.

**Decision: Check**
Villain's calling range includes AK, A9, A5, K9, K5 — all of which beat you. A bet
of any size is called by hands that dominate you and folded by hands you also beat.
Check and accept the free showdown.

---

## Common Mistakes

1. **Thin value betting medium hands** — the classic trap; you bet, get called by better,
   and wonder why you lost.
2. **Not value betting strong hands** — checking your best river hands leaves significant
   EV on the table. Villain cannot know you're strong.
3. **Wrong sizing** — a 33% pot bet with the nuts is far too small; a 125% pot bet with
   top pair is far too risky.

---

## Engine Modelling Notes

- Always a river scenario (5 board cards).
- Hero is always on the Button, villain in the Big Blind (checked to hero).
- Three hand strengths: `Nuts`, `Strong`, `Medium` (uniform distribution).
- Correct answers: Nuts → Overbet (~125%), Strong → Large bet (~75%), Medium → Check.
- Four answer options: Check, Small (~33%), Large (~75%), Overbet (~125%).
- `current_bet = 0` (villain checked to hero).

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [4 — Bluff Spot](04_bluff_spot.md) | The mirror: betting river without a strong hand; both share sizing principles |
| [14 — River Call or Fold](14_river_call_or_fold.md) | When villain bets the river instead, hero must evaluate calling equity |
