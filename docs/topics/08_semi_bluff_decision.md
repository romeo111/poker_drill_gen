# Topic 8 — Semi-Bluff Decision

**Enum variant:** `TrainingTopic::SemiBluffDecision`
**Scenario ID prefix:** `SB-`
**Street:** Flop or Turn
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **semi-bluff** is a raise or bet made with a hand that is currently losing but has
significant equity to improve. Unlike a pure bluff (no outs), a semi-bluff can win in
two ways:

1. **Villain folds** — you win the pot immediately (fold equity).
2. **You hit your draw** — you win at showdown even when called.

This dual path to profit makes semi-bluffs among the highest-EV plays in poker,
especially with strong draws on early streets.

The key distinction from **Topic 3 (Pot Odds & Equity)** is that here you have the
option to **raise** — applying active pressure — rather than just passively calling.

---

## Draw Types and Equity

| Draw type | Outs | Equity (flop) | Equity (turn) | Raise? |
|-----------|------|---------------|---------------|--------|
| Combo draw (flush + OESD) | ~15 | ~54% | ~30% | Yes — near-favourite |
| Open-ended straight draw (OESD) | 8 | ~32% | ~17% | Yes — with stack depth |
| Flush draw | 9 | ~35% | ~20% | Depends on position |
| Gutshot | 4 | ~17% | ~9% | No — insufficient equity |

---

## The Four Decision Outcomes

The engine presents three options: Fold, Call, or Raise (semi-bluff).

| Draw type | Position | Stack | Correct action | Reason |
|-----------|----------|-------|----------------|--------|
| ComboDraw | Any | Any | **Raise** | ~54% equity = favourite; maximise pressure |
| FlushDraw | IP | Any | **Call** | Good equity, realise it in position |
| OESD | Any | ≥ 40 BB | **Raise** | Fold equity + 32% equity = strong semi-bluff |
| GutShot | Any | Any | **Fold** | ~17% equity rarely justifies call or raise |
| FlushDraw | OOP | Any | **Call** | Can't raise without positional advantage |

---

## Why Position Changes the Decision

**In position (IP):** You act last on every street. You can:
- Call draws cheaply and see the turn with information.
- Fire on the river when you hit or when villain shows weakness.
- Control pot size and decide whether to semi-bluff the turn.

**Out of position (OOP):** You act first. Calling OOP has disadvantages:
- Villain can bet the turn when you check, putting you in repeated difficult spots.
- Your hand is harder to defend on bad runouts.
- Semi-bluff raising OOP requires a stronger draw (combo draw level) to justify the
  risk of getting re-raised into a large pot without a made hand.

---

## Fold Equity — The Extra Dimension

Fold equity is what makes raising better than calling with a draw:

```
Raise EV = (P(villain folds) × Pot) + (P(villain calls) × Equity × Future pot)
Call EV = Equity × Future pot - Call amount
```

With a combo draw (~54% equity), your raise EV is:
- If villain folds 40% of the time to a 2.5× raise: you win the pot 40% outright.
- If villain calls 60% of the time: you're a slight favourite (54% > 46%).

This means raising with a combo draw is correct even if villain never folds — the
raw equity justifies it. The fold equity is pure bonus.

---

## Semi-Bluff Raise Sizing

The engine uses 2.5× villain's bet as the semi-bluff raise size.

**Why 2.5×?**
- Small enough to not over-commit when villain 3-bets.
- Large enough to make villain's equity-holding calls expensive.
- Consistent with check-raise sizing (Topic 7) for range balance.

On the flop with a combo draw, you ideally build a pot where you can stack off — you
want a large pot when you're a favourite.

---

## Worked Examples

### Example A — Combo Draw: Always Raise
**Hand:** 8♦ 7♦
**Board:** 9♦ 6♦ 5♣ (flush draw + OESD — combo draw)
**Villain bets:** 80 chips into 120-chip pot.
**Stack:** 500 chips (42 BB)

**Calculation:**
```
Combo draw equity (flop) = ~54%
Villain's equity = ~46%
You are a FAVOURITE!
```
**Decision: Raise to 200 chips (2.5 × 80)**
You're ahead in equity and have fold equity on top. This is the clearest semi-bluff in
poker. Raise to build the pot, apply pressure, and set up a turn shove or call.

---

### Example B — OESD with Stack Depth: Raise
**Hand:** J♣ T♦
**Board:** Q♠ 9♥ 3♣ (open-ended straight draw: K or 8 completes)
**Villain bets:** 50 chips into 100-chip pot.
**Stack:** 500 chips (50 BB — deep)

**Decision: Raise to 125 chips (2.5 × 50)**
OESD has ~32% equity (8 outs × 4 = 32%). At 50 BB deep, fold equity is substantial
and implied odds make continuing very profitable. Semi-bluffing here denies villain's
equity from pairs, allows you to win outright when villain folds, and sets up a large
pot when you make the straight.

At shallow stacks (< 25 BB), the risk of raising into a shove diminishes the play's
value — calling would be better.

---

### Example C — Flush Draw IP: Call
**Hand:** A♥ 4♥
**Board:** K♥ 8♥ 3♦ (nut flush draw)
**Position:** Button (IP)
**Villain bets:** 60 chips into 100-chip pot.
**Stack:** 400 chips

**Decision: Call**
Nut flush draw from the Button. You have position to realise your ~35% equity on all
remaining streets. Raising here:
- May get 3-bet by strong hands (KK, 88, K8) that have you crushed.
- Bloats the pot when you'd rather see the turn and river cheaply.
- Calling lets you set the pace — bet the turn when you hit, check-back when you miss.

---

### Example D — Flush Draw OOP: Call (not raise)
**Hand:** A♣ 6♣
**Board:** K♣ T♣ 2♠ (nut flush draw)
**Position:** Big Blind (OOP)
**Villain bets:** 60 chips into 100-chip pot.

**Decision: Call**
Same hand, but now OOP. Raising here puts you in a bloated pot OOP:
- If re-raised, you face a huge call from out of position with only a draw.
- Every card except a club is awkward — you'll check and face another bet.
- Calling allows you to check-raise the turn on club runouts (stronger line).

---

### Example E — Gutshot: Fold
**Hand:** K♣ J♦
**Board:** A♠ T♥ 5♦ (gutshot straight draw to the Q)
**Villain bets:** 70 chips into 100-chip pot.

**Decision: Fold**
```
Gutshot equity (flop) = ~17%
Required equity to call = 70 / (100 + 70) = 41.2%
17% << 41.2% — mathematically, must fold
```
Even raising as a semi-bluff is incorrect: you have almost no raw equity and insufficient
fold equity to justify the investment. Fold.

---

## Balancing Semi-Bluffs and Value Raises

For your raising range to be balanced (unexploitable):
- ~60–70% value hands when raising.
- ~30–40% semi-bluffs when raising.

This means you should not raise *every* draw you have. Choose the strongest draws
(combo draws, nut flush draws with position) and fold the weakest (gutshots, non-nut
draws OOP).

---

## Common Mistakes

1. **Calling every draw passively** — the worst players call with 35% equity when raising
   with 35% equity + fold equity is higher EV.
2. **Raising gutshots** — 4 outs is not enough equity to build a large pot against a
   calling range.
3. **Raising OOP without a plan** — semi-bluff raises OOP create huge pots that are
   difficult to navigate on bad turns.
4. **Never semi-bluffing** — players who only bet with made hands are extremely easy
   to read and fold against.

---

## Engine Modelling Notes

- Draw type determined from board: flush draw = 2+ same-suit board cards; straight draw
  = 2 consecutive/near-consecutive rank board cards; combo = both.
- Villain bet is randomly sized; hero position (IP/OOP) and stack depth vary by difficulty.
- Stack depth threshold for OESD raise: 40 BB.
- Answers: Fold, Call (passive), Raise (semi-bluff, 2.5× bet).
- Correct answer derived from a decision table keyed on (draw type, position, stack).
