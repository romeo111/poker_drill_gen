# Topic 16 — Delayed C-Bet

**Enum variant:** `TrainingTopic::DelayedCbet`
**Scenario ID prefix:** `DC-`
**Street:** Turn (4 board cards: 3 flop + 1 turn)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **delayed c-bet** occurs when the preflop raiser (Hero, in position on the Button)
checks back the flop and then bets the turn. Unlike a standard c-bet (fired on the flop)
or a turn barrel (fired after a flop c-bet), the delayed c-bet exploits the information
gained from the flop check-through:

1. Villain checked the flop twice (preflop and flop) — their range may be weak or passive.
2. Hero checked back the flop — this deceptively conceals hand strength.
3. On the turn, Hero can now leverage positional advantage with a delayed bet.

This is distinct from **Turn Barrel (TB-)** which fires after a flop c-bet, and from
**Turn Probe Bet (PB-)** where the OOP player leads into the in-position player.

---

## Why Check the Flop to Bet the Turn?

Reasons for delaying the c-bet include:

- **Board texture was unfavorable on the flop** — a wet flop with draws everywhere
  favors the caller's range. Checking avoids bloating the pot in a bad spot.
- **Pot control with medium hands** — keep the pot small on the flop and re-evaluate
  after the turn card.
- **Deception** — checking back disguises hand strength. Villain may interpret the
  check-back as weakness and pay off a turn bet with a wider range.
- **Turn card changes dynamics** — a blank turn narrows villain's perceived range,
  while a scare card may give Hero new fold equity.

---

## Decision Matrix

| Hand Strength | Turn Card | Correct Action | Sizing | Rationale |
|--------------|-----------|----------------|--------|-----------|
| Strong (overpair, top pair good kicker, two pair, set) | Any | Medium delayed c-bet | ~60% pot | Extract value; charge draws; build pot for river |
| Medium (middle pair, weak top pair, underpair) | Blank | Small delayed c-bet | ~33% pot | Thin value; deny equity; keep pot manageable |
| Medium | Scare card | Check | — | Pot control; scare card devalues medium hand |
| Weak (missed, low pair, air) | Any | Check | — | Save chips; no equity to justify betting |

---

## Hand Strength Classification

Unlike some topics that randomly assign strength, this module classifies Hero's actual
dealt hand against the board:

- **Strong:** Set (pocket pair + one on board), two pair, overpair, top pair with
  good kicker (J+)
- **Medium:** Middle pair, weak top pair (low kicker), underpair
- **Weak:** No pair, missed entirely

---

## Turn Card Classification

- **Blank:** Turn card is below the highest flop card, doesn't complete flush or
  straight draws
- **Scare card:** Overcard to the flop, third card of same suit (flush possible),
  or four-straight on board

---

## Worked Examples

### Example A — Strong hand: Medium delayed c-bet
**Hand:** K♠ K♥ (overpair)
**Board:** Q♣ 8♦ 3♠ 5♥ (blank turn)
**Position:** Button (IP). Pot: 20 BB. Stack: 80 BB.

**Decision: Delayed c-bet ~12 BB (~60%)**
You have an overpair that dominates villain's likely range after two checks. The blank
turn doesn't change anything — bet to extract value and charge any draws.

---

### Example B — Medium hand, blank turn: Small delayed c-bet
**Hand:** T♣ 9♣ (middle pair on T-high flop)
**Board:** T♦ 7♠ 2♥ 4♣ (blank turn)
**Position:** Button (IP). Pot: 16 BB.

**Decision: Delayed c-bet ~5 BB (~33%)**
Middle pair is decent but vulnerable. A small bet denies free cards to overcards and
extracts thin value from worse pairs, without over-committing.

---

### Example C — Medium hand, scare turn: Check
**Hand:** 9♠ 9♦ (pocket nines, underpair)
**Board:** T♣ 7♥ 3♠ A♦ (ace on the turn)
**Position:** Button (IP). Pot: 18 BB.

**Decision: Check**
The ace is a scare card — villain may have an ace in their checking range. Your nines
went from a decent underpair to a very vulnerable hand. Check for pot control.

---

### Example D — Weak hand: Check
**Hand:** J♠ 8♦ (missed, no pair)
**Board:** Q♣ 6♥ 2♦ K♠
**Position:** Button (IP). Pot: 14 BB.

**Decision: Check**
You have no pair and no draw. Betting here is a pure bluff into a player who may have
a queen or a king. Save your chips.

---

## Common Mistakes

1. **Always c-betting the flop** — sometimes checking back to delay is higher EV,
   especially on wet boards or with medium hands.
2. **Never delayed c-betting** — checking back the flop and then checking the turn
   again with a strong hand gives away too much value.
3. **Betting medium hands on scare turns** — when the turn improves villain's range,
   pot control is preferred.
4. **Bluffing with air** — unlike a flop c-bet where fold equity is high, a delayed
   c-bet faces a villain who has seen two free cards and may have improved.

---

## Engine Modelling Notes

- Always a turn scenario (4 board cards: 3 flop + 1 turn).
- Hero is always on the Button (IP); villain is in the Big Blind.
- Hero raised preflop, villain called. Hero checked back the flop.
- Hand strength is classified from the actual dealt cards vs board (not random).
- Turn card is classified as Blank or Scare based on board dynamics.
- Three answer options: Check, Small delayed c-bet (~33%), Medium delayed c-bet (~60%).
- `current_bet = 0` (villain checks to hero on the turn).
- `branch_key` format: `"{Strength}:{TurnType}"` — e.g., `"Strong:Blank"`, `"Medium:Scare"`.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [6 — Turn Barrel Decision](06_turn_barrel_decision.md) | Fires the turn after a flop c-bet — DC fires after checking the flop |
| [15 — Turn Probe Bet](15_turn_probe_bet.md) | OOP version — BB probes the turn after both players checked the flop |
| [2 — Postflop Continuation Bet](02_postflop_continuation_bet.md) | The flop c-bet that Hero chose NOT to fire in this scenario |
