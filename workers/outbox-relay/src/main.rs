//! Outbox relay worker — main entry point.
//!
//! This worker polls the outbox table, publishes events to the event bus,
//! and tracks checkpoints and deduplication.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use event_bus::adapters::memory_bus::InMemoryEventBus;
use event_bus::ports::EventBus;
use runtime::ports::{PubSub, MessageEnvelope, State};
use runtime::ports::state::StateEntry;
use runtime::adapters::memory::{MemoryPubSub, MemoryState};
use tokio::sync::RwLock;
use tracing::{info, warn};

mod checkpoint;
mod dedupe;
mod polling;
mod publish;

use polling::{MemoryOutboxReader, OutboxPoller, OutboxReader, PendingOutboxEntry, PollerConfig};
use publish::OutboxPublisher;

/// Worker state shared across tasks.
struct WorkerState {
    healthy: RwLock<bool>,
    processed_count: RwLock<u64>,
    failed_count: RwLock<u64>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            processed_count: RwLock::new(0),
            failed_count: RwLock::new(0),
        }
    }

    async fn record_success(&self, count: usize) {
        let mut guard = self.processed_count.write().await;
        *guard += count as u64;
    }

    async fn record_failure(&self, count: usize) {
        let mut guard = self.failed_count.write().await;
        *guard += count as u64;
    }
}

/// Health check endpoint.
async fn healthz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let healthy = state.healthy.read().await;
    let processed = state.processed_count.read().await;
    let failed = state.failed_count.read().await;

    axum::Json(serde_json::json!({
        "status": if *healthy { "ok" } else { "unhealthy" },
        "processed_count": *processed,
        "failed_count": *failed,
    }))
}

/// Readiness check endpoint.
async fn readyz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let healthy = state.healthy.read().await;
    if *healthy {
        axum::Json(serde_json::json!({ "status": "ready" }))
    } else {
        axum::Json(serde_json::json!({ "status": "not ready" }))
    }
}

/// Start the health check HTTP server.
async fn start_health_server(state: Arc<WorkerState>, addr: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Health check server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "outbox_relay_worker=info".into()),
        )
        .init();

    info!("Outbox relay worker starting");

    let state = Arc::new(WorkerState::new());

    // Start health check server
    let health_addr: SocketAddr = "0.0.0.0:3030".parse()?;
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    // Create event bus (in-memory for now; would be NATS in production)
    let event_bus = InMemoryEventBus::new();

    // Create runtime pubsub (in-memory for now; would be NATS in production)
    let pubsub = MemoryPubSub::new();

    // Create outbox publisher with both event bus and pubsub
    let publisher = OutboxPublisher::new(event_bus, pubsub);

    // Create outbox reader (stub for now; would be Turso/SQLite in production)
    let reader = MemoryOutboxReader::new(Vec::new());
    let config = PollerConfig::default();
    let mut poller = OutboxPoller::new(reader, config.clone());

    info!("Outbox relay worker running (poll interval: {:?})", config.poll_interval);

    // Main processing loop
    loop {
        let entries = poller.poll_cycle().await;

        if !entries.is_empty() {
            let (successes, failures) = publisher.publish_batch(&entries).await;
            state.record_success(successes.len()).await;
            state.record_failure(failures.len()).await;
            poller.mark_processed(&entries);
        }

        // Sleep for the poll interval
        tokio::time::sleep(config.poll_interval).await;
    }
}
