//! Projector worker — builds read models from event streams.
//!
//! Consumes replayable events from the outbox, runs them through interested
//! consumers, and updates materialized read models.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use event_bus::ports::EventEnvelope;
use storage_turso::TursoBackend;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod checkpoint;
mod config;
mod consumers;
mod error;
mod live;
mod readmodels;
mod replay;
mod source;

use checkpoint::ProjectionCheckpoint;
use config::Config;
use consumers::EventConsumer;
use error::ProjectorError;
use live::LiveEventSubscriber;
use readmodels::{ReadModel, SqliteCounterReadModel};
use replay::{ReplayManager, ReplayStrategy};
use source::CounterOutboxSource;

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
            checkpoint: ProjectionCheckpoint::new(0, "/tmp/projector-checkpoint.json"),
        }
    }

    pub fn with_checkpoint(checkpoint: ProjectionCheckpoint) -> Self {
        Self {
            consumers: Vec::new(),
            read_models: Vec::new(),
            checkpoint,
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

    pub fn checkpoint(&self) -> &ProjectionCheckpoint {
        &self.checkpoint
    }
}

impl Default for Projector {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    let _observability = observability::init_observability("projector-worker", &config.rust_log)
        .map_err(anyhow::Error::msg)?;

    info!("Projector worker starting");
    info!("Database: {}", config.database_url);

    let state = Arc::new(WorkerState::new());

    let health_addr: SocketAddr = config.health_addr();
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    let db = TursoBackend::connect(&config.database_url, config.turso_auth_token.as_deref())
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "failed to open projector database '{}': {e}",
                config.database_url
            )
        })?;
    let source = CounterOutboxSource::new(db.clone());
    let sqlite_read_model = SqliteCounterReadModel::new(db.clone());
    sqlite_read_model.init().await?;

    let mut projector =
        Projector::with_checkpoint(ProjectionCheckpoint::new(0, &config.checkpoint_path));
    projector.add_consumer(Box::new(consumers::LoggingConsumer));
    projector.add_consumer(Box::new(consumers::CounterStateConsumer::new()));
    projector.add_read_model(Box::new(sqlite_read_model));

    let replay = ReplayManager::new(ReplayStrategy::Checkpoint)
        .with_fallback_checkpoint(projector.checkpoint().get());

    info!(
        "Projector worker running (poll interval: {:?}, batch size: {}, checkpoint: {})",
        config.poll_interval(),
        config.batch_size,
        config.checkpoint_path
    );

    replay_outbox(
        &source,
        &projector,
        &state,
        replay.start_sequence(),
        config.batch_size,
    )
    .await?;

    if let Some(nats_url) = &config.nats_url {
        let queue_group =
            (!config.nats_queue_group.is_empty()).then_some(config.nats_queue_group.as_str());
        info!(subject = %config.nats_subject, queue_group = %config.nats_queue_group, "projector switching to live NATS tail");
        let mut live =
            LiveEventSubscriber::connect(nats_url, &config.nats_subject, queue_group).await?;

        loop {
            if let Some(envelope) = live.try_next(config.poll_interval()).await? {
                let projected = projector.process_event(&envelope).await?;
                state.record_projected(projected).await;
            }
        }
    }

    loop {
        let events = source
            .fetch_since(projector.checkpoint().get(), config.batch_size)
            .await?;

        if !events.is_empty() {
            let mut projected = 0;
            for event in events {
                projected += projector.process_event(&event.envelope).await?;
                projector.checkpoint().advance(event.sequence);
            }
            state.record_projected(projected).await;
        }

        tokio::time::sleep(config.poll_interval()).await;
    }
}

async fn replay_outbox(
    source: &CounterOutboxSource<TursoBackend>,
    projector: &Projector,
    state: &Arc<WorkerState>,
    start_sequence: u64,
    batch_size: usize,
) -> Result<(), ProjectorError> {
    let mut since = start_sequence;

    loop {
        let events = source.fetch_since(since, batch_size).await?;
        if events.is_empty() {
            return Ok(());
        }

        let mut projected = 0;
        for event in events {
            projected += projector.process_event(&event.envelope).await?;
            projector.checkpoint().advance(event.sequence);
            since = event.sequence;
        }
        state.record_projected(projected).await;
    }
}
