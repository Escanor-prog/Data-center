mod api;
mod comms;
mod control;
mod scenarios;

use control::{metrics::SimulationMetrics, state::new_shared_state};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

use api::routes::{build_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let state = new_shared_state();
    let metrics = Arc::new(SimulationMetrics::default());
    let app_state = AppState { control_state: state, metrics };

    let app = build_router(app_state);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8005));

    tracing::info!(%addr, "Starting Agent Simulateur HTTP server");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
