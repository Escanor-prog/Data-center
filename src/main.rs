mod api;
mod comms;
mod config;
mod control;
mod scenarios;

use api::routes::{build_router, AppState};
use comms::AgentClients;
use config::Config;
use control::metrics::SimulationMetrics;
use control::state::new_shared_state;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cfg = Arc::new(Config::load().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?);

    let clients = Arc::new(AgentClients::from_config(&cfg));
    let state = new_shared_state();
    let metrics = Arc::new(SimulationMetrics::default());

    let app_state = AppState {
        config: cfg.clone(),
        clients,
        control_state: state,
        metrics,
        started_at: Instant::now(),
    };

    let app = build_router(app_state);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.listen_port));

    tracing::info!(%addr, dry_run = cfg.dry_run, "Agent Simulateur démarré");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
