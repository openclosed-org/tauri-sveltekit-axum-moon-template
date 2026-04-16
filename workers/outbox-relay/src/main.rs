//! Outbox relay worker — main entry point.
//!
//! This worker polls the outbox table, publishes events to the event bus,
//! and tracks checkpoints and deduplication.
//!
//! Configuration is loaded via SOPS-encrypted secrets, never from `.env` files.
//! For local development: `just sops-run outbox-relay-worker`

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use event_bus::adapters::memory_bus::InMemoryEventBus;
use event_bus::ports::EventBus;
use runtime::adapters::memory::{MemoryPubSub, MemoryState};
use runtime::ports::state::StateEntry;
use runtime::ports::{MessageEnvelope, PubSub, State};
use storage_turso::embedded::EmbeddedTurso;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

mod checkpoint;
mod config;
mod dedupe;
mod idempotency;
mod polling;
mod publish;

use config::Config;
use idempotency::IdempotencyStore;
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
    // Load configuration from SOPS-encrypted environment variables
    let config = Config::from_env()?;

    // Initialize tracing with config
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.rust_log)
                .unwrap_or_else(|_| "outbox_relay_worker=info".into()),
        )
        .init();

    info!("Outbox relay worker starting");
    info!("Database: {}", config.database_url);
    info!("NATS URL: {}", config.nats_url);
    info!("Poll interval: {:?}", config.poll_interval());

    let state = Arc::new(WorkerState::new());

    // Start health check server using config
    let health_addr = config.health_addr();
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

    // Create idempotency store for exactly-once publishing
    let idempotency_store = IdempotencyStore::default();

    // Choose reader based on runtime config
    let mut poller = create_outbox_poller(&config).await;

    info!(
        "Outbox relay worker running (poll interval: {:?}, batch size: {}, checkpoint: {})",
        config.poll_interval(),
        config.batch_size,
        config.checkpoint_path
    );

    // Main processing loop
    loop {
        let entries = poller.poll_cycle().await;

        if !entries.is_empty() {
            // Filter out already-processed entries via idempotency store
            let mut to_publish = Vec::new();
            for entry in &entries {
                if idempotency_store.is_already_processed(&entry.id) {
                    info!(entry_id = %entry.id, "skipping already-processed entry");
                    continue;
                }
                if idempotency_store.start(&entry.id) {
                    to_publish.push(entry.clone());
                } else {
                    info!(entry_id = %entry.id, "skipping entry already in progress");
                }
            }

            if !to_publish.is_empty() {
                let (successes, failures) = publisher.publish_batch(&to_publish).await;
                state.record_success(successes.len()).await;
                state.record_failure(failures.len()).await;

                // Mark published in database for successful entries
                if !successes.is_empty() {
                    if let Err(e) = poller.mark_published(&successes).await {
                        error!(error = %e, "failed to mark outbox entries as published");
                        for id in &successes {
                            idempotency_store.fail(id, format!("mark_published failed: {}", e));
                        }
                    } else {
                        for id in &successes {
                            idempotency_store.complete(id);
                        }
                    }
                }

                // Mark failures in idempotency store
                for (id, err) in &failures {
                    idempotency_store.fail(id, err.to_string());
                }

                // Advance checkpoint and dedup
                poller.mark_processed(&entries);
            }
        }

        // Sleep for the poll interval
        tokio::time::sleep(config.poll_interval()).await;
    }
}

/// Create the appropriate outbox poller based on configuration.
/// Uses libsql-backed reader for production, in-memory for testing.
async fn create_outbox_poller(config: &Config) -> OutboxPoller<Box<dyn OutboxReader>> {
    let poller_config = PollerConfig {
        poll_interval: config.poll_interval(),
        batch_size: config.batch_size,
    };

    // Try to create libsql reader; fall back to memory reader if database unavailable
    match EmbeddedTurso::new(&config.database_url).await {
        Ok(turso) => {
            let reader = polling::LibSqlOutboxReader::new(turso);
            OutboxPoller::new(Box::new(reader), poller_config, &config.checkpoint_path)
        }
        Err(e) => {
            warn!(
                error = %e,
                "failed to create libsql outbox reader, using in-memory stub"
            );
            let reader = MemoryOutboxReader::new(Vec::new());
            OutboxPoller::new(Box::new(reader), poller_config, &config.checkpoint_path)
        }
    }
}
