# Topic 2 — Postflop Continuation Bet

**Enum variant:** `TrainingTopic::PostflopContinuationBet`
**Scenario ID prefix:** `CB-`
**Street:** Flop
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **continuation bet (c-bet)** is a bet made on the flop by the player who was the
aggressor preflop (i.e., the one who raised or 3-bet). Because the raiser's range
contains all premium hands, they have a natural "story" to tell — they raised preflop,
so why wouldn't they bet the flop?

The key question is not *whether* to c-bet, but **how big**, or whether to skip the c-bet
entirely on boards that don't favour the aggressor's range.

---

## Board Texture — The Foundation of Sizing

The engine classifies every flop into three textures:

| Texture | Description | Example boards |
|---------|-------------|---------------|
| **Dry** | No flush draws, no connected cards | K♠ 7♦ 2♣ |
| **Semi-Wet** | One draw type present (flush OR straight) | J♥ 8♦ 3♥ |
| **Wet** | Both flush and straight draws possible | 9♠ 8♥ 7♦ |

### Why texture drives sizing

On a **dry** board, draws are rare. Villain can't improve dramatically between streets.
A small c-bet (25–33% pot) achieves the same fold equity as a large one — you're making
villain fold their share of weak hands at minimal cost.

On a **wet** board, villain likely has many draw combinations (flush draws, straight draws,
combo draws). A small bet gives them the correct pot odds to continue. You need a larger
sizing (67–75% pot) to charge draws correctly.

---

## Range Advantage

**Range advantage** means your overall preflop range (all hands you'd play the same way)
hits this board harder than villain's range.

A late-position (CO, BTN) raiser has a range full of Broadway cards (A, K, Q, J, T) and
connected suited hands. On a **low** board like 7♦ 4♣ 2♥, the raiser's range actually
has *less* of an advantage because villain (BB caller) has a lot of 7x, 44, 22 type
hands.

On a **high** board like K♠ Q♦ 5♣, the CO/BTN range is heavily favoured — lots of KQ,
KJ, QJ, AK, sets. A small bet here is optimal.

---

## Correct Sizing by Scenario

| Board type | Hero has range advantage? | Correct c-bet |
|-----------|--------------------------|---------------|
| Dry | Yes (late position) | 33% pot (small) |
| Dry | No | Check |
| Semi-Wet / Wet | Either | 75% pot (large) |

---

## Worked Examples

### Example A — Dry Board, Range Advantage
**Hand:** A♠ K♣ (AKo)
**Position:** Button (hero raised preflop)
**Flop:** K♦ 7♣ 2♥ (dry)
**Villain:** BB checked

**Decision:** Bet 33% pot
**Why:** You have top pair top kicker, and the Board is dry (no draws to charge). From the
Button your range is heavily skewed toward Broadway cards, giving you a clear range
advantage. A small bet extracts value from villain's Kx and middle pairs without
over-committing. Betting large here folds out hands you beat and calls by hands that have
legitimate equity.

---

### Example B — Dry Board, No Range Advantage
**Hand:** 9♠ 9♦
**Position:** CO (hero raised preflop)
**Flop:** 8♣ 4♦ 2♠ (dry)
**Villain:** BB checked

**Decision:** Check
**Why:** The BB's calling range includes a lot of 8x, 44, 22 hands that hit this board
hard. Your range advantage on this low, dry board is not clear. Betting folds hands you
crush and keeps hands with equity. Checking back controls the pot, lets you see the turn,
and avoids building a large pot without a strong hand.

---

### Example C — Wet Board
**Hand:** Q♥ J♦
**Position:** Button (hero raised preflop)
**Flop:** T♦ 9♦ 6♣ (wet — flush draw + straight draw)
**Villain:** BB checked

**Decision:** Bet 75% pot
**Why:** This is an extremely wet board. Villain could have 87, 78, J8, flush draws, combo
draws. A small bet gives them the correct odds to draw. A 75% bet makes it mathematically
incorrect for villain to call with a bare flush draw (~20% equity on the flop with one
street to come). Required equity to call a 75% pot bet: 75 / (100 + 75) = 43%, but a
flush draw is only worth ~35%.

---

## The Math: Pot Odds to Call a C-bet

For any bet size as a percentage of the pot, the required equity to call is:

```
Required equity = Bet / (Pot + Bet)
```

| C-bet size | Required equity |
|-----------|----------------|
| 33% pot   | 24.8%          |
| 50% pot   | 33.3%          |
| 75% pot   | 42.9%          |
| 100% pot  | 50.0%          |

A **flush draw** has ~35% equity on the flop (two streets). A 75% pot bet forces the
wrong call; a 33% pot bet gives them the right price.

---

## When NOT to C-bet

1. **Multiway pots** — the more callers, the harder it is to get all to fold.
2. **Low, connected boards vs calling ranges** — BB defends have many low-card
   combinations.
3. **When you have showdown value** — sometimes checking back with a medium pair
   is better than turning it into a bluff.
4. **Very deep stacks** — without a strong hand, building a large pot at 200+ BB depth
   risks playing for stacks later.

---

## Common Mistakes

1. **Auto c-betting every flop** — c-betting 100% is exploitable; villains can
   check-raise wide and deny equity.
2. **Using one size for all boards** — a 33% bet on a wet board is too small; a 75%
   bet on a dry board is too large.
3. **C-betting without a plan for later streets** — if you bet the flop, have a turn
   and river strategy in mind.

---

## Engine Modelling Notes

- The engine generates a 3-card flop and classifies texture via `board_texture()` in
  `evaluator.rs`.
- `hero_has_range_adv` is `true` when hero is CO/BTN **and** the lowest board rank ≤ 8
  (low boards reduce late-position range advantage).
- Answers: Check, Bet 33% pot, Bet 75% pot, Overbet 125% pot.
- Correct answer is determined by the `(texture, range_advantage)` matrix above.
