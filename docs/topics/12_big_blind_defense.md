# Topic 12 — Big Blind Defense

**Enum variant:** `TrainingTopic::BigBlindDefense`
**Scenario ID prefix:** `BD-`
**Street:** Preflop (no board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

The **Big Blind** is the most unique position in poker: you have money already invested
in the pot, so you are getting a direct discount on any call. This changes your
defence frequency dramatically compared to every other position.

The three decisions are:
1. **Fold** — your hand is too weak to justify even the discounted price.
2. **Call** — the pot-odds discount makes defence profitable.
3. **3-bet** — you have enough equity to build a larger pot as the favourite.

---

## The BB Pot-Odds Discount

When villain raises from any position, you already have 1 BB invested. The call price
is `(raise - 1 BB)`, not the full raise. This is the "BB discount":

**Example:** Villain raises to 3 BB. You are in the BB.
- Without the discount: calling 3 BB into a 4.5 BB pot = 40% pot odds required.
- With the BB already posted: calling 2 BB more into a 4.5 BB pot = 30.8% required.

This is why the BB should defend much wider than any other position.

---

## Minimum Defence Frequency (MDF)

MDF tells you the minimum fraction of your range that must continue to prevent villain
from profiting with any two cards:

```
MDF = Pot / (Pot + Bet)
```

**Example:** Villain raises to 3 BB. Pot = 4.5 BB (including blinds and antes).
```
MDF = 4.5 / (4.5 + 3) = 4.5 / 7.5 = 60%
```
You must defend ~60% of your Big Blind range to avoid being exploited.

---

## Range Construction

### Strong (3-bet range): JJ+, AK, AQs
These hands have a large equity advantage over most opening ranges. A 3-bet builds the
pot where you're ahead, may win the dead money, and denies villain a cheap flop with
their suited connectors and small pairs.

### Playable (call range): 22–TT, suited connectors, broadway hands, suited aces
These hands have enough equity with the pot-odds discount to justify calling. Speculative
hands (small pairs, suited connectors) have strong implied odds when they hit. Broadway
and suited broadways have direct equity.

### Weak (fold): Off-suit non-broadway trash (Q2o, J4o, 83o)
Even from the BB, calling with dominated off-suit hands is a long-term leak. The
pot-odds discount does not overcome poor equity and difficult post-flop situations.

---

## Position Matters — Even from the BB

You are always **out of position** postflop when defending the Big Blind. This is a
structural disadvantage that costs you EV on every flop, turn, and river. Narrow your
3-bet range accordingly: 3-bet hands that can win big pots (JJ+, AK) rather than
speculative hands that prefer to play in position.

---

## Raiser Position Adjusts Your Range

| Raiser position | Opening range width | Your defence range |
|----------------|--------------------|--------------------|
| UTG | Tight (13–15%) | 3-bet only AA–QQ, AK; call strong hands |
| CO | Medium (25–28%) | 3-bet JJ+, AQs; call wide pairs/connectors |
| BTN | Wide (40–45%) | 3-bet JJ+, AQ, AJs; call very wide |

Against a wide BTN opener, your calling range expands significantly because villain has
many weak hands in their range that you can profitably call.

---

## Worked Examples

### Example A — Strong: 3-bet
**Hand:** K♣ K♦
**Situation:** BTN raises to 3 BB. Pot: 5 BB (BTN + SB + BB). Stack: 100 BB.

**Decision: 3-bet to 10 BB**
KK has a massive equity advantage over BTN's wide range. Build the pot now. Flatting
lets villain realise equity cheaply with every suited connector in their range.

---

### Example B — Playable: Call
**Hand:** 7♥ 7♣
**Situation:** CO raises to 3 BB. Pot: 5 BB. Stack: 80 BB.

**Decision: Call (2 BB more)**
77 has solid equity and set-mining value. With 80 BB effective, hitting a set gives
you excellent implied odds. Calling from the BB is higher EV than folding.

---

### Example C — Weak: Fold
**Hand:** J♠ 4♦
**Situation:** UTG raises to 3 BB. Pot: 4.5 BB. Stack: 100 BB.

**Decision: Fold**
J4o has poor equity against UTG's tight range (mostly broadways and pairs). Even the
BB discount doesn't rescue a dominated, uncoordinated hand. Fold.

---

## Common Mistakes

1. **Calling too wide without pot-odds justification** — the BB discount is real but
   not unlimited; off-suit trash still loses money.
2. **Not 3-betting strong hands** — flatting JJ+, AK from the BB allows villain to
   realise equity cheaply; build the pot when you're ahead.
3. **3-betting marginal speculative hands** — suited connectors prefer multiway pots
   with implied odds, not bloated heads-up pots OOP.
4. **Ignoring raiser position** — UTG's tight range means you should fold more; BTN's
   wide range means you should call (and 3-bet) more.

---

## Engine Modelling Notes

- Always preflop (no board cards).
- Hero is always in the Big Blind; SB folds (heads-up vs one raiser).
- Three hand strengths: `Strong`, `Playable`, `Weak` (uniform distribution).
- Raiser position is randomised: UTG, CO, or BTN.
- Correct answers: Strong → 3-bet, Playable → Call, Weak → Fold.
- Three answer options: Fold, Call, 3-bet to calculated size.
- `current_bet` = villain's raise amount.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [1 — Preflop Decision](01_preflop_decision.md) | Open-raise ranges that hero faces; understanding the raiser's range width |
| [7 — Check-Raise Spot](07_check_raise_spot.md) | Both are OOP spots from the BB; Big Blind defense extends into postflop decisions |
| [11 — Squeeze Play](11_squeeze_play.md) | Similar 3-bet decision but in squeeze context (open + callers) vs single raiser |
