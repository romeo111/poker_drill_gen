# Topic 11 — Squeeze Play

**Enum variant:** `TrainingTopic::SqueezePlay`
**Scenario ID prefix:** `SQ-`
**Street:** Preflop (no board cards)
**Difficulty range:** Beginner → Advanced

---

## Core Principle

A **squeeze** is a 3-bet made against an open-raiser when one or more players have
cold-called the open. The squeeze has two sources of profit:

1. **Collecting dead money** — the callers' chips are in the pot with no real range
   advantage (they cold-called, so they don't have premium hands).
2. **Isolation** — with a large enough sizing, you often force folds and play a
   big heads-up pot against just the original opener.

---

## Why Callers Matter

The more callers between the opener and you, the larger and more profitable the squeeze:

| Callers | Dead money | Effective squeeze size | Rationale |
|---------|-----------|----------------------|-----------|
| 1 | 1× open | 3× open + 1 caller | Moderate dead money; one player to isolate |
| 2 | 2× open | 3× open + 2 callers | Large dead money; strong profitability |
| 3+ | 3×+ open | 3× open + callers | Enormous dead money; any premium hand profits |

Callers have **capped ranges** — if they had a premium hand they would have 3-bet
themselves. This makes them likely to fold the squeeze.

---

## Hand Requirements

Not every hand profits from a squeeze. The key question is: what do you do when called?

### Premium (AA, KK, QQ, AKs) → Always Squeeze
These hands are equity monsters. Even if all callers call your squeeze (rare), you have
a dominant equity advantage. The pot is often won preflop or with a top-pair-or-better hand.

### Speculative (77–99, suited connectors, AJs) → Call
These hands have good **implied odds** — you want callers in the pot, not to fold them.
Squeezing with 87s creates a large pot where you're a coin flip; calling creates a
multiway pot with high implied odds for sets and draws. Calling is higher EV.

### Weak (off-suit rags, dominated hands) → Fold
Squeezing with trash is a bluff that requires all opponents to fold. Even if it works
sometimes, the risk-to-reward is poor. Fold and wait.

---

## Squeeze Sizing

Standard formula: `3× the open + 1 open per caller`

Example: UTG opens 3 BB, two callers.
Squeeze = (3 × 3) + 2 = 11 BB

The "+1 per caller" accounts for dead money and ensures the sizing is large enough to
deter floating.

---

## Worked Examples

### Example A — Premium Hand: Squeeze
**Hand:** A♠ A♣
**Situation:** UTG opens 3 BB, CO calls. You are on the Button.
**Pot:** 7.5 BB. Stack: 100 BB.

**Decision: Squeeze to ~10 BB**
You have the nuts preflop. A 10 BB squeeze wins the 7.5 BB pot often, and when called
you play a large pot as a massive favourite. Never flat AA in a squeeze spot.

---

### Example B — Speculative Hand: Call
**Hand:** 8♦ 7♦
**Situation:** UTG opens 3 BB, HJ and CO both call. You are on the Button.
**Pot:** 10.5 BB. Stack: 100 BB.

**Decision: Call (3 BB)**
Three callers mean a multiway pot with excellent implied odds for a straight or flush.
Your equity is good in a bloated pot. Squeezing turns this into a bluff with weak equity
against ranges that 4-bet or call.

---

### Example C — Weak Hand: Fold
**Hand:** Q♥ 4♦
**Situation:** CO opens 3 BB, BTN calls. You are in the Small Blind.
**Pot:** 7 BB. Stack: 80 BB.

**Decision: Fold**
Q4o has poor equity and no implied odds. Even with position as part of the equation,
squeezing is a low-EV bluff and calling creates a dominated hand OOP. Fold.

---

## Common Mistakes

1. **Calling premium hands** — flatting AA/KK in a squeeze spot is the biggest leak;
   you're surrendering massive EV.
2. **Squeezing speculative hands** — turns a good implied-odds call into a bluff; stack
   bloats without equity dominance.
3. **Under-sizing the squeeze** — a 2× open squeeze is not enough to fold out callers;
   use 3× + callers.
4. **Ignoring stack depth** — at short stacks (< 30 BB), squeezes commit you; adjust
   hand requirements accordingly.

---

## Engine Modelling Notes

- Always preflop (no board cards).
- Hero is always on the Button; one opener from UTG.
- Three hand strengths: `Premium`, `Speculative`, `Weak` (uniform distribution).
- Number of callers: 1 (Beginner), 1–2 (Intermediate), 1–3 (Advanced).
- Correct answers: Premium → Squeeze, Speculative → Call, Weak → Fold.
- Three answer options: Fold, Call, Squeeze to calculated size.
- `current_bet` = open raise amount (hero faces this raise + callers).

---

## Related Topics

| Topic | Connection |
|-------|-----------|
| [1 — Preflop Decision](01_preflop_decision.md) | Hand classification and open-raising ranges that feed into squeeze spots |
| [9 — Anti-Limper Isolation](09_anti_limper_isolation.md) | Similar isolation logic but against limpers rather than callers |
| [12 — Big Blind Defense](12_big_blind_defense.md) | Both are preflop 3-bet spots; BB defense faces a single raiser while squeeze involves callers |
