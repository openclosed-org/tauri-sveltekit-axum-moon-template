//! Standalone server binary — runs the Axum HTTP server.
//!
//! Usage: cargo run -p runtime_server
//!
//! Listens on 0.0.0.0:3001 by default.
//! Override with SERVER_PORT env var or config.toml.

use runtime_server::{config::Config, create_router, error::AppError, state::AppState};
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Bridge all `log` crate usage (from dependencies) into tracing
    let _ = tracing_log::LogTracer::init();

    // Initialize tracing with structured logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,runtime_server=debug"));
    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(filter)
        .try_init();

    // Try to load config from environment, fall back to defaults
    let config = Config::from_env().unwrap_or_default();

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| config.server.port.to_string())
            .parse()
            .unwrap_or(config.server.port),
    ));

    // Initialize shared application state
    let state = AppState::new_dev().await?;
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Config(format!("Failed to bind to {}: {}", addr, e)))?;

    tracing::info!("🚀 Runtime server listening on {}", addr);
    tracing::info!("   Health: http://localhost:{}/healthz", config.server.port);
    tracing::info!("   Ready:  http://localhost:{}/readyz", config.server.port);

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
