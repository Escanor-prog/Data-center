use crate::comms::dto;
use reqwest::Client;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub struct AuditeurClient {
    client: Client,
    url: String,
    token: String,
    backup_path: String,
    dry_run: bool,
}

impl AuditeurClient {
    pub fn new(url: String, token: String, backup_path: String, dry_run: bool) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
            backup_path,
            dry_run,
        }
    }

    pub async fn send_event(&self, simulation_id: &str, event_type: &str, details: Value) {
        let body = dto::log_event_body(simulation_id, event_type, details);
        self.post_event(body).await;
    }

    async fn post_event(&self, body: Value) {
        let endpoint = format!("{}/events", self.url);
        let res = self
            .client
            .post(&endpoint)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await;

        let ok = matches!(res, Ok(ref r) if r.status().is_success());
        if !ok {
            self.backup(&body);
            if self.dry_run {
                tracing::debug!("Event Auditeur sauvegardé localement (dry_run)");
            }
        }
    }

    fn backup(&self, body: &Value) {
        if let Ok(json_string) = serde_json::to_string(body) {
            if let Some(parent) = Path::new(&self.backup_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.backup_path)
            {
                let _ = writeln!(f, "{json_string}");
            }
        }
    }

    pub async fn simulation_start(&self, sim_id: &str, scenario: &str) {
        self.send_event(
            sim_id,
            "SIMULATION_START",
            serde_json::json!({ "scenario": scenario }),
        )
        .await;
    }

    pub async fn simulation_end(&self, sim_id: &str, report: Value) {
        self.send_event(sim_id, "SIMULATION_END", report).await;
    }

    pub async fn simulation_error(&self, sim_id: &str, error: &str) {
        self.send_event(
            sim_id,
            "SIMULATION_ERROR",
            serde_json::json!({ "error": error }),
        )
        .await;
    }

    pub async fn simulation_stopped(&self, sim_id: &str, reason: &str) {
        self.send_event(
            sim_id,
            "SIMULATION_STOPPED",
            serde_json::json!({ "reason": reason }),
        )
        .await;
    }

    pub async fn phase_event(
        &self,
        sim_id: &str,
        phase: u8,
        status: &str,
        extra: Value,
    ) {
        let event_type = format!("PHASE{phase}_{status}");
        self.send_event(sim_id, &event_type, extra).await;
    }
}
