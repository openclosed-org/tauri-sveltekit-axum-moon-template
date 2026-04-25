//! Projector worker — builds read models from event streams.
//!
//! Consumes replayable events from the outbox, runs them through interested
//! consumers, and updates materialized read models.

#![deny(unused_imports, unused_variables)]

use event_bus::ports::EventEnvelope;
use storage_turso::TursoBackend;
use tracing::info;
use worker_runtime::{
    FileCheckpointStore, WorkerHealthState, bootstrap_worker, build_checkpoint_store,
    shutdown_signal, spawn_health_server,
};

mod checkpoint;
mod config;
mod consumers;
mod error;
mod live;
mod readmodels;
mod replay;
mod source;

use checkpoint::ProjectionCheckpointPort;
use config::Config;
use consumers::EventConsumer;
use error::ProjectorError;
use live::LiveEventSubscriber;
use readmodels::{ReadModel, SqliteCounterReadModel};
use replay::{ReplayManager, ReplayStrategy};
use source::CounterOutboxSource;

/// The projector — consumes events and updates read models.
pub struct Projector {
    consumers: Vec<Box<dyn EventConsumer>>,
    read_models: Vec<Box<dyn ReadModel>>,
    checkpoint: Box<dyn ProjectionCheckpointPort>,
}

impl Projector {
    pub fn new() -> Self {
        Self {
            consumers: Vec::new(),
            read_models: Vec::new(),
            checkpoint: Box::new(FileCheckpointStore::new(
                "/tmp/projector-checkpoint.json",
                0,
            )),
        }
    }

    pub fn with_checkpoint(checkpoint: Box<dyn ProjectionCheckpointPort>) -> Self {
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

    pub fn checkpoint(&self) -> &dyn ProjectionCheckpointPort {
        self.checkpoint.as_ref()
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

    let worker_runtime::WorkerBootstrap {
        observability: _observability,
        state,
    } = bootstrap_worker("projector-worker", &config.rust_log)?;

    info!("Projector worker starting");
    info!("Database: {}", config.database_url);

    spawn_health_server(state.clone(), config.health_addr(), "projector-worker");

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

    let (checkpoint, checkpoint_backend) = build_checkpoint_store(
        "projector-worker",
        &config.database_url,
        config.turso_auth_token.as_deref(),
        &config.checkpoint_path,
        0,
    )
    .await?;

    let mut projector = Projector::with_checkpoint(checkpoint);
    projector.add_consumer(Box::new(consumers::LoggingConsumer));
    projector.add_consumer(Box::new(consumers::CounterStateConsumer::new()));
    projector.add_read_model(Box::new(sqlite_read_model));

    let replay = ReplayManager::new(ReplayStrategy::Checkpoint)
        .with_fallback_checkpoint(projector.checkpoint().get().await.unwrap_or(0));

    info!(
        "Projector worker running (poll interval: {:?}, batch size: {}, checkpoint: {}, store_backend: {})",
        config.poll_interval(),
        config.batch_size,
        config.checkpoint_path,
        checkpoint_backend.as_str(),
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
            tokio::select! {
                _ = shutdown_signal() => {
                    state.set_healthy(false).await;
                    info!("shutdown signal received, stopping projector worker");
                    return Ok(());
                }
                result = live.try_next(config.poll_interval()) => {
                    if let Some(envelope) = result? {
                        let projected = projector.process_event(&envelope).await?;
                        state.record_count("projected_count", projected).await;
                    }
                }
            }
        }
    }

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                state.set_healthy(false).await;
                info!("shutdown signal received, stopping projector worker");
                break;
            }
            _ = tokio::time::sleep(config.poll_interval()) => {}
        }

        let events = source
            .fetch_since(
                projector.checkpoint().get().await.unwrap_or(0),
                config.batch_size,
            )
            .await?;

        if !events.is_empty() {
            let mut projected = 0;
            for event in events {
                projected += projector.process_event(&event.envelope).await?;
                let current = projector.checkpoint().get().await.unwrap_or(0);
                if event.sequence > current {
                    let _ = projector.checkpoint().advance(event.sequence).await;
                }
            }
            state.record_count("projected_count", projected).await;
        }
    }

    Ok(())
}

async fn replay_outbox(
    source: &CounterOutboxSource<TursoBackend>,
    projector: &Projector,
    state: &WorkerHealthState,
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
            let current = projector.checkpoint().get().await.unwrap_or(0);
            if event.sequence > current {
                let _ = projector.checkpoint().advance(event.sequence).await;
            }
            since = event.sequence;
        }
        state.record_count("projected_count", projected).await;
    }
}
