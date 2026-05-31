use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SimulationLaunchRequest {
    pub request_id: Option<String>,
    pub message_type: Option<String>,
    pub scenario: String,
    pub target: TargetRef,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub max_duration_seconds: Option<u64>,
    #[serde(default)]
    pub authorized_by: Option<String>,
    #[serde(default)]
    pub authorization_ref: Option<String>,
    #[serde(default)]
    pub snapshot_id: Option<String>,
    #[serde(default)]
    pub rollback_plan_id: Option<String>,
    #[serde(default)]
    pub intensity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TargetRef {
    #[serde(rename = "type")]
    pub target_type: Option<String>,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct SimulationStopOrder {
    pub request_id: Option<String>,
    pub message_type: Option<String>,
    pub simulation_id: String,
    pub simulation_token: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PhaseAckRequest {
    pub request_id: Option<String>,
    pub message_type: Option<String>,
    pub simulation_id: String,
    pub phase: u8,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub detected: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub request_id: String,
    pub message_type: &'static str,
    pub status: &'static str,
    pub uptime_sec: u64,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ScenarioListResponse {
    pub request_id: String,
    pub message_type: &'static str,
    pub scenarios: Vec<ScenarioItem>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioItem {
    pub id: String,
    pub description: String,
    pub agent_cible: String,
    pub phase: String,
    pub anomaly_ref: String,
    pub mitre_tactic: String,
    pub max_duration_secs: u64,
    pub detection_timeout_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct SimulationStatusResponse {
    pub request_id: String,
    pub message_type: &'static str,
    pub active: bool,
    pub fsm_state: String,
    pub simulation_id: Option<String>,
    pub scenario: Option<String>,
    pub started_at: Option<String>,
    pub phase_results: Vec<crate::control::state::PhaseResult>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub request_id: String,
    pub message_type: &'static str,
    #[serde(flatten)]
    pub metrics: crate::control::metrics::MetricsSnapshot,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub request_id: String,
    pub message_type: &'static str,
    pub error: String,
    pub description: String,
}
