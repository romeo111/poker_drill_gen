# Topic 13 — 3-Bet Pot C-Bet

**Enum variant:** `TrainingTopic::ThreeBetPotCbet`
**Scenario ID prefix:** `3B-`
**Street:** Flop (3 board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

When you 3-bet preflop and get called, the flop arrives in a **larger-than-normal pot**
with a **lower stack-to-pot ratio (SPR)**. This changes c-betting strategy fundamentally
compared to single-raised pots.

Key properties of a 3-bet pot flop:
1. **Low SPR** — stacks commit faster; bets are more decisive.
2. **Range advantage** — you typically have a stronger range than a BB caller.
3. **Different sizing** — because SPR is lower, absolute bet sizes are a larger fraction
   of the remaining stack.

---

## Stack-to-Pot Ratio in 3-Bet Pots

**SPR = Remaining stack ÷ Pot size on the flop**

In a typical single-raised pot: SPR ≈ 10–15.
In a 3-bet pot: SPR ≈ 2–6 (stacks are much closer to being committed).

| SPR | Interpretation |
|-----|----------------|
| < 1.5 | Both players essentially committed; check/call anything |
| 1.5–3 | One street of betting before all-in; bet or check with care |
| 3–6 | 3-bet pot typical; bets start the commitment process |
| > 6 | Unusual for 3-bet pot; treat more like a deep-stack decision |

---

## C-Bet Decision Matrix

| Board texture | Hand strength | Correct action | Rationale |
|---------------|---------------|----------------|-----------|
| Dry (rainbow, uncoordinated) | Strong (top pair+, overpair) | Small c-bet (~33%) | Dry boards miss villain's range; small probe extracts value and starts stack commitment |
| Wet (two-tone, connected) | Strong (top pair+, overpair) | Large c-bet (~67%) | Charge draws; deny equity; commit stack on this street while ahead |
| Any | Weak (missed, underpair) | Check | Low SPR means any bet is hard to fold to a raise; preserve options |

---

## Why Small Bets Work on Dry Boards

On a dry, rainbow, uncoordinated board:
- Villain's calling range (mostly suited connectors, mid pairs) missed.
- There are few draws to charge.
- A 33% pot bet achieves max fold equity at minimum cost.
- With low SPR, this probe starts the natural stack commitment without over-risking.

---

## Why Large Bets Work on Wet Boards

On a wet, connected, or two-tone board:
- Villain frequently has flush draws, open-ended straight draws, or gutshots.
- A small bet gives them excellent pot odds to call and realise their draw equity.
- A 67% pot bet prices draws below their equity breakeven point:
  - Flush draw (9 outs): ~35% equity needs > 26% pot odds to call profitably.
  - At 67% bet: required equity = 40%. The draw cannot call profitably.

---

## Worked Examples

### Example A — Dry board, strong hand: Small c-bet
**Hand:** A♠ Q♣ (top pair top kicker)
**Board:** A♦ 7♣ 2♥ (rainbow, uncoordinated)
**Position:** Button (3-better). Pot: 22 BB. Stack: 70 BB. SPR ≈ 3.2.

**Decision: C-bet ~7 BB (~33%)**
Villain's BB calling range rarely hits this board. A 7 BB c-bet starts building a pot
you'll likely win. If raised, you can call or re-evaluate; if called, you bet the turn
with top pair in a manageable pot.

---

### Example B — Wet board, strong hand: Large c-bet
**Hand:** K♥ K♣ (overpair)
**Board:** Q♦ J♥ 9♦ (two diamonds, connected straight draws)
**Position:** Button (3-better). Pot: 18 BB. Stack: 60 BB. SPR ≈ 3.3.

**Decision: C-bet ~12 BB (~67%)**
Villain has many draws: T8s, K8s, 98s, flush draws. A 12 BB bet denies all of these
cheap equity. If called by Qx or Jx, you still have the best hand and the pot is
building toward commitment naturally.

---

### Example C — Any board, weak hand: Check back
**Hand:** A♣ J♦ (total miss — no pair)
**Board:** Q♥ 8♦ 4♣ (dry)
**Position:** Button (3-better). Pot: 16 BB. Stack: 55 BB. SPR ≈ 3.4.

**Decision: Check**
With SPR 3.4, a 33% c-bet (5.3 BB) puts a meaningful chunk of your stack in with no
equity. If villain check-raises (pot-committing you), you cannot fold profitably. Check
back and hope the turn gives you equity or a free showdown.

---

## Common Mistakes

1. **C-betting 100% of your range** — checking back weak hands in 3-bet pots protects
   your checking range and avoids low-SPR commitment with no equity.
2. **Using single-raised pot sizing in 3-bet pots** — a 50% pot c-bet that is fine at
   SPR 12 becomes a dangerous stack-off at SPR 3.
3. **Ignoring board texture** — the same hand plays very differently on a dry vs wet
   board in a 3-bet pot.

---

## Engine Modelling Notes

- Always a flop scenario (3 board cards).
- Hero is always on the Button (the 3-better); villain is BB (the caller).
- Two board textures: `FlopTexture::Dry`, `FlopTexture::Wet` (equal probability).
- Two hand strengths: `FlopStrength::Strong`, `FlopStrength::Weak` (equal probability).
- Four scenarios: Dry/Strong → small cbet, Wet/Strong → large cbet,
  Dry/Weak → check, Wet/Weak → check.
- Three answer options: Check back, Small c-bet (~33%), Large c-bet (~67%).
- `current_bet = 0` (villain checks to hero).
- SPR is displayed in the question for context.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [2 — Postflop Continuation Bet](02_postflop_continuation_bet.md) | Single-raised pot c-bet; same sizing logic but different SPR context |
| [6 — Turn Barrel Decision](06_turn_barrel_decision.md) | After a 3-bet pot flop c-bet, the turn barrel decision follows |
| [12 — Big Blind Defense](12_big_blind_defense.md) | The caller in this topic is the BB defender — understanding their range helps calibrate the c-bet |
