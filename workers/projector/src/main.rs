//! Projector worker — builds read models from event streams.
//!
//! Consumes events from the event bus, runs them through interested
//! consumers, and updates materialized read models.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use event_bus::adapters::memory_bus::InMemoryEventBus;
use event_bus::ports::{EventBus, EventEnvelope};
use runtime::adapters::memory::{MemoryPubSub, MemoryState};
use runtime::ports::{PubSub, State};
use tokio::sync::RwLock;
use tracing::{info, warn};

mod checkpoint;
mod consumers;
mod error;
mod readmodels;

use checkpoint::ProjectionCheckpoint;
use consumers::EventConsumer;
use error::ProjectorError;
use readmodels::ReadModel;

/// Worker state.
struct WorkerState {
    healthy: RwLock<bool>,
    projected_count: RwLock<u64>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            projected_count: RwLock::new(0),
        }
    }

    async fn record_projected(&self, count: usize) {
        let mut guard = self.projected_count.write().await;
        *guard += count as u64;
    }
}

async fn healthz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let projected = state.projected_count.read().await;
    axum::Json(serde_json::json!({
        "status": "ok",
        "projected_count": *projected,
    }))
}

async fn readyz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let healthy = state.healthy.read().await;
    if *healthy {
        axum::Json(serde_json::json!({ "status": "ready" }))
    } else {
        axum::Json(serde_json::json!({ "status": "not ready" }))
    }
}

async fn start_health_server(state: Arc<WorkerState>, addr: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Projector health server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

/// The projector — consumes events and updates read models.
pub struct Projector {
    consumers: Vec<Box<dyn EventConsumer>>,
    read_models: Vec<Box<dyn ReadModel>>,
    checkpoint: ProjectionCheckpoint,
}

impl Projector {
    pub fn new() -> Self {
        Self {
            consumers: Vec::new(),
            read_models: Vec::new(),
            checkpoint: ProjectionCheckpoint::new(0),
        }
    }

    pub fn add_consumer(&mut self, consumer: Box<dyn EventConsumer>) {
        self.consumers.push(consumer);
    }

    pub fn add_read_model(&mut self, model: Box<dyn ReadModel>) {
        self.read_models.push(model);
    }

    /// Process a single event through the projection pipeline.
    pub async fn process_event(&self, envelope: &EventEnvelope) -> Result<usize, ProjectorError> {
        let mut projected = 0;

        for consumer in &self.consumers {
            if consumer.is_interested(&envelope.event)
                && let Some(update) = consumer.consume(envelope).await?
            {
                for model in &self.read_models {
                    model.apply_update(&update).await?;
                    projected += 1;
                }
            }
        }

        Ok(projected)
    }
}

impl Default for Projector {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "projector_worker=info".into()),
        )
        .init();

    info!("Projector worker starting");

    let state = Arc::new(WorkerState::new());

    let health_addr: SocketAddr = "0.0.0.0:3032".parse()?;
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    // Build projector with stub components
    let mut projector = Projector::new();
    projector.add_consumer(Box::new(consumers::LoggingConsumer));
    projector.add_read_model(Box::new(readmodels::MemoryReadModel::new()));

    // Subscribe to the event bus
    let event_bus = InMemoryEventBus::new();

    // Initialize runtime ports for projection state and pubsub
    let projection_state = MemoryState::new();
    let pubsub = MemoryPubSub::new();

    info!("Projector worker running with runtime ports");

    // In a real implementation, we'd subscribe to the event bus here.
    // For now, just keep the worker alive.
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
