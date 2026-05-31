use crate::api::auth::{check_auditeur, check_decideur};
use crate::api::dto::{
    HealthResponse, MetricsResponse, PhaseAckRequest, ScenarioItem, ScenarioListResponse,
    SimulationLaunchRequest, SimulationStatusResponse, SimulationStopOrder,
};
use crate::comms::dto;
use crate::config::Config;
use crate::control::fsm::FsmState;
use crate::control::metrics::SimulationMetrics;
use crate::control::orchestrator::{self, StartParams};
use crate::control::state::SharedSimulationState;
use crate::scenarios::catalogue;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use std::sync::Arc;
use std::time::Instant;

use crate::comms::AgentClients;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub clients: Arc<AgentClients>,
    pub control_state: SharedSimulationState,
    pub metrics: Arc<SimulationMetrics>,
    pub started_at: Instant,
}

fn phase_label(p: crate::scenarios::types::PhaseKind) -> String {
    match p {
        crate::scenarios::types::PhaseKind::Filtreur => "filtreur".into(),
        crate::scenarios::types::PhaseKind::Analyseur => "analyseur".into(),
        crate::scenarios::types::PhaseKind::Chiffreur => "chiffreur".into(),
        crate::scenarios::types::PhaseKind::Auditeur => "auditeur".into(),
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/simulation/list", get(simulation_list))
        .route("/simulation/status", get(simulation_status))
        .route("/simulation/start", post(start_simulation))
        .route("/simulation/stop", post(stop_simulation))
        .route("/simulation/phase_ack", post(phase_ack))
        .with_state(state)
}

fn uptime_sec(started: Instant) -> u64 {
    started.elapsed().as_secs()
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            request_id: dto::new_request_id(),
            message_type: "response",
            status: "ok",
            uptime_sec: uptime_sec(state.started_at),
            version: env!("CARGO_PKG_VERSION"),
        }),
    )
}

async fn metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    check_auditeur(&headers, &state.config.agent_token)?;
    Ok((
        StatusCode::OK,
        Json(MetricsResponse {
            request_id: dto::new_request_id(),
            message_type: "response",
            metrics: state.metrics.snapshot(),
        }),
    ))
}

async fn simulation_list(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let scenarios = catalogue::known_scenarios()
        .iter()
        .map(|s| ScenarioItem {
            id: s.id.to_string(),
            description: s.description.to_string(),
            agent_cible: s.agent_cible.to_string(),
            phase: phase_label(s.phase),
            anomaly_ref: s.anomaly_ref.to_string(),
            mitre_tactic: s.mitre_tactic.to_string(),
            max_duration_secs: s.max_duration_secs,
            detection_timeout_secs: s.detection_timeout_secs,
        })
        .collect();

    (
        StatusCode::OK,
        Json(ScenarioListResponse {
            request_id: dto::new_request_id(),
            message_type: "response",
            scenarios,
        }),
    )
}

async fn simulation_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    check_auditeur(&headers, &state.config.agent_token)?;
    let s = state.control_state.read().unwrap();
    let started = s.started_at.map(|t| t.to_rfc3339());

    Ok((
        StatusCode::OK,
        Json(SimulationStatusResponse {
            request_id: dto::new_request_id(),
            message_type: "response",
            active: s.active,
            fsm_state: format!("{:?}", s.fsm).to_lowercase(),
            simulation_id: s.simulation_id.clone(),
            scenario: s.scenario.clone(),
            started_at: started,
            phase_results: s.phase_results.clone(),
        }),
    ))
}

async fn start_simulation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SimulationLaunchRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    check_auditeur(&headers, &state.config.agent_token)?;

    {
        let s = state.control_state.read().unwrap();
        if s.active {
            return Err(StatusCode::CONFLICT);
        }
    }

    let params = StartParams {
        scenario: body.scenario.clone(),
        target: body.target.value.clone(),
        authorized_by: body.authorized_by.clone(),
        authorization_ref: body.authorization_ref.clone(),
        snapshot_id: body.snapshot_id.clone(),
        rollback_plan_id: body.rollback_plan_id.clone(),
        max_duration_seconds: body.max_duration_seconds,
        token_valid: true,
    };

    let cfg = state.config.clone();
    let clients = state.clients.clone();
    let control = state.control_state.clone();
    let metrics = state.metrics.clone();

    tokio::spawn(async move {
        orchestrator::run_full_pipeline(cfg, clients, control, metrics, params).await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "request_id": dto::new_request_id(),
            "message_type": "response",
            "status": "accepted",
            "mode": "full_pipeline"
        })),
    ))
}

async fn stop_simulation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SimulationStopOrder>,
) -> Result<impl IntoResponse, StatusCode> {
    check_decideur(&headers, &state.config.decideur_token())?;

    let sim_id;
    {
        let mut s = state.control_state.write().unwrap();
        if !s.active {
            return Err(StatusCode::NOT_FOUND);
        }
        if s.simulation_token.as_deref() != Some(body.simulation_token.as_str()) {
            return Err(StatusCode::FORBIDDEN);
        }
        sim_id = s.simulation_id.clone().unwrap_or_default();
        orchestrator::request_stop(&state.control_state);
        s.set_fsm(FsmState::Stopping);
    }

    state.metrics.inc_stopped();
    state
        .clients
        .auditeur
        .simulation_stopped(&sim_id, body.reason.as_deref().unwrap_or("Décideur"))
        .await;

    let mut s = state.control_state.write().unwrap();
    s.stop();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id": dto::new_request_id(),
            "message_type": "response",
            "status": "stopped",
            "simulation_id": sim_id
        })),
    ))
}

/// Endpoint additionnel : le Décideur confirme qu'une phase a détecté/bloqué le flux.
async fn phase_ack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PhaseAckRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    check_decideur(&headers, &state.config.decideur_token())?;

    {
        let s = state.control_state.read().unwrap();
        if s.simulation_id.as_deref() != Some(body.simulation_id.as_str()) {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    orchestrator::signal_phase_ack(&state.control_state);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id": dto::new_request_id(),
            "message_type": "response",
            "status": "phase_acknowledged",
            "phase": body.phase
        })),
    ))
}
