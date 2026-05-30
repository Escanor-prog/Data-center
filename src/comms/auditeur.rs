use reqwest::Client;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;

pub struct AuditeurClient {
    client: Client,
    url: String,
    token: String,
}

impl AuditeurClient {
    pub fn new(url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
        }
    }

    pub async fn envoyer_event(&self, body: Value) {
        let endpoint = format!("{}/events", self.url);
        let res = self
            .client
            .post(&endpoint)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await;

        if res.is_err() || !res.unwrap().status().is_success() {
            // Sauvegarde locale en cas d'erreur de communication
            if let Ok(json_string) = serde_json::to_string(&body) {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/var/log/simulateur/backup.jsonl");

                if let Ok(mut f) = file {
                    let _ = writeln!(f, "{}", json_string);
                }
            }
        }
    }

    pub async fn simulation_start(&self, sim_id: &str, scenario: &str) {
        let body = serde_json::json!({
            "event": "SIMULATION_START",
            "simulation_id": sim_id,
            "scenario": scenario,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.envoyer_event(body).await;
    }

    pub async fn simulation_end(&self, sim_id: &str) {
        let body = serde_json::json!({
            "event": "SIMULATION_END",
            "simulation_id": sim_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.envoyer_event(body).await;
    }

    pub async fn simulation_error(&self, sim_id: &str, error: &str) {
        let body = serde_json::json!({
            "event": "SIMULATION_ERROR",
            "simulation_id": sim_id,
            "error": error,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.envoyer_event(body).await;
    }

    pub async fn simulation_stopped(&self, sim_id: &str) {
        let body = serde_json::json!({
            "event": "SIMULATION_STOPPED",
            "simulation_id": sim_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.envoyer_event(body).await;
    }
}
