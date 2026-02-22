# Topic 9 — Anti-Limper Isolation

**Enum variant:** `TrainingTopic::AntiLimperIsolation`
**Scenario ID prefix:** `AL-`
**Street:** Preflop
**Hero positions:** CO, BTN, or SB
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **limp** is when a player enters the pot by just calling the big blind instead of
raising. Against limpers, the correct aggressive response is often an **isolation raise**
(iso-raise): a raise designed to play heads-up against the weakest player in the field.

> "Punish limpers. Never let them see a cheap flop."

Limpers have shown weakness. They either have a speculative hand or lack the confidence
to raise. Either way, their range is capped — they would have raised premiums. An
iso-raise exploits this by:
1. **Denying multi-way pots** — reduce the number of players, improving your equity.
2. **Taking initiative** — the aggressor controls the action on every street.
3. **Defining your range** — a raise builds a pot with a clear narrative.
4. **Stealing** — the limper(s) may simply fold, awarding you the dead money.

---

## Why Limping Is Usually Wrong

When you limp, you:
- Invite every player behind you to enter the pot cheaply.
- Sacrifice initiative and the c-bet advantage.
- Play a multi-way pot where your hand loses equity (even AA is only 35% vs 3 opponents).
- Allow the BB to see a free or cheap flop with any two cards.

**The one exception:** Overlimping (calling behind other limpers) with very speculative
hands (small pairs, suited connectors) from the SB or BTN can be correct in some
live game environments — but this is a specific exploit, not a default strategy.

---

## Isolation Raise Sizing

Adjust the iso-raise size upward for each additional limper:

| Limpers in front | Correct iso-raise size |
|-----------------|------------------------|
| 1 limper | 4 BB |
| 2 limpers | 5 BB |
| 3 limpers | 6 BB |

**Why increase for more limpers?**
Each limper increases the total dead money in the pot, but also increases the chance
that one of them calls. A larger raise is needed to make the pot odds unfavourable for
speculative hands and to deter multiple callers.

---

## Position Matters

| Position | Advantages | Iso-raise range |
|----------|-----------|-----------------|
| **BTN** | Acts last postflop always | Widest — isolate with playable+ hands |
| **CO** | Acts 2nd-last postflop | Wide — similar to BTN |
| **SB** | Acts first postflop | Tighter — disadvantaged OOP |

From the BTN you can iso-raise a wider range because you have positional advantage on
all postflop streets. From the SB, you'll be OOP every street — this narrows the
profitable isolating range considerably.

---

## Hand Categories and Correct Action

| Hand category | IP (CO/BTN) | OOP (SB) |
|--------------|-------------|----------|
| Premium (AA, KK, QQ, AKs) | **Iso-raise** | **Iso-raise** |
| Strong (JJ, TT, AQ+) | **Iso-raise** | **Iso-raise** |
| Playable (99–77, AJs, KQs, suited connectors) | **Iso-raise** | **Overlimp** |
| Marginal (66–22, KJo, weak aces) | **Fold** | **Fold** |
| Trash | **Fold** | **Fold** |

**Key distinction for Playable hands:**
- From BTN/CO — iso-raise, because you'll have position.
- From SB — overlimp (call behind all limpers), because going OOP with a medium hand
  in a bloated pot is a losing proposition.

**Why never iso-raise marginal hands?**
Even with one limper, marginal hands (22, KJo, A5o) don't have enough postflop playability
to justify building a large pot. You create tough spots when called and end up playing
a big pot OOP with a dominated or low-equity hand. Fold is almost always correct.

---

## Worked Examples

### Example A — Premium Hand from BTN: Iso-Raise
**Hand:** A♥ K♦ (AKo — Strong)
**Position:** Button
**Limpers:** 1 (UTG)
**Stack:** 100 BB

**Decision: Raise to 4 BB**
AKo is a mandatory iso-raise from any position. 4 BB puts the limper in an uncomfortable
spot — they almost certainly limp-folded an Ax hand or a suited connector, both of which
AKo dominates. Even if called, you'll have position postflop with the best hand more
often than not.

---

### Example B — Playable Hand from BTN: Iso-Raise
**Hand:** 8♠ 7♠ (87s — Playable)
**Position:** Button
**Limpers:** 2 (UTG, HJ)
**Stack:** 80 BB

**Decision: Raise to 5 BB**
From the Button, 87s has great playability IP: you flop draws, make straights and
flushes, and play well in position. 5 BB (2 limpers) denies limpers' cheap flop, wins
the pot sometimes uncontested, and sets up a great postflop spot. Do not overlimp — that
would invite the blinds in for free and defeat the purpose.

---

### Example C — Playable Hand from SB: Overlimp
**Hand:** 9♦ 8♦ (98s — Playable)
**Position:** Small Blind
**Limpers:** 2 (UTG, CO)
**Stack:** 80 BB

**Decision: Overlimp (call 1 BB)**
The same hand that iso-raises from BTN just overlimps from SB. Why?
- You are permanently OOP postflop.
- Iso-raising to 5 BB from SB builds a large pot that you'll play from the worst
  position at the table.
- 98s plays well multiway with implied odds — you can call 1 BB and flop a draw.
- The BB may still squeeze, but your 1 BB investment is small.

---

### Example D — Marginal Hand: Fold
**Hand:** K♦ J♠ (KJo — Marginal)
**Position:** Cutoff
**Limpers:** 1 (UTG)
**Stack:** 100 BB

**Decision: Fold**
KJo might look playable, but iso-raising into a UTG limper is problematic:
- UTG limpers often have Kx or Jx hands that dominate KJo.
- If called, you'll often face KQ, KT, AJ type hands postflop.
- Even from CO, the post-flop complexity with a marginal OOP-like hand is too high.
- Limping is even worse — it creates a multi-way pot where KJo performs poorly.

Fold and wait for a better spot.

---

### Example E — Multiple Limpers, Premium Hand from SB
**Hand:** Q♥ Q♣ (QQ — Premium)
**Position:** Small Blind
**Limpers:** 3 (UTG, HJ, CO)
**Stack:** 100 BB

**Decision: Raise to 6 BB (3 limpers)**
QQ is a mandatory iso-raise regardless of position or limper count. You want to:
- Thin the field (QQ plays better heads-up or 3-way than 5-way).
- Build a large pot with the best hand.
- Deny speculative hands (small pairs, suited connectors) the cheap flop they want.

Even from SB OOP, QQ's raw equity advantage over any calling range justifies the large
raise. 6 BB (3 limpers) is the correct size.

---

## The Cost of Overlimping with Premiums

Imagine you hold AA in the SB and three players limp. If you overlimp:
- 4 players see the flop.
- AA is only ~50% to win 4-way.
- A random low board (6-4-2) may have hit multiple players.

You limp AA and lose with it — a classic "cooler" that was entirely preventable with a
raise. **Always raise premium and strong hands regardless of limpers.**

---

## Counterpressure: What Happens After You Iso-Raise?

If a limper re-raises (squeeze): treat it as a 3-bet spot (see Topic 1). Their squeeze
range tends to be tighter than a cold 3-bet range — adjust accordingly.

If multiple limpers call: play postflop in position (if BTN/CO) with a hand that likely
has range advantage vs their wide limping ranges.

---

## Common Mistakes

1. **Iso-raising marginal hands** — KJo, QTo, and weak aces leak money through domination.
2. **Using a flat open-raise size into limpers** — 2.5 BB open is too small; limpers
   need to be charged at 4 BB+ to make their implied odds incorrect.
3. **Overlimping with premiums** — the single most profitable leak to fix.
4. **Iso-raising from SB with medium hands** — building a large OOP pot with 97s is a
   losing proposition long-term.
5. **Forgetting to increase size per limper** — 4 BB vs 1 limper is fine; 4 BB vs 3
   limpers gives everyone correct odds.

---

## Engine Modelling Notes

- Hero position: CO, BTN, or SB (sampled randomly).
- Limper count: 1, 2, or 3 (sampled randomly).
- Hand classification: inline 5-category logic (same as preflop module).
- Iso-raise size displayed dynamically: 4 BB for 1 limper, 5 BB for 2, 6 BB for 3.
- Answers: Fold, Overlimp (call), Iso-raise to N×BB.
- Correct: Iso-raise for Premium/Strong always; Iso-raise for Playable + IP; Overlimp
  for Playable + SB; Fold for Marginal/Trash.
