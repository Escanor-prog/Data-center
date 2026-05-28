use serde_json::json;

#[derive(Clone)]
pub struct AuditeurClient {
    pub base_url: String,
    pub token: String,
    client: reqwest::Client,
}

impl AuditeurClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn simulation_start(&self, simulation_id: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/events", self.base_url);
        let body = json!({
            "event": "simulation_start",
            "simulation_id": simulation_id,
        });

        self.client
            .post(&url)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
}
