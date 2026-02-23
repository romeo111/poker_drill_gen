# Topic 15 — Turn Probe Bet

**Enum variant:** `TrainingTopic::TurnProbeBet`
**Scenario ID prefix:** `PB-`
**Street:** Turn (4 board cards: 3 flop + 1 turn)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **probe bet** is a lead-out bet made by an out-of-position (OOP) player on the turn
after the flop was checked through by both players. This is a **non-standard initiative
take** — normally the preflop aggressor c-bets the flop, but when both players check the
flop, the OOP player can "probe" the turn to:

1. Extract value with medium-to-strong hands.
2. Deny free cards to villain's drawing hands.
3. Reclaim initiative after a passive flop.

---

## Why the Flop Check-Through Changes Everything

When an in-position player (e.g., BTN) **checks back the flop**, their range is
**capped** — they almost certainly do not have:
- Strong top pair (would have c-bet for value)
- Sets (same reason)
- Straights or flushes (same)

Their checking range typically contains:
- Medium pairs (middle pair, second pair)
- Floating hands (two high cards waiting for a turn card)
- Speculative draws they chose to take free cards with

This capped range is **vulnerable to a probe**. You are OOP but your range is *not*
capped — you can have any hand that you checked on the flop (including strong made hands
you slowplayed).

---

## Probe Bet Sizing Guide

| Hero hand | Correct action | Rationale |
|-----------|----------------|-----------|
| Strong (top pair+, strong draw with equity) | Probe large (~70% pot) | Build the pot; charge draws; collect thin value from villain's pairs |
| Medium (middle pair, weak draw) | Probe small (~40% pot) | Semi-bluff/thin value at low risk; foldable if raised |
| Weak (bottom pair, air) | Check | No equity; probing into a capped but medium-strength range is a bluff with poor backing |

---

## The Math Behind Probe Sizing

**Probe large (70%):**
Required fold frequency = 70 / (100 + 70) = 41%. Villain's capped range folds often
enough — they frequently have nothing (air, two-overcards) or one pair that cannot face
continued aggression.

**Probe small (40%):**
Required fold frequency = 40 / (100 + 40) = 28.5%. A much lower bar — villain needs
to fold barely over a quarter of their range for this to break even. Even when called
you often have equity to continue on the river.

---

## Worked Examples

### Example A — Strong hand: Large probe
**Hand:** A♣ Q♦ (top pair top kicker)
**Board:** Q♠ 8♥ 4♣ K♦ (turn brings a K)
**Position:** Big Blind (OOP). Pot: 24 BB. Stack: 80 BB.

**Decision: Probe ~17 BB (~70%)**
Your top pair is well ahead of villain's checking range (middle pairs, floats). The K
may have helped villain's floats slightly, but you still dominate with TPTK. A 70% probe
extracts value now and builds the pot for a manageable river decision.

---

### Example B — Medium hand: Small probe
**Hand:** J♠ 9♦ (open-ended straight draw — some equity)
**Board:** T♣ 7♥ 3♠ 8♣ (turn completes your OESD to 9-high)
**Position:** Big Blind (OOP). Pot: 18 BB. Stack: 60 BB.

**Decision: Probe ~7 BB (~40%)**
Wait — if the turn completed your straight, this is a strong hand, not medium. Adjust
accordingly. But with J9 not completing (say the board is T♣ 7♥ 3♠ 2♣), you have
a gutshot and some backdoor equity. A small probe (~40%) applies pressure without
committing, and folds out villain's pure air.

---

### Example C — Weak hand: Check
**Hand:** K♦ 5♠ (overcards only, missed pair)
**Board:** 9♣ 6♥ 2♦ J♠
**Position:** Big Blind (OOP). Pot: 14 BB.

**Decision: Check**
Your hand has minimal equity (overcard draws only). A probe here is a pure bluff into
a player who checked back — likely a medium pair. Without a draw to fall back on if
called, there is no justification for betting. Check and hope for a free river card.

---

## Common Mistakes

1. **Never probing OOP** — failing to use the turn as an OOP weapon after a checked flop
   is passive. Villain's capped range is exploitable.
2. **Always probing OOP** — probing with weak hands when villain's checking range still
   contains many medium pairs burns chips without justification.
3. **Wrong sizing** — a 33% probe is too small to deny free cards; a 100% probe
   over-commits on a hand where you want to remain flexible.
4. **Forgetting the river** — a probe is a commitment to fight for this pot. Plan your
   river action before betting.

---

## Engine Modelling Notes

- Always a turn scenario (4 board cards: 3 flop + 1 turn).
- Hero is always in the Big Blind (OOP); villain is on the Button.
- Both players checked the flop (noted in question text).
- Three hand strengths: `Strong`, `Medium`, `Weak` (uniform distribution).
- Correct answers: Strong → Probe large (~70%), Medium → Probe small (~40%), Weak → Check.
- Three answer options: Check, Probe small (~40%), Probe large (~70%).
- `current_bet = 0` (hero acts first; no action from villain yet).

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [6 — Turn Barrel Decision](06_turn_barrel_decision.md) | The IP version of this topic — BTN as aggressor deciding whether to barrel the turn |
| [7 — Check-Raise Spot](07_check_raise_spot.md) | Another OOP Big Blind decision; check-raise on the flop vs probe bet on the turn |
| [2 — Postflop Continuation Bet](02_postflop_continuation_bet.md) | If neither player c-bet the flop (both checked), the probe situation in T15 can arise |
