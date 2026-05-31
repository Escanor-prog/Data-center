use crate::comms::dto;
use reqwest::Client;
use serde_json::json;

pub struct AnalyseurClient {
    client: Client,
    url: String,
    token: String,
    dry_run: bool,
}

impl AnalyseurClient {
    pub fn new(url: String, token: String, dry_run: bool) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
            dry_run,
        }
    }

    pub async fn sim_start(
        &self,
        simulation_id: &str,
        perimetre_ip: &str,
        duree_estimee: u64,
    ) -> Result<(), String> {
        let body = dto::envelope(
            "simulation_mode",
            json!({
                "event": "SIM_START",
                "simulation_id": simulation_id,
                "perimetre_ip": perimetre_ip,
                "duree_estimee": duree_estimee
            }),
        );

        self.post_mode(body).await
    }

    pub async fn sim_end(&self, simulation_id: &str) -> Result<(), String> {
        let body = dto::envelope(
            "simulation_mode",
            json!({
                "event": "SIM_END",
                "simulation_id": simulation_id
            }),
        );

        self.post_mode(body).await
    }

    async fn post_mode(&self, body: serde_json::Value) -> Result<(), String> {
        let endpoint = format!("{}/simulation/mode", self.url);
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
                tracing::warn!(status = %r.status(), "Analyseur mode — ignoré en dry_run");
                Ok(())
            }
            Ok(r) => Err(format!("Analyseur mode refusé: HTTP {}", r.status())),
            Err(e) if self.dry_run => {
                tracing::warn!(error = %e, "Analyseur injoignable — ignoré en dry_run");
                Ok(())
            }
            Err(e) => Err(format!("Erreur Analyseur: {e}")),
        }
    }
}
