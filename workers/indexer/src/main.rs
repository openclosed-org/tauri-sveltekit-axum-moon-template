//! Indexer worker — pulls events from sources, transforms, and writes to sinks.
//!
//! Migrated from `servers/indexer/` to `workers/indexer/` per architecture rules.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use contracts_events::event_type_name;
use runtime::adapters::memory::{MemoryPubSub, MemoryState};
use runtime::ports::state::StateEntry;
use runtime::ports::{MessageEnvelope, PubSub, State};
use tokio::sync::RwLock;
use tracing::{info, warn};

mod checkpoint;
mod sinks;
mod sources;
mod transforms;

use checkpoint::SourceCheckpoint;
use sinks::{EventSink, IndexedEvent, MemoryEventSink};
use sources::{EventSource, RawEvent};
use transforms::{EventTransform, TransformedEvent};

/// Indexer error types.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("Source error: {0}")]
    Source(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Sink error: {0}")]
    Sink(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Worker state.
struct WorkerState {
    healthy: RwLock<bool>,
    indexed_count: RwLock<u64>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            indexed_count: RwLock::new(0),
        }
    }

    async fn record_indexed(&self, count: usize) {
        let mut guard = self.indexed_count.write().await;
        *guard += count as u64;
    }
}

/// Health check endpoint.
async fn healthz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let indexed = state.indexed_count.read().await;
    axum::Json(serde_json::json!({
        "status": "ok",
        "indexed_count": *indexed,
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
    info!("Indexer health server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

/// The indexer — coordinates pulling events from sources, transforming, and sinking.
pub struct Indexer {
    sources: Vec<Box<dyn EventSource>>,
    transformers: Vec<Box<dyn EventTransform>>,
    sinks: Vec<Box<dyn EventSink>>,
    checkpoint: Arc<SourceCheckpoint>,
    state: MemoryState,
    pubsub: MemoryPubSub,
}

impl Indexer {
    pub fn new(state: MemoryState, pubsub: MemoryPubSub) -> Self {
        Self {
            sources: Vec::new(),
            transformers: Vec::new(),
            sinks: Vec::new(),
            checkpoint: Arc::new(SourceCheckpoint::new()),
            state,
            pubsub,
        }
    }

    pub fn add_source(&mut self, source: Box<dyn EventSource>) {
        self.sources.push(source);
    }

    pub fn add_transformer(&mut self, transformer: Box<dyn EventTransform>) {
        self.transformers.push(transformer);
    }

    pub fn add_sink(&mut self, sink: Box<dyn EventSink>) {
        self.sinks.push(sink);
    }

    /// Run a full indexing cycle.
    pub async fn run_cycle(&self) -> Result<usize, IndexerError> {
        let mut total_indexed = 0;

        // 1. Pull events from all sources
        let mut raw_events = Vec::new();
        for source in &self.sources {
            let cursor = self.checkpoint.get(source.name());
            let events = source.pull_events(cursor.as_deref()).await?;
            raw_events.extend(events);
        }

        if raw_events.is_empty() {
            return Ok(0);
        }

        info!(count = raw_events.len(), "pulled events from sources");

        // 2. Transform raw events to canonical EventEnvelope
        let mut indexed_events = Vec::new();
        for raw in raw_events {
            for transformer in &self.transformers {
                if transformer.can_transform(&raw) {
                    if let Some(TransformedEvent { envelope }) = transformer.transform(&raw).await?
                    {
                        let event_type = event_type_name(&envelope.event);

                        let indexed = IndexedEvent {
                            id: envelope.id.to_string(),
                            event_type: event_type.to_string(),
                            source: raw.source.clone(),
                            payload: serde_json::to_string(&envelope.event)
                                .map_err(|e| IndexerError::Transform(format!("serialize: {e}")))?,
                            metadata: envelope.metadata,
                            indexed_at: chrono::Utc::now().to_rfc3339(),
                        };
                        indexed_events.push(indexed);
                    }
                    break;
                }
            }
        }

        if indexed_events.is_empty() {
            return Ok(0);
        }

        // 3. Write to all sinks
        for event in &indexed_events {
            for sink in &self.sinks {
                sink.write(event).await?;
                total_indexed += 1;
            }
        }

        // 4. Publish indexed event to pubsub for downstream consumers
        for event in &indexed_events {
            if let Ok(app_event) =
                serde_json::from_str::<contracts_events::AppEvent>(&event.payload)
            {
                let envelope = MessageEnvelope::new(
                    app_event,
                    format!("indexer.{}", event.event_type),
                    "indexer-worker",
                )
                .with_metadata(event.metadata.clone());

                if let Err(e) = self
                    .pubsub
                    .publish(&format!("indexer.{}", event.event_type), envelope)
                    .await
                {
                    warn!(error = %e, "failed to publish indexed event");
                }
            }
        }

        // 5. Update checkpoints
        for source in &self.sources {
            // In a real implementation, the source would return a cursor
            // For now, we use a placeholder
            self.checkpoint.update(source.name(), "latest".to_string());
        }

        // 6. Persist checkpoint to state storage
        let checkpoint_data = self.checkpoint.list();
        let checkpoint_json = serde_json::to_string(&checkpoint_data)
            .map_err(|e| IndexerError::RuntimeError(format!("checkpoint serialize: {e}")))?;

        let checkpoint_entry = StateEntry::new("indexer:checkpoints", checkpoint_json, None);
        if let Err(e) = self.state.set(checkpoint_entry).await {
            warn!(error = %e, "failed to persist checkpoint");
        }

        info!(count = total_indexed, "indexing cycle complete");
        Ok(total_indexed)
    }
}

impl Default for Indexer {
    fn default() -> Self {
        // Default uses memory adapters
        Self::new(MemoryState::new(), MemoryPubSub::new())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use contracts_events::{AppEvent, CounterChanged, EventEnvelope};
    use runtime::adapters::memory::{MemoryPubSub, MemoryState};
    use tokio::sync::Mutex;

    use super::*;
    use crate::sinks::{EventSink, IndexedEvent, MemoryEventSink};
    use crate::sources::{MemoryEventSource, RawEvent};
    use crate::transforms::PassthroughTransform;

    struct SharedMemorySink {
        inner: Arc<MemoryEventSink>,
    }

    #[async_trait]
    impl EventSink for SharedMemorySink {
        fn name(&self) -> &str {
            self.inner.name()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        async fn write(&self, event: &IndexedEvent) -> Result<(), IndexerError> {
            self.inner.write(event).await
        }
    }

    #[tokio::test]
    async fn run_cycle_preserves_envelope_identity_and_correlation() {
        let base_envelope = EventEnvelope::new(
            AppEvent::CounterChanged(CounterChanged {
                tenant_id: "tenant-a".to_string(),
                counter_key: "counter-a".to_string(),
                operation: "increment".to_string(),
                new_value: 1,
                delta: 1,
                version: 1,
            }),
            "counter-service",
        )
        .with_correlation_id("req-idx-1");
        let envelope_id = base_envelope.id.to_string();

        let raw = RawEvent {
            source: "source-a".to_string(),
            raw_payload: serde_json::to_string(&base_envelope).unwrap(),
            timestamp: "now".to_string(),
            metadata: HashMap::new(),
        };

        let state = MemoryState::new();
        let pubsub = MemoryPubSub::new();
        let mut indexer = Indexer::new(state, pubsub);
        let sink = Arc::new(MemoryEventSink {
            events: Mutex::new(Vec::new()),
        });
        indexer.add_source(Box::new(MemoryEventSource::new(vec![raw])));
        indexer.add_transformer(Box::new(PassthroughTransform));
        indexer.add_sink(Box::new(SharedMemorySink {
            inner: sink.clone(),
        }));

        let count = indexer.run_cycle().await.unwrap();
        assert_eq!(count, 1);

        let events = sink.events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, envelope_id);
        assert_eq!(
            events[0].metadata.correlation_id.as_deref(),
            Some("req-idx-1")
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _observability = observability::init_observability("indexer-worker", "indexer_worker=info")
        .map_err(anyhow::Error::msg)?;

    info!("Indexer worker starting");

    let state = Arc::new(WorkerState::new());

    // Health server
    let health_addr: SocketAddr = "0.0.0.0:3031".parse()?;
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    // Build indexer with runtime ports (memory adapters for now)
    let mut indexer = Indexer::default();
    indexer.add_source(Box::new(sources::MemoryEventSource::new(Vec::new())));
    indexer.add_transformer(Box::new(transforms::PassthroughTransform));
    indexer.add_sink(Box::new(MemoryEventSink::new()));

    info!("Indexer worker running (memory adapter mode)");

    // Main loop
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        match indexer.run_cycle().await {
            Ok(count) => {
                state.record_indexed(count).await;
            }
            Err(e) => {
                warn!(error = %e, "indexing cycle failed");
            }
        }
    }
}
