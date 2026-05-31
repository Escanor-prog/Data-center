use crate::comms::dto;
use reqwest::Client;
use serde_json::json;

pub struct ChiffreurClient {
    client: Client,
    url: String,
    token: String,
    dry_run: bool,
}

#[derive(Debug)]
pub enum CryptoTestResult {
    Ok,
    RejectedAsExpected,
    Failed(String),
}

impl ChiffreurClient {
    pub fn new(url: String, token: String, dry_run: bool) -> Self {
        Self {
            client: Client::new(),
            url,
            token,
            dry_run,
        }
    }

    pub async fn encrypt_decrypt_roundtrip(&self, plaintext: &str) -> CryptoTestResult {
        if self.dry_run {
            tracing::info!("dry_run: encrypt/decrypt roundtrip simulé OK");
            return CryptoTestResult::Ok;
        }

        let enc_body = dto::envelope(
            "encryption_request",
            json!({
                "plaintext": plaintext,
                "key_id": "simulateur_communication_key"
            }),
        );

        let enc_url = format!("{}/encrypt", self.url);
        let enc_res = match self
            .client
            .post(&enc_url)
            .header("X-Agent-Token", &self.token)
            .json(&enc_body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return CryptoTestResult::Failed(format!("encrypt: {e}")),
        };

        if !enc_res.status().is_success() {
            return CryptoTestResult::Failed(format!("encrypt HTTP {}", enc_res.status()));
        }

        let enc_json: serde_json::Value = match enc_res.json().await {
            Ok(v) => v,
            Err(e) => return CryptoTestResult::Failed(format!("encrypt JSON: {e}")),
        };

        let dec_body = dto::envelope(
            "decryption_request",
            json!({
                "ciphertext": enc_json.get("ciphertext"),
                "iv": enc_json.get("iv"),
                "auth_tag": enc_json.get("auth_tag"),
                "key_id": "simulateur_communication_key"
            }),
        );

        let dec_url = format!("{}/decrypt", self.url);
        let dec_res = match self
            .client
            .post(&dec_url)
            .header("X-Agent-Token", &self.token)
            .json(&dec_body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return CryptoTestResult::Failed(format!("decrypt: {e}")),
        };

        if !dec_res.status().is_success() {
            return CryptoTestResult::Failed(format!("decrypt HTTP {}", dec_res.status()));
        }

        CryptoTestResult::Ok
    }

    pub async fn test_weak_password(&self, password: &str) -> CryptoTestResult {
        if self.dry_run {
            if password.len() < 8 {
                return CryptoTestResult::RejectedAsExpected;
            }
            return CryptoTestResult::Failed("dry_run: mot de passe trop fort pour le test".into());
        }

        let body = dto::envelope(
            "credential_generation_request",
            json!({
                "reason": "simulation_weak_password_test",
                "scope": [],
                "credential_types": ["password"],
                "priority": "low",
                "proposed_password": password
            }),
        );

        let url = format!("{}/credential/rotate", self.url);
        let res = match self
            .client
            .post(&url)
            .header("X-Agent-Token", &self.token)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return CryptoTestResult::Failed(format!("weak password test: {e}")),
        };

        if res.status().is_client_error() {
            CryptoTestResult::RejectedAsExpected
        } else if res.status().is_success() {
            CryptoTestResult::Failed("mot de passe faible accepté".into())
        } else {
            CryptoTestResult::Failed(format!("HTTP {}", res.status()))
        }
    }
}
