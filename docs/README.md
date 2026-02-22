# Poker Drill Generator — Topic Documentation

This directory contains one reference document per training topic. Each doc covers
the core poker principle, decision framework, worked examples, and how the engine
models the scenario.

---

## Topic Index

| # | Topic | Street | Enum Variant | Scenario ID Prefix |
|---|-------|--------|-------------|-------------------|
| 1 | [Preflop Decision](topics/01_preflop_decision.md) | Preflop | `PreflopDecision` | `PF-` |
| 2 | [Postflop Continuation Bet](topics/02_postflop_continuation_bet.md) | Flop | `PostflopContinuationBet` | `CB-` |
| 3 | [Pot Odds & Equity](topics/03_pot_odds_and_equity.md) | Flop | `PotOddsAndEquity` | `PO-` |
| 4 | [Bluff Spot](topics/04_bluff_spot.md) | River | `BluffSpot` | `BL-` |
| 5 | [ICM & Tournament Decision](topics/05_icm_tournament_decision.md) | Preflop | `ICMAndTournamentDecision` | `IC-` |
| 6 | [Turn Barrel Decision](topics/06_turn_barrel_decision.md) | Turn | `TurnBarrelDecision` | `TB-` |
| 7 | [Check-Raise Spot](topics/07_check_raise_spot.md) | Flop | `CheckRaiseSpot` | `CR-` |
| 8 | [Semi-Bluff Decision](topics/08_semi_bluff_decision.md) | Flop/Turn | `SemiBluffDecision` | `SB-` |
| 9 | [Anti-Limper Isolation](topics/09_anti_limper_isolation.md) | Preflop | `AntiLimperIsolation` | `AL-` |

---

## Quick-Start Glossary

| Term | Definition |
|------|-----------|
| **BB** | Big blind — the base unit of bet sizing in cash games |
| **SPR** | Stack-to-pot ratio: remaining stack ÷ pot |
| **IP** | In position — acts last postflop |
| **OOP** | Out of position — acts first postflop |
| **c-bet** | Continuation bet — a bet by the preflop aggressor on the flop |
| **Equity** | Your probability of winning the pot at showdown |
| **EV** | Expected value — long-run average profit of an action |
| **ICM** | Independent Chip Model — converts tournament chips to prize-money equity |
| **OESD** | Open-ended straight draw — eight outs to complete on either end |
| **Fold equity** | The extra EV gained when a bet forces villain to fold hands they would otherwise win with |
| **Pot odds** | The ratio of the call amount to the total pot after calling |
