use crate::config::Config;
use reqwest::Client;

pub async fn check_datacenter_health(cfg: &Config) -> Result<(), String> {
    if cfg.dry_run || !cfg.enforce_health_checks {
        return Ok(());
    }

    let client = Client::new();
    for (name, base) in [
        ("decideur", cfg.decideur_url.as_str()),
        ("analyseur", cfg.analyseur_url.as_str()),
    ] {
        let url = format!("{base}/health");
        let res = client
            .get(&url)
            .header("X-Agent-Token", &cfg.agent_token)
            .send()
            .await
            .map_err(|e| format!("C7: {name} injoignable: {e}"))?;

        if !res.status().is_success() {
            return Err(format!(
                "C7: {name} état non nominal HTTP {}",
                res.status()
            ));
        }
    }
    Ok(())
}
