use serde_json::json;

#[derive(Clone)]
pub struct DecideurClient {
    pub base_url: String,
    pub token: String,
    client: reqwest::Client,
}

impl DecideurClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn handshake(&self, simulation_id: &str, scenario: &str, target: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/simulation/handshake", self.base_url);
        let body = json!({
            "simulation_id": simulation_id,
            "scenario": scenario,
            "target": target,
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
