# User Statistics & Scoring — Methodology

This document defines the data model, scoring formula, adaptive difficulty rules, and progress tracking for the poker drill system. No implementation language is assumed — any backend (file, Redis, SQL) can follow this specification.

---

## Section 1 — What to Record Per Answer

Each time a user submits an answer to a drill, record the following fields:

| Field | Type | Description |
|---|---|---|
| `user_id` | string | Unique identifier for the user |
| `scenario_id` | string | The scenario ID from the drill response (e.g. `PF-3A1B2C4D`) |
| `topic` | string | Topic enum variant name (e.g. `PreflopDecision`, `BluffSpot`) |
| `branch_key` | string | Fine-grained branch within the topic (e.g. `OpenRaise:premium:IP`) |
| `difficulty` | string | `Beginner`, `Intermediate`, or `Advanced` |
| `answer_id` | string | The answer the user selected (`"A"`, `"B"`, or `"C"`) |
| `is_correct` | boolean | Whether the selected answer was correct |
| `timestamp` | ISO 8601 string | UTC datetime of submission |

These records are immutable — never update a submitted answer. Append only.

---

## Section 2 — Scoring Formula Per Topic

### Point Values by Difficulty

| Difficulty | Points (correct) | Points (wrong) |
|---|---|---|
| Beginner | 10 | 0 |
| Intermediate | 20 | 0 |
| Advanced | 30 | 0 |

Wrong answers always score 0. There are no negative points — this encourages continued participation without penalizing exploration.

### Per-Topic Score

```
topic_score = sum of points earned across all answer records for that topic
```

A user who answers 5 Beginner questions correctly and 3 Intermediate questions correctly on `BluffSpot` has a topic score of `(5 × 10) + (3 × 20) = 110`.

### Lifetime Score

```
lifetime_score = sum of topic_score across all 15 topics
```

---

## Section 3 — Adaptive Difficulty Rules

Difficulty adapts per `(user_id, branch_key)` pair — not per topic as a whole, because a user may be advanced at one branch of a topic and still a beginner at another.

### Starting Condition

- Any new `(user_id, branch_key)` pair begins at **Beginner**.

### Promotion Rule

- **3 correct answers in a row** on the current difficulty → promote one level.
  - Beginner → Intermediate
  - Intermediate → Advanced
  - Advanced → stays at Advanced (no further promotion)

### Demotion Rule

- **2 wrong answers in a row** on the current difficulty → demote one level.
  - Advanced → Intermediate
  - Intermediate → Beginner
  - Beginner → stays at Beginner (no further demotion)

### Streak Tracking

- The streak counter tracks only the **current direction** (consecutive correct or consecutive wrong).
- Any direction change (correct after wrong, or wrong after correct) resets the streak to 1 in the new direction.
- Example streak sequence for a branch: `C C W C C C` → after the 3rd consecutive correct the user promotes.

---

## Section 4 — Per-Topic Progress Rating (Mastery Level)

Each topic receives a **mastery level** from 0 to 100.

### Inputs

| Input | Description |
|---|---|
| `accuracy` | `correct_count / total_attempts` for the topic (0.0–1.0). Uses 0 if no attempts yet. |
| `difficulty_level` | Highest difficulty level currently reached on any branch of this topic: Beginner=0, Intermediate=1, Advanced=2 |

### Formula

```
mastery = (accuracy × 50) + (difficulty_level × 25)
```

### Range

| Scenario | Mastery |
|---|---|
| No attempts | 0 |
| 100% accuracy at Beginner | 50 |
| 100% accuracy at Intermediate | 75 |
| 100% accuracy at Advanced | 100 |
| 50% accuracy at Advanced | 75 |

The mastery score is a **display value** — it can be shown as a progress bar (0–100%) or a star rating.

---

## Section 5 — branch_key Granularity

The `branch_key` is finer-grained than the topic name. It identifies a specific decision sub-type within a topic. For example, within `PreflopDecision` there may be branches such as:

- `OpenRaise:premium:IP`
- `OpenRaise:marginal:OOP`
- `CallOrFold:strong:BB`

This allows the system to give targeted feedback. A user might have mastered `OpenRaise:premium:IP` but still struggle with `OpenRaise:marginal:OOP`, even though both belong to the `PreflopDecision` topic.

**Per-branch stats** support fine-grained feedback like: *"You are accurate on premium hands in position, but your marginal hand decisions need work."*

**Per-topic stats** aggregate across all branches and are used for the overall mastery level and lifetime score.

The `branch_key` format is defined by each topic module. It should be a short, stable, human-readable string with no spaces.

---

## Section 6 — Storage Shape

The following describes the JSON shape of the key records. No specific database is assumed.

### UserProfile

```json
{
  "user_id": "u_abc123",
  "lifetime_score": 340,
  "topic_scores": {
    "PreflopDecision": 110,
    "BluffSpot": 80,
    "PotOddsAndEquity": 150
  },
  "topic_mastery": {
    "PreflopDecision": 75,
    "BluffSpot": 50,
    "PotOddsAndEquity": 87
  },
  "created_at": "2026-01-15T10:00:00Z",
  "updated_at": "2026-02-23T14:32:00Z"
}
```

### BranchStats

One record per `(user_id, branch_key)` pair.

```json
{
  "user_id": "u_abc123",
  "branch_key": "OpenRaise:premium:IP",
  "topic": "PreflopDecision",
  "current_difficulty": "Intermediate",
  "correct_streak": 2,
  "wrong_streak": 0,
  "total_attempts": 12,
  "correct_count": 9,
  "points_earned": 130,
  "updated_at": "2026-02-23T14:32:00Z"
}
```

### AnswerRecord

One immutable record per submitted answer (append-only log).

```json
{
  "user_id": "u_abc123",
  "scenario_id": "PF-3A1B2C4D",
  "topic": "PreflopDecision",
  "branch_key": "OpenRaise:premium:IP",
  "difficulty": "Intermediate",
  "answer_id": "B",
  "is_correct": true,
  "timestamp": "2026-02-23T14:32:00Z"
}
```

---

## Summary

| Concept | Granularity | Key Formula |
|---|---|---|
| Points per answer | Per answer record | difficulty × (10/20/30) if correct, else 0 |
| Topic score | Per topic | Sum of points on that topic |
| Lifetime score | Global | Sum of all topic scores |
| Adaptive difficulty | Per branch_key | 3-correct streak → promote; 2-wrong streak → demote |
| Mastery level | Per topic | `(accuracy × 50) + (difficulty_level × 25)` |

---

## Section 7 — UX/UI Design

### 7.1 Progress Bar Conventions

All progress bars use a filled/empty block style and are 20 characters wide:

```
[████████████░░░░░░░░]  60%
```

Color tiers (CSS class or color token):

| Mastery Range | Color  | Label       |
|---------------|--------|-------------|
| 0 – 33        | red    | Beginner    |
| 34 – 66       | yellow | Intermediate|
| 67 – 99       | green  | Advanced    |
| 100           | gold   | Mastered    |

Difficulty badge colors:

| Difficulty   | Badge style              |
|--------------|--------------------------|
| Beginner     | grey background          |
| Intermediate | blue background          |
| Advanced     | purple background        |

---

### 7.2 Screen 1 — Player Stats Dashboard

The top-level view shown after a player logs in or taps "My Stats".

```
╔══════════════════════════════════════════════════════════════╗
║  PLAYER STATS                                    [≡ Menu]    ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   Jake M.                              ★ 1,840 pts lifetime  ║
║   Member since Jan 2026                                      ║
║                                                              ║
║   OVERALL MASTERY                                            ║
║   [████████████████░░░░]  78%   ·  12 / 15 topics started   ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  TOPIC PROGRESS                                [Sort ▾]      ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  Preflop Decision          [INTERMEDIATE]                    ║
║  [████████████████░░░░]  80%          110 pts   9/12 ✓      ║
║                                                              ║
║  Postflop C-Bet            [ADVANCED]                        ║
║  [████████████████████] 100%          210 pts  14/14 ✓      ║
║                                                              ║
║  Pot Odds & Equity         [INTERMEDIATE]                    ║
║  [██████████░░░░░░░░░░]  50%           80 pts   6/10 ✓      ║
║                                                              ║
║  Bluff Spot                [BEGINNER]                        ║
║  [████░░░░░░░░░░░░░░░░]  20%           30 pts   3/8  ✓      ║
║                                                              ║
║  ICM / Tournament          [BEGINNER]                        ║
║  [░░░░░░░░░░░░░░░░░░░░]   0%            0 pts   —  not started║
║                                                              ║
║  Turn Barrel               [INTERMEDIATE]                    ║
║  [██████████████░░░░░░]  70%          140 pts  10/12 ✓      ║
║                                                              ║
║  Check-Raise               [INTERMEDIATE]                    ║
║  [████████░░░░░░░░░░░░]  40%           60 pts   5/9  ✓      ║
║                                                              ║
║  Semi-Bluff                [BEGINNER]                        ║
║  [██████░░░░░░░░░░░░░░]  30%           40 pts   4/8  ✓      ║
║                                                              ║
║  Anti-Limper Iso           [ADVANCED]                        ║
║  [████████████████████]  95%          190 pts  13/13 ✓      ║
║                                                              ║
║  River Value Bet           [INTERMEDIATE]                    ║
║  [██████████████░░░░░░]  65%          120 pts   9/11 ✓      ║
║                                                              ║
║  Squeeze Play              [BEGINNER]                        ║
║  [██████░░░░░░░░░░░░░░]  25%           20 pts   2/6  ✓      ║
║                                                              ║
║  Big Blind Defense         [INTERMEDIATE]                    ║
║  [████████████░░░░░░░░]  60%          100 pts   8/10 ✓      ║
║                                                              ║
║  3-Bet Pot C-Bet           [BEGINNER]                        ║
║  [████░░░░░░░░░░░░░░░░]  15%           10 pts   1/5  ✓      ║
║                                                              ║
║  River Call or Fold        [INTERMEDIATE]                    ║
║  [██████████░░░░░░░░░░]  50%           80 pts   6/9  ✓      ║
║                                                              ║
║  Turn Probe Bet            [BEGINNER]                        ║
║  [░░░░░░░░░░░░░░░░░░░░]   0%            0 pts   —  not started║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║       [▶ Start Next Drill]     [⟳ Review Wrong Answers]      ║
╚══════════════════════════════════════════════════════════════╝
```

**Layout notes:**
- Each topic row = topic name + difficulty badge + progress bar + pts + accuracy fraction.
- Rows are sorted by mastery descending by default; user can toggle sort.
- "Not started" topics show an empty bar and greyed text.
- The overall mastery bar at the top is the average of all 15 topic mastery values.
- "Start Next Drill" routes to the lowest-mastery topic that is not yet mastered.

---

### 7.3 Screen 2 — Topic Detail View

Tapping any topic row opens the detail view for that topic.

```
╔══════════════════════════════════════════════════════════════╗
║  ← BACK          PREFLOP DECISION              [≡ Menu]     ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   MASTERY                                                    ║
║   [████████████████░░░░]  80%    110 pts total               ║
║   Accuracy: 75%  (9 correct / 12 attempts)                   ║
║   Current level: INTERMEDIATE                                ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  BRANCHES                                                    ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  OpenRaise : premium : IP          [ADVANCED]   streak ✓✓   ║
║  [████████████████████]  95%       5/5 correct               ║
║                                                              ║
║  OpenRaise : marginal : OOP        [INTERMEDIATE] streak ✗   ║
║  [██████████░░░░░░░░░░]  50%       2/4 correct               ║
║                                                              ║
║  CallOrFold : strong : BB          [BEGINNER]   streak ✓    ║
║  [████████░░░░░░░░░░░░]  40%       2/3 correct               ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  RECENT ACTIVITY                                             ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  ✓  PF-3A1B2C4D  OpenRaise:premium:IP    Intermediate  +20  ║
║  ✗  PF-9F2E1A3B  OpenRaise:marginal:OOP  Intermediate   +0  ║
║  ✓  PF-1C4D5E6F  OpenRaise:premium:IP    Intermediate  +20  ║
║  ✓  PF-7B8A2C1D  CallOrFold:strong:BB    Beginner      +10  ║
║  ✗  PF-2E3F4A5B  OpenRaise:marginal:OOP  Beginner       +0  ║
║                                                              ║
║                               [Load more...]                 ║
╠══════════════════════════════════════════════════════════════╣
║             [▶ Drill This Topic]                             ║
╚══════════════════════════════════════════════════════════════╝
```

**Layout notes:**
- Each branch row shows its own mini progress bar, current difficulty badge, and streak indicator.
- Streak indicator: `✓✓` = 2 correct in a row, `✗` = last answer was wrong.
- Recent activity log shows the last 5 answer records, newest first.
- `+20` / `+0` shows points earned on each attempt.
- "Drill This Topic" starts the next scenario for this topic at the user's current adaptive difficulty.

---

### 7.4 Screen 3 — Post-Answer Feedback Panel

Shown immediately after the user submits an answer to a drill. Overlays or replaces the drill card.

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║         ✓  CORRECT  +20 pts                                  ║
║                                                              ║
║  "Raise to 3BB from BTN with AKs — premium hand in          ║
║   position. Folding is too weak; calling is suboptimal."     ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  PREFLOP DECISION  —  OpenRaise:premium:IP                   ║
║                                                              ║
║  Mastery    [████████████████░░░░]  80%   (was 75%)  ▲ +5   ║
║  Streak     ✓ ✓  (1 more correct → ADVANCED)                 ║
║  Difficulty [INTERMEDIATE]                                   ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║   [▶ Next Drill]             [✕ Back to Stats]               ║
╚══════════════════════════════════════════════════════════════╝
```

Wrong answer variant:

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║         ✗  INCORRECT  +0 pts                                 ║
║                                                              ║
║  Correct answer: B — Raise to 3BB                            ║
║  "KJo from UTG is marginal out of position. Folding is       ║
║   correct at this stack depth."                              ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  PREFLOP DECISION  —  OpenRaise:marginal:OOP                 ║
║                                                              ║
║  Mastery    [██████████░░░░░░░░░░]  50%   (unchanged)        ║
║  Streak     ✗ ✗  (1 more wrong → demoted to BEGINNER)        ║
║  Difficulty [INTERMEDIATE]                                   ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║   [▶ Try Again (same branch)]   [✕ Back to Stats]            ║
╚══════════════════════════════════════════════════════════════╝
```

**Layout notes:**
- Mastery bar shows the delta `▲ +5` on correct, no delta on wrong (mastery only increases on correct).
- Streak displays as filled dots or ✓/✗ icons, with a hint about the next threshold.
- On wrong answer: reveal the correct answer ID and show the full explanation.
- "Try Again" re-drills the same branch at the same difficulty.

---

### 7.5 UI Component Summary

| Component | Used on | Data source |
|---|---|---|
| Overall mastery bar | Dashboard header | avg of all 15 `topic_mastery` values |
| Per-topic mastery bar | Dashboard row, Topic detail | `topic_mastery[topic]` |
| Per-branch mastery bar | Topic detail — Branches | derived from `BranchStats` accuracy + difficulty |
| Difficulty badge | Dashboard row, Topic detail, Feedback panel | `BranchStats.current_difficulty` |
| Streak indicator | Topic detail, Feedback panel | `BranchStats.correct_streak` / `wrong_streak` |
| Lifetime score | Dashboard header | `UserProfile.lifetime_score` |
| Points delta | Feedback panel | points earned on last answer |
| Recent activity log | Topic detail | last N `AnswerRecord` rows for this topic |
