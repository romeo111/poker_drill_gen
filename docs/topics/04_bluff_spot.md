# Topic 4 — Bluff Spot

**Enum variant:** `TrainingTopic::BluffSpot`
**Scenario ID prefix:** `BL-`
**Street:** River (5 board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **bluff** is a bet or raise made with a hand that has little or no chance of winning at
showdown. The bluff wins when villain folds a hand that would have beaten you.

The river is the most important street for bluffing because:
1. There are no more draws — your hand is what it is.
2. Villain must make a binary decision (call or fold) — no more card-to-come hope.
3. Bet sizing can be calibrated precisely to the fold frequency needed.

---

## The Bluff Math

For a bluff to be profitable, villain must fold often enough:

```
Required fold frequency = Bet / (Pot + Bet)
```

**Example:** Pot = 100 chips. You bluff 75 chips.
```
Required fold frequency = 75 / (100 + 75) = 75 / 175 = 42.9%
```
Villain must fold more than 42.9% of the time for the bluff to show profit.

| Bet size (% pot) | Required fold frequency |
|-----------------|------------------------|
| 33%             | 25%                    |
| 50%             | 33%                    |
| 75%             | 43%                    |
| 100%            | 50%                    |
| 150%            | 60%                    |

Larger bets need villain to fold more often to break even but extract more when they do.

---

## Stack-to-Pot Ratio (SPR) and Fold Equity

**SPR = Remaining stack ÷ Current pot**

SPR is the key factor in determining whether a bluff can be profitable:

| SPR | Fold equity | Bluffing guidance |
|-----|------------|-------------------|
| < 2 | Very low | Check/give up; villain pot-committed |
| 2–4 | Moderate | Smaller bluffs only |
| 4–8 | Good | 75% pot bluffs viable |
| > 8 | Excellent | Large/overbet bluffs most potent |

When SPR is low, villain is likely pot-committed and calling with most hands. Bluffing
here burns chips without fold equity.

---

## The Three Bluff Archetypes

### 1. Missed Flush Draw
You called with a draw, bricked the river, and now have zero showdown value.

**Strengths as a bluff hand:**
- You hold suits that would be in villain's value range if you had made the flush.
- Your hand has no showdown value — you can't "check and see."

**Example:**
- Board: A♥ K♥ 7♥ 4♣ 2♠
- Hand: Q♥ J♦ — missed flush draw, two clean outs bricked
- Bet 75% pot representing AK, set, or made flush

---

### 2. Capped Range
You checked the turn, capping your range (representing that you don't have the nuts).
Now you cannot credibly represent strong hands on the river.

**Why this is NOT a good bluff spot:**
Villain knows your turn check means you don't have AA, sets, or the nut flush. If you
now bet the river, your range is uncredible. Villain can call wide.

**Correct play:** Check behind. Cut losses. Accept the showdown.

---

### 3. Overcard Brick
You held two high cards (like AQ on a low board), missed top pair, and now face the river.

**Semi-credible as a bluff when:**
- You are in late position (BTN, CO).
- Board is ace-high or king-high and you "could" have a set or two pair.
- SPR is high enough for fold equity to exist.

---

## Sizing Your Bluff

Two schools of thought:

**Polarised sizing (large/overbet):** Match the sizing of your value hands. If your value
range bets 75% pot, your bluffs should too. Villain cannot exploit you by calling wider
since they can't distinguish value from bluff.

**Balanced approach:** Use a ratio of roughly 1 bluff : 2 value bets (geometric bluffing
frequency). This means about 33% of your large river bets are bluffs — enough that villain
can't purely fold, but not so many that you're lighting money on fire.

---

## Worked Examples

### Example A — Clear Bluff Opportunity
**Hand:** 8♥ 7♥ (missed flush draw)
**Board:** K♥ Q♥ 5♦ 3♣ 2♠
**Position:** Button (IP)
**Pot:** 200 chips. Stack: 400 chips. SPR = 2.0.

**Decision: Check (give up)**
SPR of 2.0 means villain needs only 33% equity to call a pot-sized bet. Their KQ, K5,
Q5 calling range has far more than 33%. Bluffing here burns 200 chips into a wall.

---

### Example B — Classic Large Bluff
**Hand:** A♠ Q♣ (bricked overcards)
**Board:** J♥ 8♦ 4♣ 2♥ 3♠
**Position:** Button (IP)
**Pot:** 120 chips. Stack: 600 chips. SPR = 5.0.

**Decision: Bet 75% pot (90 chips)**
Required fold frequency: 90 / 210 = 42.9%. With SPR = 5.0 villain is not pot-committed.
You represent a strong range — from BTN you could have 55, 44, A4s, 32s, A3s completing
the wheel. Villain holding J9 or T8 likely folds under this pressure.

---

### Example C — Capped Range, Check Behind
**Hand:** K♦ J♠
**Board:** A♥ Q♣ 5♦ 9♠ 2♣
**Position:** Button. You checked the turn, giving up the hand story.

**Decision: Check behind**
Your turn check signals you don't have AQ, AA, QQ. Betting the river now is uncredible.
Villain can exploit by calling with Ax, Qx, or any pair. Check and accept the showdown
loss.

---

## Blockers — The Advanced Layer

**Blockers** are cards in your hand that reduce the combinations of strong hands in
villain's range.

If you hold A♠ K♠ on a board of A♥ Q♣ J♦ 2♣ 9♠, you block AQ (only 3 combos instead
of 12), making it less likely villain has two pair or the nut straight. This makes your
bluff more credible.

Key blocker types:
- **Nut blockers** — hold one card to the best hand (A♣ on a flush board).
- **Top pair blockers** — hold an ace or top card that villain's two-pair hands need.
- **Set blockers** — hold a pocket pair that reduces villain's set combinations.

---

## Common Mistakes

1. **Bluffing into a capped range** — if you showed weakness on the turn, don't bluff
   the river. The story doesn't add up.
2. **Bluffing at too low SPR** — pot-committed villains call regardless of your sizing.
3. **Bluffing too frequently** — if you bluff more than ~33% of your betting range,
   observant opponents will exploit by always calling.
4. **No thought about blockers** — the best bluff hands block villain's calling combos.

---

## Engine Modelling Notes

- Always a river scenario (5 board cards dealt).
- Three bluff archetypes: `MissedFlushDraw`, `CappedRange`, `OvercardBrick`.
- SPR calculated from randomly sampled stack and pot.
- Correct answer: `Check` when SPR < 2 or `CappedRange`; `Large bluff (75% pot)` otherwise.
- Answers: Check (give up), Small bluff (40% pot), Large bluff (75% pot), All-in shove.
- All-in shove is never the correct answer in the engine — it serves as a trap for
  overaggressive tendencies.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [6 — Turn Barrel Decision](06_turn_barrel_decision.md) | Building a bluffing story across streets; turn decisions feed into river bluff spots |
| [8 — Semi-Bluff Decision](08_semi_bluff_decision.md) | Earlier-street version of applying pressure without a made hand |
