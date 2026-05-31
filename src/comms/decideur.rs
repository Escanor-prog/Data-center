use crate::comms::dto;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde_json::json;

pub struct DecideurClient {
    client: Client,
    url: String,
    token: String,
    dry_run: bool,
}

impl DecideurClient {
    pub fn new(url: String, token: String, dry_run: bool) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
            dry_run,
        }
    }

    pub async fn handshake(
        &self,
        simulation_id: &str,
        simulation_token: &str,
        attack_type: &str,
        target_value: &str,
        max_duration_secs: u64,
    ) -> Result<(), String> {
        let now = Utc::now();
        let end = now + Duration::seconds(max_duration_secs as i64);

        let body = dto::envelope(
            "simulation_handshake",
            json!({
                "simulation_id": simulation_id,
                "simulation_token": simulation_token,
                "attack_type": attack_type,
                "target": { "type": "vm", "value": target_value },
                "expected_start": now.to_rfc3339(),
                "expected_end": end.to_rfc3339()
            }),
        );

        let endpoint = format!("{}/simulation/handshake", self.url);
        let res = self
            .client
            .post(&endpoint)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => Ok(()),
            Ok(r) if self.dry_run => {
                tracing::warn!(
                    status = %r.status(),
                    "Handshake Décideur non OK — ignoré en dry_run"
                );
                Ok(())
            }
            Ok(r) => Err(format!("Handshake refusé: HTTP {}", r.status())),
            Err(e) if self.dry_run => {
                tracing::warn!(error = %e, "Décideur injoignable — ignoré en dry_run");
                Ok(())
            }
            Err(e) => Err(format!("Erreur handshake Décideur: {e}")),
        }
    }
}
