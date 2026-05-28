use crate::control::metrics::SimulationMetrics;
use crate::control::state::SharedSimulationState;
use crate::scenarios::catalogue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::Router;
use axum::routing::get;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub control_state: SharedSimulationState,
    pub metrics: Arc<SimulationMetrics>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ScenarioList {
    scenarios: Vec<&'static str>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/simulation/list", get(simulation_list))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

async fn simulation_list() -> impl IntoResponse {
    let list = catalogue::known_scenarios()
        .iter()
        .map(|scenario| scenario.id)
        .collect();

    (StatusCode::OK, Json(ScenarioList { scenarios: list }))
}
