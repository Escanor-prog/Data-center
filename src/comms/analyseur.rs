use serde_json::json;

#[derive(Clone)]
pub struct AnalyseurClient {
    pub base_url: String,
    pub token: String,
    client: reqwest::Client,
}

impl AnalyseurClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn sim_start(&self, simulation_id: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/simulation/mode", self.base_url);
        let body = json!({
            "event": "SIM_START",
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

    pub async fn sim_end(&self, simulation_id: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/simulation/mode", self.base_url);
        let body = json!({
            "event": "SIM_END",
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
