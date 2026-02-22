use axum::{routing::{get, post}, Router};
use super::handler::{get_scenario, submit_answer, DrillCache};

pub fn router(cache: DrillCache) -> Router {
    Router::new()
        .route("/api/drill/scenario", get(get_scenario))
        .route("/api/drill/answer",   post(submit_answer))
        .with_state(cache)
}
