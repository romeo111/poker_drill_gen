# Drill API — Server Integration Guide

Ready-to-copy Axum handlers that expose two HTTP endpoints for the poker drill training mode.

---

## Files

```
integration/
  src/
    drill/
      mod.rs       — module declaration
      handler.rs   — GET /api/drill/scenario, POST /api/drill/answer, in-memory cache
      routes.rs    — builds the Axum Router
  INTEGRATION.md   — this file
```

Copy the `src/drill/` directory into your server's `src/` folder.

---

## Step 1 — Add the dependency

In your server's `Cargo.toml`:

```toml
[dependencies]
poker_drill_gen = { path = "<relative-path-to-poker_drill_gen>" }
# serde and serde_json are already required; ensure they are present
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
```

For Docker builds, place `poker_drill_gen/` inside the build context (e.g. repo root)
and use `path = "../poker_drill_gen"` from the server crate.

---

## Step 2 — Declare the module

In `src/main.rs` (or wherever your top-level `mod` declarations live):

```rust
mod drill;
```

---

## Step 3 — Register the routes

The drill router carries its own `Arc<Mutex<HashMap>>` state, so it does **not**
conflict with your existing socket-based state.

**Option A — standalone `Router::new()` then `.merge()`:**

```rust
use crate::drill;

let drill_cache = drill::handler::new_cache();

let app = Router::new()
    // ... your existing routes ...
    .merge(drill::routes::router(drill_cache));
```

**Option B — if you build your app in a separate function (e.g. `socket_server.rs`):**

```rust
// At the top of the function:
let drill_cache = drill::handler::new_cache();

// After your existing Router::new() chain, before .layer():
let app = Router::new()
    .route("/", get(|| async { "ok" }))
    // ...
    .with_state(io.clone())
    .layer(your_layers)
    .merge(drill::routes::router(drill_cache));  // ← add this line
```

> **Important:** `.merge()` must come **after** `.layer()`. Axum merges the inner
> router's own layers separately, so CORS and other middleware on the outer router
> will still apply to the merged routes.

---

## Endpoints

### `GET /api/drill/scenario`

**Query parameters:**

| Param        | Values                                                                                          |
|--------------|-------------------------------------------------------------------------------------------------|
| `topic`      | `PreflopDecision` · `PostflopContinuationBet` · `PotOddsAndEquity` · `BluffSpot` · `ICMAndTournamentDecision` · `TurnBarrelDecision` · `CheckRaiseSpot` · `SemiBluffDecision` · `AntiLimperIsolation` |
| `difficulty` | `Beginner` · `Intermediate` · `Advanced`                                                        |

**Response (200):**

```jsonc
{
  "table_state": { /* NtTableState — feed directly to the Angular TableComponent */ },
  "drill": {
    "scenario_id": "CR-3F2A1B0E",
    "topic":       "Check-Raise Spot",
    "branch_key":  "CheckRaise:Draw:OOP",
    "question":    "You flopped a flush draw out of position...",
    "answers": [
      { "id": "A", "text": "Check-raise to 3x" },
      { "id": "B", "text": "Check-call" },
      { "id": "C", "text": "Fold" }
    ]
  }
}
```

`is_correct` and `explanation` are **withheld** from this response.

---

### `POST /api/drill/answer`

**Body:**

```json
{ "scenario_id": "CR-3F2A1B0E", "answer_id": "A" }
```

**Response (200):**

```jsonc
{
  "is_correct":  true,
  "explanation": "Check-raising your flush draw as a semi-bluff...",
  "correct_id":  "A"
}
```

**Response (404):** scenario not found (expired or never generated in this server session).

**Response (400):** unknown `topic`, `difficulty`, or `answer_id`.

---

## How the cache works

- Scenarios are stored in an `Arc<Mutex<HashMap<String, TrainingScenario>>>` keyed by `scenario_id`.
- The cache is created once at startup (`new_cache()`) and shared across requests.
- It is **in-process only** — not persisted across server restarts.
- A simple cap of 1000 entries evicts the oldest when the limit is hit.
- If you need persistence or cross-instance sharing, replace the HashMap with a Redis
  call using `serde_json::to_string` / `from_str` on `TrainingScenario`
  (serde derives are already on all types).

---

## NtTableState mapping details

`to_nt_table_state(scenario, hero_player_id)` in `poker_drill_gen::nt_adapter`:

| Slot            | Value                                      |
|-----------------|--------------------------------------------|
| seat_idx 1      | Hero — real cards shown, `is_active: true` |
| seat_idx 2      | Villain — cards hidden (`"b"`)             |
| seat_idx 0, 3-5 | Empty seats (`player_id: 0`)               |
| `game_state`    | `"PreFlop"` / `"Flop"` / `"Turn"` / `"River"` from board length |
| Card format     | rank 10 → `"10s"`, others → `"Ts"` style  |
| `action_option` | `actions: []` — drill overlay handles answers, not the table bar |

---

## CORS note

The drill routes are merged into your existing `Router` **after** your CORS layer, so
they inherit whatever origins you already allow. No additional CORS config needed.

---

## Verification

```bash
# Start your server, then:

curl "http://localhost:9001/api/drill/scenario?topic=CheckRaiseSpot&difficulty=Intermediate" | jq .

# Copy the scenario_id from the response, then:

curl -X POST http://localhost:9001/api/drill/answer \
  -H "Content-Type: application/json" \
  -d '{"scenario_id":"CR-XXXXXXXX","answer_id":"A"}' | jq .
```
