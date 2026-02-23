use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use poker_drill_gen::{
    generate_training, to_nt_table_state, DifficultyLevel, TrainingRequest, TrainingScenario,
    TrainingTopic,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Shared state: in-memory scenario cache keyed by scenario_id
// ---------------------------------------------------------------------------

pub type DrillCache = Arc<Mutex<HashMap<String, TrainingScenario>>>;

pub fn new_cache() -> DrillCache {
    Arc::new(Mutex::new(HashMap::new()))
}

// ---------------------------------------------------------------------------
// Query / body types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DrillQuery {
    pub topic: String,
    pub difficulty: String,
}

#[derive(Deserialize)]
pub struct AnswerRequest {
    pub scenario_id: String,
    pub answer_id: String,
}

#[derive(Serialize)]
pub struct AnswerResponse {
    pub is_correct: bool,
    pub explanation: String,
    pub correct_id: String,
}

// ---------------------------------------------------------------------------
// Topic / difficulty parsing
// ---------------------------------------------------------------------------

fn parse_topic(s: &str) -> Option<TrainingTopic> {
    match s {
        "PreflopDecision"          => Some(TrainingTopic::PreflopDecision),
        "PostflopContinuationBet"  => Some(TrainingTopic::PostflopContinuationBet),
        "PotOddsAndEquity"         => Some(TrainingTopic::PotOddsAndEquity),
        "BluffSpot"                => Some(TrainingTopic::BluffSpot),
        "ICMAndTournamentDecision" => Some(TrainingTopic::ICMAndTournamentDecision),
        "TurnBarrelDecision"       => Some(TrainingTopic::TurnBarrelDecision),
        "CheckRaiseSpot"           => Some(TrainingTopic::CheckRaiseSpot),
        "SemiBluffDecision"        => Some(TrainingTopic::SemiBluffDecision),
        "AntiLimperIsolation"      => Some(TrainingTopic::AntiLimperIsolation),
        "RiverValueBet"            => Some(TrainingTopic::RiverValueBet),
        "SqueezePlay"              => Some(TrainingTopic::SqueezePlay),
        "BigBlindDefense"          => Some(TrainingTopic::BigBlindDefense),
        "ThreeBetPotCbet"          => Some(TrainingTopic::ThreeBetPotCbet),
        "RiverCallOrFold"          => Some(TrainingTopic::RiverCallOrFold),
        "TurnProbeBet"             => Some(TrainingTopic::TurnProbeBet),
        "DelayedCbet"              => Some(TrainingTopic::DelayedCbet),
        _ => None,
    }
}

fn parse_difficulty(s: &str) -> Option<DifficultyLevel> {
    match s {
        "Beginner"     => Some(DifficultyLevel::Beginner),
        "Intermediate" => Some(DifficultyLevel::Intermediate),
        "Advanced"     => Some(DifficultyLevel::Advanced),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// GET /api/drill/scenario?topic=...&difficulty=...
// ---------------------------------------------------------------------------

pub async fn get_scenario(
    State(cache): State<DrillCache>,
    Query(params): Query<DrillQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let topic = parse_topic(&params.topic).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Unknown topic: {}", params.topic) })),
        )
    })?;

    let difficulty = parse_difficulty(&params.difficulty).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Unknown difficulty: {}", params.difficulty) })),
        )
    })?;

    let scenario = generate_training(TrainingRequest {
        topic,
        difficulty,
        rng_seed: None,
    });

    let table_state = to_nt_table_state(&scenario, 1);

    // Strip is_correct / explanation from answers sent to the client.
    let public_answers: Vec<Value> = scenario
        .answers
        .iter()
        .map(|a| json!({ "id": a.id, "text": a.text }))
        .collect();

    let response = json!({
        "table_state": table_state,
        "drill": {
            "scenario_id":  scenario.scenario_id,
            "topic":        scenario.topic.to_string(),
            "branch_key":   scenario.branch_key,
            "question":     scenario.question,
            "answers":      public_answers,
        }
    });

    // Cache the full scenario for the answer endpoint.
    {
        let mut map = cache.lock().unwrap();
        // Evict oldest entries if cache grows too large (simple cap at 1000).
        if map.len() >= 1000 {
            if let Some(first_key) = map.keys().next().cloned() {
                map.remove(&first_key);
            }
        }
        map.insert(scenario.scenario_id.clone(), scenario);
    }

    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// POST /api/drill/answer   body: { scenario_id, answer_id }
// ---------------------------------------------------------------------------

pub async fn submit_answer(
    State(cache): State<DrillCache>,
    Json(body): Json<AnswerRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let map = cache.lock().unwrap();

    let scenario = map.get(&body.scenario_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Scenario not found or expired" })),
        )
    })?;

    let chosen = scenario
        .answers
        .iter()
        .find(|a| a.id == body.answer_id)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Unknown answer_id: {}", body.answer_id) })),
            )
        })?;

    let correct_id = scenario
        .answers
        .iter()
        .find(|a| a.is_correct)
        .map(|a| a.id.clone())
        .unwrap_or_default();

    Ok(Json(json!({
        "is_correct":  chosen.is_correct,
        "explanation": chosen.explanation,
        "correct_id":  correct_id,
    })))
}
