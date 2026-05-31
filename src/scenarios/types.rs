use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    Filtreur,
    Analyseur,
    Chiffreur,
    Auditeur,
}

impl PhaseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PhaseKind::Filtreur => "filtreur",
            PhaseKind::Analyseur => "analyseur",
            PhaseKind::Chiffreur => "chiffreur",
            PhaseKind::Auditeur => "auditeur",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "filtreur" => Some(PhaseKind::Filtreur),
            "analyseur" => Some(PhaseKind::Analyseur),
            "chiffreur" => Some(PhaseKind::Chiffreur),
            "auditeur" => Some(PhaseKind::Auditeur),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteOutcome {
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseResultStatus {
    Ok,
    Timeout,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInfo {
    pub id: String,
    pub description: String,
    pub agent_cible: String,
    pub phase: PhaseKind,
    pub anomaly_ref: String,
    pub mitre_tactic: String,
    pub tool: String,
    pub max_duration_secs: u64,
    pub detection_timeout_secs: u64,
    #[serde(default)]
    pub pipeline_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct AttacksCatalog {
    pub version: String,
    pub scenarios: Vec<ScenarioInfo>,
    pub timeouts_policy: TimeoutsPolicy,
}

#[derive(Debug, Deserialize)]
pub struct TimeoutsPolicy {
    pub detection_default_secs: u64,
    pub detection_phase1_secs: u64,
    pub detection_phase2_secs: u64,
    pub detection_phase3_secs: u64,
    pub global_pipeline_max_secs: u64,
}
