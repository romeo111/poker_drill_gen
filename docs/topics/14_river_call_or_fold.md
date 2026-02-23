# Topic 14 — River Call or Fold

**Enum variant:** `TrainingTopic::RiverCallOrFold`
**Scenario ID prefix:** `RF-`
**Street:** River (5 board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

When facing a river bet, you have no more cards coming. The decision is purely:
**does your hand have enough equity to justify a call at this price?**

The three decisions are:
1. **Fold** — your hand is too weak for the price being charged.
2. **Call** — pot odds justify calling; you are ahead of enough of villain's range.
3. **Raise for value** — villain bet small with a thin range you beat; extract more.

---

## Required Equity Formula

The minimum equity to break even on a call:

```
Required equity = Call size / (Current pot + Call size)
```

Where "current pot" is the pot *after* villain's bet (your call is added to the whole pot).

More precisely:
```
Required equity = Call / (Pre-bet pot + Villain bet + Call)
                = Call / (Pre-bet pot + 2 × Villain bet)
```

| Villain bet (% pre-bet pot) | Required equity |
|----------------------------|-----------------|
| 33% | ~20% |
| 50% | ~25% |
| 67% | ~29% |
| 100% | ~33% |
| 125% | ~36% |

---

## Thinking in Ranges, Not Hands

On the river, villain has a **polarised betting range** — strong hands that want to
extract value, plus bluffs. The question is: what fraction of villain's betting range
do you beat?

- **Against a small bet (~33%):** Villain's range is wide (includes many semi-bluffs
  and thin value bets). You can call profitably with a wider range.
- **Against a large bet (~pot):** Villain is claiming a polar range (strong value or
  total bluff). You need strong hands or reliable bluff-catchers.
- **Against an overbet:** Villain is representing the absolute nuts. Only call with
  very strong hands that beat most of villain's value range.

---

## When to Raise for Value

A river raise is correct when:
1. Your hand beats the majority of villain's betting range.
2. Villain bet small, leaving room to grow the pot.
3. Villain will call your raise with worse hands (top pair, second pair).

**Do not raise as a bluff on the river** unless you have a clear plan — bluff-raising
the river commits a large portion of the stack with limited fold equity once called.

---

## Worked Examples

### Example A — Fold: Villain large bet, weak hand
**Hand:** 6♣ 5♣ (missed straight draw)
**Board:** K♠ Q♦ 7♥ 8♣ A♠
**Position:** Button. Pot: 200 chips. Villain bets 200 chips (pot).
**Required equity:** 200 / (200 + 200 + 200) = 33.3%

**Decision: Fold**
Your hand has essentially 0% showdown equity. You need 33% to call — you have 0%.
Fold immediately.

---

### Example B — Call: Standard bet, marginal hand
**Hand:** K♦ 9♣ (top pair medium kicker)
**Board:** K♠ 7♣ 3♥ 2♦ J♠
**Position:** Button. Pot: 160 chips. Villain bets 107 chips (~67%).
**Required equity:** 107 / (160 + 107 + 107) = 28.6%

**Decision: Call**
You beat all of villain's bluffs (missed clubs, missed straight draws) and a portion of
their value range (K-worse kicker, 77, 33, 22 are all possible in a calling range but
you beat J-x and missed draws). Against a polarised 67% bet you need only 28.6% — your
top pair satisfies this.

---

### Example C — Raise: Small bet, strong hand
**Hand:** K♠ K♥ (top set)
**Board:** K♣ 7♦ 4♠ 2♣ 9♥
**Position:** Button. Pot: 200 chips. Villain bets 66 chips (~33%).
**Call price:** 66 chips. Your equity: ~85%.

**Decision: Raise to ~165 chips (2.5×)**
Villain's small bet often represents a thin value range (top pair, middle pair). You
have the nuts. A raise to 165 chips is credible and extracts 2.5× more than a call.
Villain will call with Kx, 77, 44. Their 9x may also call, thinking their one pair is
good here.

---

## Common Mistakes

1. **Hero-calling with insufficient equity** — top pair is not always worth calling a
   pot-sized bet; calculate the required equity threshold.
2. **Folding too cheaply against small bets** — villain's small river bet has high
   bluffing frequency; a 33% bet needs you to fold only 25% for it to profit, meaning
   you should call wide.
3. **Missing value raise opportunities** — when you have a very strong hand and villain
   bet small, raising is significantly higher EV than calling.
4. **Over-folding to overbets** — overbets do not always mean the nuts; balanced players
   overbet with bluffs too (especially hands that block your calling range).

---

## Engine Modelling Notes

- Always a river scenario (5 board cards).
- Hero is always on the Button; villain bets into hero.
- Three paired (hand strength, bet size) scenarios with one correct answer each:
  - `Strong + Small bet` → Raise for value
  - `Marginal + Standard bet` → Call
  - `Weak + Large bet` → Fold
- Three answer options: Fold, Call, Raise.
- Required equity displayed in the question.
- `current_bet` = villain's bet amount.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [3 — Pot Odds & Equity](03_pot_odds_and_equity.md) | Same pot-odds math but on the flop with a draw; the river version has no equity run-out |
| [10 — River Value Bet](10_river_value_bet.md) | The inverse: here hero faces a bet; in T10 hero makes the bet |
| [4 — Bluff Spot](04_bluff_spot.md) | Villain may be bluffing — understanding bluff frequencies informs call decisions |
