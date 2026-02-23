# Topic 16 — Multiway Pot

**Enum variant:** `TrainingTopic::MultiwayPot`
**Scenario ID prefix:** `MW-`
**Street:** Flop (3 board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **multiway pot** is any pot with three or more active players. Multiway pots change
the mathematics of value betting and bluffing fundamentally:

1. **More players = stronger hands needed to bet.** The probability that at least one
   opponent has a strong hand increases with each extra player.
2. **Draws compound.** Each additional player multiplies the draw combinations out
   against you — a flush draw is out there more frequently.
3. **Bluffing becomes much harder.** All opponents must fold simultaneously for a bluff
   to succeed.

---

## Probability of Being Behind

In a heads-up pot, the probability villain has top pair or better on an A-K-x board is
modest. In a multiway pot it rises sharply:

| Players | Probability ≥1 has top pair or better |
|---------|---------------------------------------|
| 2 (HU) | ~30% |
| 3 | ~50% |
| 4 | ~65% |
| 5 | ~75% |

This is why set and two-pair hands **must** bet for protection in multiway pots — there
are too many drawing hands that will outrun you by the river.

---

## Multiway Betting Guidelines

| Hand | Action | Rationale |
|------|--------|-----------|
| Strong (set, two pair, overpair) | Bet large (~67% pot) | Protection critical; multiple draws compound; bet NOW while you are ahead |
| Top pair good kicker | Bet small (~33% pot) | Thin value extraction without overcommitting to a vulnerable hand |
| Middle pair or worse | Check | Too many opponents have better; bluffing is unprofitable; pot control |

---

## Why Bluffing in Multiway Pots Fails

**Bluff success probability = p₁ × p₂ × p₃ × … (all opponents fold)**

If each opponent folds 50% of the time to a bet, in a 3-way pot:
```
Success = 0.50 × 0.50 = 25%
```

A 75% pot bluff requires a 43% fold frequency. With 3 players each folding 50%, you
achieve only 25% — you lose chips on average with every bluff.

The larger the field, the more credibility your bets must have (strong hand) to justify
the risk.

---

## Protection in Multiway Pots

Even with a strong hand, **protecting your equity** is the primary goal of a multiway
flop bet. Example:

- **Hand:** J♠ J♥ (overpair)
- **Board:** 9♣ 8♦ 4♠ (3 opponents)
- Each opponent calling with any two-pair combo, straight draw, or flush draw has
  roughly 35–50% equity against your overpair.
- Checking gives 3 players a free card — catastrophic equity erosion.
- Betting 67% pot charges all draws correctly and denies free equity.

---

## Worked Examples

### Example A — Strong hand: Bet large
**Hand:** 8♣ 8♠ (middle set)
**Board:** Q♦ 8♥ 4♣
**Position:** Cutoff. 3 opponents. Pot: 32 chips. Stack: 100 chips.

**Decision: Bet ~22 chips (~67%)**
You flopped middle set in a multiway pot. There are flush draws, straight draws, and
top pair out against you. Bet 67% to charge every draw. Don't slow-play — giving
free cards in multiway pots with a strong hand is the most costly mistake in poker.

---

### Example B — Top pair: Bet small
**Hand:** A♦ J♠ (top pair top kicker)
**Board:** A♣ 7♠ 3♥
**Position:** Cutoff. 2 opponents. Pot: 20 chips. Stack: 80 chips.

**Decision: Bet ~7 chips (~33%)**
Top pair top kicker is ahead of most hands but vulnerable to sets (A7, A3, 77, 33),
two-pair combos, and some draws. A small bet extracts thin value from weaker aces and
pairs while keeping the pot manageable if one opponent raises.

---

### Example C — Weak hand: Check
**Hand:** K♣ 5♦ (backdoor draw only)
**Board:** Q♠ J♦ 6♥
**Position:** Cutoff. 3 opponents. Pot: 24 chips.

**Decision: Check**
No pair, no draw. In multiway, a bet here needs all 3 opponents to fold — essentially
impossible on a Q-J-6 board with straight draws in play. Check and hope for a free card.

---

## Common Mistakes

1. **Slow-playing strong hands multiway** — checking a set or two pair into 3+ opponents
   is the most expensive error; draws outrun you far too often.
2. **Betting thin value wide** — top pair is often best hands-up but is dominated in
   multiway pots; size down or check depending on texture.
3. **Bluffing into the field** — most multiway pot bluffs fail; you need all players to
   fold simultaneously, which requires unrealistically high individual fold rates.
4. **Ignoring the extra draw risk** — one card that helps a draw in HU doubles the
   effect in a 3-way pot (two players might hold that draw).

---

## Engine Modelling Notes

- Always a flop scenario (3 board cards).
- Hero is always in the Cutoff; opponents fill BTN, BB, SB, and HJ positions.
- Three hand strengths: `Strong`, `TopPair`, `Weak` (uniform distribution).
- Number of opponents: 2 (Beginner), 2–3 (Intermediate), 2–4 (Advanced).
- Correct answers: Strong → Bet large (~67%), TopPair → Bet small (~33%), Weak → Check.
- Three answer options: Check, Bet small (~33%), Bet large (~67%).
- `current_bet = 0` (hero acts first before any opposition).

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [2 — Postflop Continuation Bet](02_postflop_continuation_bet.md) | Heads-up c-bet decisions; compare how sizing changes in multiway context |
| [7 — Check-Raise Spot](07_check_raise_spot.md) | OOP multiway decisions; check-raising in multiway pots requires even stronger hands |
| [8 — Semi-Bluff Decision](08_semi_bluff_decision.md) | Draw equity in multiway pots; draws become harder to play profitably with many opponents |
