use reqwest::Client;

pub struct DecideurClient {
    client: Client,
    url: String,
    token: String,
}

impl DecideurClient {
    pub fn new(url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
        }
    }

    pub async fn handshake(
        &self,
        simulation_id: &str,
        type_attaque: &str,
        cible: &str,
    ) -> Result<(), reqwest::Error> {
        let endpoint = format!("{}/simulation/handshake", self.url);
        let body = serde_json::json!({
            "simulation_id": simulation_id,
            "type_attaque": type_attaque,
            "cible": cible
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
