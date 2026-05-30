use reqwest::Client;

pub struct AnalyseurClient {
    client: Client,
    url: String,
    token: String,
}

impl AnalyseurClient {
    pub fn new(url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
        }
    }

    pub async fn sim_start(
        &self,
        perimetre_ip: &str,
        duree_estimee: u32,
    ) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/simulation/mode", self.url);
        let body = serde_json::json!({
            "event": "SIM_START",
            "perimetre_ip": perimetre_ip,
            "duree_estimee": duree_estimee
        });

        self.client
            .post(&endpoint)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn sim_end(&self) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/simulation/mode", self.url);
        let body = serde_json::json!({
            "event": "SIM_END"
        });

        self.client
            .post(&endpoint)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await?;

        Ok(())
    }
}
