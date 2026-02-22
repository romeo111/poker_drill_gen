# Topic 7 — Check-Raise Spot

**Enum variant:** `TrainingTopic::CheckRaiseSpot`
**Scenario ID prefix:** `CR-`
**Street:** Flop
**Hero position:** Big Blind (OOP)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **check-raise** (CR) is when you check, villain bets, and then you raise. It is one of
the strongest plays in poker because it:

1. **Represents strength** — you checked (as if weak), then raised (revealing power or
   deception).
2. **Denies equity** — forces villain to either call a large bet or fold draws/weak pairs.
3. **Builds the pot with strong hands** — when you flop a set or two pair OOP, a
   check-raise is often the highest-EV play.
4. **Creates fold equity as a bluff** — semi-bluff check-raises with draws can win
   outright or build equity for later streets.

---

## Why OOP Check-Raises Matter

The Big Blind (BB) is permanently **out of position** (OOP) postflop — they act first on
every street. This is a fundamental disadvantage: villain always sees your action before
deciding.

The check-raise is the BB's primary weapon to compensate. Without check-raises, the BB
becomes exploitable: villain can bet every flop knowing the BB can only call or fold.
Mixing in check-raises:
- Forces villain to size down their c-bets (risk of the raise).
- Lets the BB represent a range advantage on low boards.
- Protects the BB's calling range (harder to exploit if some calls are traps).

---

## Board Favourability

Whether you should check-raise depends heavily on which player's range hits the board.

| Board type | Who benefits | Example board | BB action tendency |
|-----------|-------------|--------------|-------------------|
| Low, connected | BB (Big Blind) | 6♥ 5♣ 3♦ | Check-raise strong hands; check-raise semi-bluffs |
| High, dry | IP raiser | A♠ K♦ 9♣ | Check-call or fold; rarely check-raise |

**BB-favorable boards** (rank sum ≤ 20) hit the BB's wide calling range: low pairs,
two pair, sets with small pocket pairs. The late-position raiser typically doesn't hold
65, 54, or 33.

**IP-favorable boards** hit the raiser's strong preflop range (AK, AQ, KK, AA). BB's
range can have some of these too, but at much lower frequency.

---

## When to Check-Raise

| Hand strength | Board | Decision | Rationale |
|--------------|-------|----------|-----------|
| Strong (two pair, set, strong pair) | BB-favorable | **Check-raise** | Value + protection |
| Combo draw (flush + straight) | Any board | **Check-raise** | Maximum semi-bluff equity |
| Flush draw or OESD | BB-favorable | **Check-call** | Realize equity; CR is too aggressive without blockers |
| Weak / air | IP-favorable | **Fold** | No equity, wrong board |
| Weak / air | BB-favorable | **Check-call** | Range protection; some equity later |

---

## Check-Raise Sizing

A check-raise should typically be 2.5× villain's bet.

**Example:** Villain bets 60 chips into a 100-chip pot. Your check-raise should be to
approximately 150 chips (2.5 × 60).

Why 2.5×?
- Large enough to make pot-odds calculations wrong for draws without strong equity.
- Small enough that you don't over-commit with semi-bluffs.
- Consistent sizing between value and bluff raises (balanced).

For the exact required equity to call a 2.5× check-raise:
```
Villain bet = 60 into 100 pot → total pot = 160 before CR
Hero CR to 150, villain must call 90 more (150 - 60)
Required equity = 90 / (160 + 90) = 90/250 = 36%
```
Only hands with 36%+ equity can profitably call — most draws cannot.

---

## Worked Examples

### Example A — Strong Hand, BB-Favorable Board: Check-Raise
**Hand:** 6♦ 5♦
**Position:** BB (OOP)
**Flop:** 6♠ 5♥ 3♣ (BB-favorable: rank sum = 14)
**Villain (BTN) bets:** 50 chips into 100-chip pot

**Decision: Check-raise to 125 chips**
You flopped two pair on a BB-favorable board. Villain's range is mostly overcards and
missed high-card hands. Check-raising protects against straights (4-2 or 7-4), extracts
value from villain's top pair, and builds the pot while you're ahead.

---

### Example B — Combo Draw, Any Board: Check-Raise
**Hand:** 9♥ 8♥
**Position:** BB (OOP)
**Flop:** T♥ 7♠ 6♥ (wet: flush draw + open-ended straight draw)
**Villain (BTN) bets:** 70 chips into 100-chip pot

**Decision: Check-raise to 175 chips**
You have a combo draw: 9-high flush draw + open-ended straight (J8 or 56 for the
straight; 8 hearts for the flush). Combo draws have ~54% equity on the flop — you are
a favourite! Check-raising applies pressure (fold equity), builds a pot you're likely
to win, and makes it expensive for villain to realise equity with weaker draws.

---

### Example C — Flush Draw Only, Check-Call
**Hand:** A♠ 4♠
**Position:** BB (OOP)
**Flop:** K♠ 7♠ 2♦ (flush draw)
**Villain (BTN) bets:** 50 chips into 100-chip pot

**Decision: Check-call**
You have a nut flush draw (any spade wins). Check-calling is correct here rather than
check-raising because:
1. The board is high (K on top) — villain's range is strong here.
2. A check-raise bluff/semi-bluff gets called by KQ, KJ which have strong equity vs you.
3. Calling preserves stack and lets you realise ~35% equity cheaply.

---

### Example D — Weak Hand, IP-Favorable Board: Fold
**Hand:** 7♣ 4♦
**Position:** BB (OOP)
**Flop:** A♠ K♦ 9♣ (high, dry: rank sum = 32)
**Villain (BTN) bets:** 60 chips into 100-chip pot

**Decision: Fold**
You have no pair, no draw, and no equity on an ace-king high board. Villain's late-
position range connects well here (AK, A9, K9, AA, KK). Check-calling burns 60 chips
with ~5% equity. Check-raising is a pure bluff against a range that won't fold. Fold.

---

## The Check-Raise as a Range-Protection Tool

Even if you don't execute a check-raise often, the *threat* of a check-raise makes your
entire BB checking range stronger. If villain knows you never check-raise, they can
bet every flop without fear. Mixing in check-raises:

- Forces smaller c-bets from villain (they fear the raise).
- Allows your check-calls to include both strong hands *and* draws (harder to read).
- Creates imbalance in your favour on boards where your range is strongest.

**Target frequency:** Approximately 1 check-raise for every 3–4 check-calls in spots
where the board favors your range.

---

## Common Mistakes

1. **Never check-raising as BB** — the most common mistake. Turns you into a calling
   station who can be exploited freely.
2. **Check-raising on IP-favorable boards** — check-raising A♠ K♦ 9♣ as BB is usually
   a bluff that gets called by the exact range that beats you.
3. **Too small check-raise sizing** — raising to 1.5× is min-raise territory and gives
   villain good odds to continue. Use 2.5× minimum.
4. **Check-raising without a plan for the turn** — if you check-raise, know what you're
   doing on all turn cards. Don't CR and then give up on the turn.

---

## Engine Modelling Notes

- Hero is always BB (OOP). Villain is always IP.
- Board classified as BB-favorable when rank sum of 3 flop cards ≤ 20.
- Hero hand classified as: Strong (hits low board), Draw (flush or straight), or Weak.
- Combo draw = both flush draw and straight draw present.
- Answers: Fold, Check-call, Check-raise to 2.5× villain bet.
- Correct: CR on (BB-favorable + Strong) or (any board + ComboDraw); Fold on (IP-
  favorable + Weak); Check-call otherwise.
