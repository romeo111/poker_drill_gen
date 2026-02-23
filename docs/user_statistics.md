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
