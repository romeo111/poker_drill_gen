# Topic 1 — Preflop Decision

**Enum variant:** `TrainingTopic::PreflopDecision`
**Scenario ID prefix:** `PF-`
**Street:** Preflop
**Difficulty range:** Beginner → Advanced

---

## Core Principle

Every poker hand starts preflop. The two hole cards you're dealt, your position at the
table, and your stack depth together determine whether you should fold, call, or raise
before the community cards appear.

Getting preflop fundamentals right is the single highest-leverage skill in poker because
every mistake made preflop compounds through three more streets. A hand you shouldn't
have played costs you chips on the flop, turn, and river too.

---

## The Five Hand Categories

The engine classifies any two-card hand into one of five tiers:

| Category | Example hands | Baseline action |
|----------|--------------|-----------------|
| **Premium** | AA, KK, QQ, AKs | Always raise; 3-bet/4-bet for value |
| **Strong** | JJ, TT, AQs, AKo, AQo | Raise; 3-bet in most spots |
| **Playable** | 99–77, AJs, KQs, suited connectors (76s+) | Raise from late position; consider folding from early |
| **Marginal** | 66–22, KJo, QJo, weak aces | Position-dependent; lean fold from early, can open from BTN/SB |
| **Trash** | 72o, 83o, 94o | Fold in almost all spots |

---

## Position is Everything

Your seat at the table determines when you act. Acting **last** (late position) gives you
information: you see what everyone else does before you commit chips.

```
Early position (UTG, UTG+1, UTG+2)  →  tightest range
Middle position (Lojack, Hijack)     →  moderate range
Late position (Cutoff, Button)       →  widest range
Blinds (SB, BB)                      →  defend wide, but OOP postflop
```

**Rule of thumb:** You can profitably open-raise hands from the Button that you must fold
from UTG, simply because positional advantage is worth ~1–2 BB in EV over an entire hand.

---

## The Three Preflop Spots the Engine Trains

### 1. Open-Raise Spot
Action folds to you. The decision: raise, limp, or fold.

- **Raise** — almost always preferred over limping. Builds a pot with the initiative.
- **Limp** — generally a leak. Invites multiway pots without initiative; gives the BB a
  free squeeze opportunity.
- **Fold** — correct for weak hands from early position.

**Sizing:** 2.5–3× BB from most positions when 40+ BB deep. Short stacks (< 20 BB) often
open to 2× or shove directly.

### 2. Facing an Open Raise
A player has raised before you. Options: fold, flat-call, or 3-bet.

- **Fold** — for marginal/trash hands, or playable hands out of position.
- **Call** — playable hands in position, speculative hands with implied odds (suited
  connectors, small pairs).
- **3-bet** — premium/strong hands for value; sometimes bluff-3-bet with blockers.

**3-bet sizing:** 3× the open from in-position; 3.5–4× from out-of-position.

### 3. Facing a 3-Bet
You opened; now someone re-raises (3-bets). Options: fold, call, or 4-bet.

- **4-bet** — premium hands (AA, KK, sometimes QQ/AKs) for value.
- **Call** — strong/playable hands with position and implied odds.
- **Fold** — marginal/trash hands; avoid calling 3-bets OOP without a strong hand.

---

## Worked Examples

### Example A — Open-Raise from the Button
**Hand:** K♥ Q♦ (KQo) — Marginal
**Position:** Button (6-max)
**Stack:** 100 BB
**Action:** Folds to you

**Decision:** Raise to 2.5 BB
**Why:** KQo from the Button is a clear open. You have position on the two remaining
players (SB, BB). Even if called you act last on every postflop street. KQo flops top
pair or a draw often enough to be profitable here.

---

### Example B — Facing an Open from UTG
**Hand:** 8♠ 7♠ (87s) — Playable
**Position:** Hijack
**Open by:** UTG (raising to 3 BB)
**Stack:** 80 BB

**Decision:** Fold
**Why:** 87s has good implied odds but UTG's range is very strong (premium/strong hands
mainly). You'll be OOP most of the time postflop, and you risk calling into a 3-bet from
the seats behind. The implied-odds calculation doesn't work here at 80 BB depth without
position.

---

### Example C — Defending BB vs a 3-Bet
**Hand:** A♣ K♠ (AKo) — Strong
**Position:** Big Blind
**Scenario:** You opened to 3 BB from HJ; BTN 3-bets to 9 BB
**Stack:** 120 BB

**Decision:** 4-bet to ~27 BB
**Why:** AKo is a mandatory 4-bet from any position. It blocks AA and KK, is likely
ahead of villain's 3-bet bluffs, and builds a large pot where you have a realistic equity
advantage vs a balanced 3-bet range (~50% equity vs most ranges).

---

## Stack-Depth Adjustments

| Stack depth | Impact |
|-------------|--------|
| **100 BB+** | Standard play; implied odds justify set-mining, suited connectors |
| **40–100 BB** | Still close to normal; some mid-pair speculative plays get dicier |
| **20–40 BB** | Avoid calling with speculative hands; lean toward raise-or-fold |
| **< 20 BB** | Push/fold territory — open-shove instead of raising to 3 BB |

---

## Common Mistakes

1. **Limping** — players limp to "see a cheap flop" but this destroys your range's
   initiative and invites cheap squeezes.
2. **Over-folding in the BB** — you've already invested 1 BB; your pot odds to defend
   are better than any other position.
3. **Under-4-betting premiums** — slowplaying AA preflop is often a mistake at shallow
   depths (< 60 BB); you want the money in with an equity edge.
4. **Ignoring position** — calling a raise OOP with a marginal hand is the number one
   way to bleed chips without realising it.

---

## Engine Modelling Notes

- Hand classification: 5 categories (`HandCategory` enum + `classify_hand()`) defined in `evaluator.rs`; called by `preflop.rs`.
- Scenarios randomly select `OpenRaise`, `FacingOpen`, or `ThreeBetPot` with equal
  probability.
- Stack depth is sampled per difficulty (Beginner: 80–120 BB; Advanced: 15–300 BB).
- Table size alternates 6-max / 9-max randomly; position is sampled from the active pool.
- The engine guarantees exactly one correct answer per scenario using a `correct: &str`
  ID matched against each `AnswerOption`.

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [5 — ICM & Tournament Decision](05_icm_tournament_decision.md) | Preflop push/fold with tournament-adjusted thresholds |
| [9 — Anti-Limper Isolation](09_anti_limper_isolation.md) | Preflop aggression against limpers; uses the same 5-category hand classification |
