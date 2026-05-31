use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub agent_token: String,
    #[serde(default)]
    pub decideur_token: Option<String>,
    pub auditeur_url: String,
    pub decideur_url: String,
    pub analyseur_url: String,
    pub chiffreur_url: String,
    #[serde(default)]
    pub trusted_ca: Option<String>,
    #[serde(default)]
    pub simulateur_whitelist: Vec<String>,
    pub sandbox_cidr: String,
    #[serde(default)]
    pub production_cidrs: Vec<String>,
    #[serde(default = "default_dry_run")]
    pub dry_run: bool,
    #[serde(default = "default_detection_timeout")]
    pub detection_timeout_secs: u64,
    pub phase1_scenario: String,
    pub phase2_scenario: String,
    pub phase3_scenario: String,
    #[serde(default = "default_min_phases_ok")]
    pub min_phases_ok_for_success: u8,
    pub backup_events_path: String,
    #[serde(default = "default_port")]
    pub listen_port: u16,
    #[serde(default = "default_true")]
    pub require_authorization: bool,
    #[serde(default)]
    pub require_authorization_ref: bool,
    #[serde(default)]
    pub require_snapshot: bool,
    #[serde(default)]
    pub require_rollback_plan: bool,
    #[serde(default)]
    pub enforce_launch_window: bool,
    #[serde(default)]
    pub enforce_health_checks: bool,
    #[serde(default = "default_launch_start")]
    pub launch_hour_start: u32,
    #[serde(default = "default_launch_end")]
    pub launch_hour_end: u32,
    #[serde(default = "default_global_max")]
    pub global_max_duration_secs: u64,
}

fn default_dry_run() -> bool {
    true
}

fn default_detection_timeout() -> u64 {
    60
}

fn default_min_phases_ok() -> u8 {
    2
}

fn default_port() -> u16 {
    8005
}

fn default_true() -> bool {
    true
}

fn default_launch_start() -> u32 {
    8
}

fn default_launch_end() -> u32 {
    22
}

fn default_global_max() -> u64 {
    900
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = env::var("SIMULATEUR_CONFIG").unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("config/config.json")
                .to_string_lossy()
                .into_owned()
        });

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Impossible de lire {path}: {e}"))?;
        let mut cfg: Config =
            serde_json::from_str(&raw).map_err(|e| format!("config.json invalide: {e}"))?;

        if let Ok(v) = env::var("SIMULATEUR_DRY_RUN") {
            cfg.dry_run = v == "1" || v.eq_ignore_ascii_case("true");
        }

        Ok(cfg)
    }

    pub fn decideur_token(&self) -> &str {
        self.decideur_token.as_deref().unwrap_or(&self.agent_token)
    }
}
