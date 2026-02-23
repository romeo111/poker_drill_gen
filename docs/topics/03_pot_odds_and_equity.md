# Topic 3 — Pot Odds & Equity

**Enum variant:** `TrainingTopic::PotOddsAndEquity`
**Scenario ID prefix:** `PO-`
**Street:** Flop (two cards to come)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

**Pot odds** are the ratio of how much you must call relative to the total pot after
calling. **Equity** is your probability of winning the pot at showdown. The fundamental
rule is simple:

> **If your equity exceeds the pot odds required to call, calling is profitable.**

This is pure mathematics — a decision you can calculate at the table — and it forms the
bedrock of all postflop decision-making.

---

## The Formula

```
Required equity to call = Call amount / (Pot after calling)
                        = Call / (Current pot + Call)
```

**Example:** Pot is 100 chips. Villain bets 50. You must call 50.

```
Required equity = 50 / (100 + 50) = 50 / 150 = 33.3%
```

If your hand has **more than 33.3%** equity, calling is +EV. Otherwise, fold.

---

## Draw Equities

The engine assigns draws based on board texture and uses the following approximate
equities on the flop (two streets remaining):

| Draw type | Outs | Equity (2 streets) | Equity (1 street) |
|-----------|------|--------------------|--------------------|
| Combo draw (flush + OESD) | ~15 | ~54% | ~30% |
| Flush draw | ~9 | ~35% | ~20% |
| Open-ended straight draw (OESD) | ~8 | ~32% | ~17% |
| Gutshot straight draw | ~4 | ~17% | ~9% |

These come from the standard "Rule of 4 and 2":
- Multiply your outs by **4** on the flop (two cards to come).
- Multiply your outs by **2** on the turn (one card to come).

---

## Worked Examples

### Example A — Flush Draw vs Large Bet
**Hand:** A♥ 5♥
**Board:** K♥ 9♥ 3♣ (two hearts on board = flush draw)
**Pot:** 100 chips
**Villain bets:** 80 chips (80% pot)

**Calculation:**
```
Required equity = 80 / (100 + 80) = 80 / 180 = 44.4%
Flush draw equity (2 streets) = ~35%
```
**Decision: Fold**
35% < 44.4% — calling is -EV. The bet is too large relative to your draw equity.

---

### Example B — Open-Ended Straight Draw vs Half-Pot Bet
**Hand:** J♠ T♦
**Board:** Q♣ 9♥ 2♦ (J+T+Q+9 = OESD: 8s or Ks complete a straight)
**Pot:** 120 chips
**Villain bets:** 60 chips (50% pot)

**Calculation:**
```
Required equity = 60 / (120 + 60) = 60 / 180 = 33.3%
OESD equity (2 streets) = ~32%
```
**Decision: Fold (marginal)**
32% is very close to 33.3%, but slightly below the break-even threshold. This is a thin
spot; on the turn with 17% equity vs a small bet it would be clearer.

---

### Example C — Combo Draw vs Small Bet
**Hand:** 8♥ 7♥
**Board:** 6♥ 5♣ 9♥ (flush draw + OESD = combo draw)
**Pot:** 200 chips
**Villain bets:** 60 chips (30% pot)

**Calculation:**
```
Required equity = 60 / (200 + 60) = 60 / 260 = 23.1%
Combo draw equity (2 streets) = ~54%
```
**Decision: Call (strongly)**
54% >> 23.1%. You are a favourite! Not only should you call, but raising for value and
fold equity is worth considering (see Topic 8 — Semi-Bluff Decision).

---

### Example D — Gutshot vs Any Bet
**Hand:** K♣ J♦
**Board:** T♠ 8♥ 3♣ (only a gutshot to the straight via Q)
**Pot:** 100 chips
**Villain bets:** 50 chips (50% pot)

**Calculation:**
```
Required equity = 50 / (100 + 50) = 33.3%
Gutshot equity (2 streets) = ~17%
```
**Decision: Fold**
17% << 33.3%. A gutshot alone rarely justifies calling. Even against a small 25% pot bet
(required equity 20%), you are still marginally below the break-even point.

---

## Implied Odds

**Pot odds alone aren't the full picture.** When you complete your draw, you often win
more chips from villain — this is called **implied odds**.

Implied odds are relevant when:
- Your draw is not obvious to villain (e.g., a backdoor flush draw vs an obvious OESD on
  a connected board).
- You are deep-stacked (SPR ≥ 4) and villain has chips to lose.
- Villain is likely to pay off a completed draw.

A rough rule: if implied odds would roughly double your pot-odds calculation, a marginal
call can become correct. However, implied odds cannot save a gutshot vs a large bet at
shallow depths.

---

## Reverse Implied Odds

**Reverse implied odds** are the chips you lose when you hit your draw but still lose.

Example: You hold A♣ 2♣ and flop a flush draw on K♣ Q♣ 3♠. If the flush completes but
villain has K♣ J♣ (a better flush draw or already-made flush), hitting your hand costs
you a big pot.

Watch out for:
- **Non-nut flush draws** — someone could have a higher flush draw.
- **Low straights** on double-paired boards — your straight might be beaten by a full
  house.

---

## Common Mistakes

1. **Calling any draw without calculating** — "I have a flush draw" is not a reason to
   call. You must compare equity to pot odds.
2. **Ignoring bet size** — a 33% pot bet is often correct to call with a flush draw; a
   100% pot bet usually is not.
3. **Forgetting about outs** — on paired boards, some of your outs may give villain a
   full house. Count **clean outs** only.
4. **Calling with gutshots** — gutshots rarely have sufficient pot odds. Fold unless
   you have overcards or other equity.

---

## Engine Modelling Notes

- The scenario always places hero on the flop (two streets remaining).
- Draw type is determined from the actual board: flush draw detected by 2+ same-suit cards,
  straight draw by 2 consecutive/near-consecutive ranks.
- Bet size is randomly sampled per difficulty (33–150% pot at Advanced).
- Correct answer (`Call` or `Fold`) is determined by `actual_equity >= required_equity`.
- The explanation shows the exact breakeven math so players learn to perform the
  calculation themselves.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [2 — Postflop C-bet](02_postflop_continuation_bet.md) | The aggressor's perspective: bet sizes that give villain wrong pot odds |
| [8 — Semi-Bluff Decision](08_semi_bluff_decision.md) | When a draw is strong enough to raise instead of just call |
