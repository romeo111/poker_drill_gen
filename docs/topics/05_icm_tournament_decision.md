# Topic 5 — ICM & Tournament Decision

**Enum variant:** `TrainingTopic::ICMAndTournamentDecision`
**Scenario ID prefix:** `IC-`
**Street:** Preflop (push-or-fold)
**Game type:** Tournament
**Difficulty range:** Beginner → Advanced

---

## Core Principle

In cash games, chips have a fixed dollar value. In tournaments, they do not.

**ICM (Independent Chip Model)** converts chip stacks into prize-money equity. The core
insight is asymmetric: doubling your stack does **not** double your tournament equity
because prize pools are not linear — finishing first pays much more than finishing second,
but you need to survive to collect anything.

This asymmetry creates an **ICM risk premium**: the extra cost of risking your stack that
you must account for when deciding to shove or fold.

---

## Why ICM Changes Push/Fold Decisions

In a cash game with 10 BB you should shove almost any two cards if action folds to you.
In a tournament the calculation changes:

- **If you bust**, you earn nothing from your chip investment.
- **If you double**, you only gain in proportion to the payout structure.
- **Near the bubble**, busting out just before the money is catastrophically expensive.

The result: tournament players should be **tighter than chip-EV alone suggests**,
especially near the bubble or at a final table with large pay jumps.

---

## Tournament Stages and ICM Pressure

| Stage | ICM pressure | Push threshold (simplified) | Rationale |
|-------|-----------|-----------------------------|-----------|
| Early Levels | Very low (~3%) | ≤ 20 BB | Deep stacks, many players; risk is cheap |
| Middle Stages | Moderate (~8%) | ≤ 15 BB | Bubble approaching; some ICM awareness |
| Bubble | High (~20%) | ≤ 10 BB | Busting = no money; max fold equity value |
| Final Table | High (~15%) | ≤ 12 BB | Pay jumps; each bust worth more |

The engine uses simplified stack thresholds **modified by hand strength** rather than
full Malmuth-Harville ICM calculations. A `PushTier` system (Premium / Strong / Playable / Weak)
adjusts the base threshold: premium hands push at deeper stacks (+8 BB), while weak hands
require more desperation (−4 BB from base).

---

## The Push/Fold Framework

When stack depth drops below ~15–20 BB in a tournament, the decision simplifies to
**push or fold**:

- **Raising small** (to 2–3 BB) is no longer viable because it commits too many chips
  relative to the stack without getting the money in.
- **Limping** is almost always wrong — it invites raises that put you in a worse spot.
- The only question is: does your hand have enough equity to justify shoving?

### Push/Fold Heuristics

At 10 BB from late position, any hand with equity ≥ 30% vs villain's calling range
is typically a shove. At 20 BB the bar is higher because the all-in commitment is
proportionally larger.

**Approximate push ranges from BTN (heads up to BB):**

| Stack depth | Push range |
|-------------|-----------|
| ≤ 5 BB | Any two cards |
| 6–8 BB | Any two cards (wide), including 72o |
| 9–12 BB | Any pair, any ace, K+x, Q+T+, suited connectors |
| 13–17 BB | Pairs 66+, A9o+, KJo+, suited Ax, KQs, broadways |
| 18–22 BB | Pairs 77+, A10o+, KQo+, strong broadways |

---

## Worked Examples

### Example A — Bubble Shove
**Stage:** Bubble — 11 players remain, top 10 paid
**Hand:** A♠ 7♦ (ATo)
**Position:** Button
**Stack:** 8 BB. Villain (BB): 40 BB.

**Decision: Shove**
At 8 BB you are losing ~12% of your stack per orbit from blinds alone. ICM pressure is
high (≈20%) but you cannot fold your way to the money from 8 BB — you'll be in the
blinds within 2 orbits with less fold equity. A7o has strong equity vs villain's calling
range. Push.

---

### Example B — Bubble Fold
**Stage:** Bubble — 11 players remain, top 10 paid
**Hand:** K♣ 4♦ (K4o)
**Position:** Button
**Stack:** 18 BB. Villain (BB): 40 BB.

**Decision: Fold**
At 18 BB with ICM pressure at ≈20%, K4o does not have sufficient equity against a
reasonable BB calling range (roughly TT+, AJo+, KQo, Axs). You have enough chips to
survive multiple orbits and find a better spot. The risk-premium makes this a fold.

---

### Example C — Final Table Shove
**Stage:** Final Table — 5 players remain (top 5 paid with escalating jumps)
**Hand:** 9♠ 9♦
**Position:** Button
**Stack:** 11 BB. Villain (BB): 25 BB.

**Decision: Shove**
99 is a clear shove at 11 BB. Even with ICM pressure (~15% risk premium), 99 has 70%+
equity vs a typical calling range. The blind pressure means waiting is costly. Any hand
this strong at 11 BB is an immediate shove.

---

### Example D — Early Levels, Deep Stack
**Stage:** Early Levels — 95 players remain
**Hand:** 7♣ 3♠
**Position:** Button
**Stack:** 85 BB. Villain (BB): 90 BB.

**Decision: Fold**
At 85 BB deep with minimal ICM pressure, limping or shoving with 73o from the BTN would
be standard poker. However, shoving 85 BB with 73o to try to steal 1.5 BB (the blinds)
is terrible risk/reward — you risk 85 BB to win 1.5 BB. Fold or play it normally as a
raise, but the engine correctly flags this as a fold.

---

## Blind Pressure: The Clock Is Ticking

Each orbit costs you 1.5 BB (SB + BB, ignoring antes). Your stack in orbits remaining:

```
Orbits remaining = Stack BB / 1.5
```

| Stack BB | Orbits remaining | Urgency |
|----------|-----------------|---------|
| 20 BB | 13 orbits | Plenty of time |
| 10 BB | 6–7 orbits | Starting to feel pressure |
| 6 BB | 4 orbits | Must shove next good spot |
| 3 BB | 2 orbits | Shove any two cards |

---

## Pay Jump Awareness

At a final table, each elimination moves all remaining players one pay step higher. The
value of **surviving one more bust** is concrete and calculable.

**Rule:** Be more conservative when the gap between current pay and next pay is large.
Be more aggressive when pay jumps are small and even distribution makes chip accumulation
worthwhile.

---

## Common Mistakes

1. **Folding into the blinds** — inexperienced players fold too often near the bubble,
   letting their stack erode to a point where they have no fold equity.
2. **Ignoring stack depth** — a 20 BB push/fold shove with A2o from the CO is usually
   correct; the same shove at 60 BB is terrible.
3. **Playing cash-game poker in tournaments** — value-betting and calling ranges from
   cash games do not transfer to short-stacked tournament spots.
4. **Over-tightening at the final table** — with antes, the pot is often large enough
   that stealing with any two cards is correct from late position.

---

## Engine Modelling Notes

- Always a tournament game type (`GameType::Tournament`).
- Four stages: `EarlyLevels`, `MiddleStages`, `Bubble`, `FinalTable`.
- Hero stack is sampled per difficulty (Beginner: 6–18 BB; Advanced: 3–30 BB).
- Hero's hand is classified into a `PushTier` (Premium/Strong/Playable/Weak) which adjusts the push threshold.
- `should_push = hero_stack_bb <= push_threshold_bb(stage, push_tier)` determines the correct answer.
- Only two answers: `Shove all-in` or `Fold` — no limping or small-raise options.
- Risk premium percentage is displayed in explanations to reinforce ICM awareness.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [1 — Preflop Decision](01_preflop_decision.md) | The cash-game equivalent; same hand categories, different stack/EV model |
