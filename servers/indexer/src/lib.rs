//! Protocol event indexer.
//!
//! Pulls events from various sources, normalizes them to business DTOs,
//! and writes to Turso for read-optimized queries.

pub mod sinks;
pub mod sources;
pub mod transformers;

use std::sync::Arc;

use contracts_events::AppEvent;
use serde::{Deserialize, Serialize};

use self::sources::{EventSource, RawEvent};
use self::transformers::EventTransformer;
use self::sinks::EventSink;

/// Indexed event record — stored in Turso for query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEvent {
    pub id: String,
    pub event_type: String,
    pub tenant_id: Option<String>,
    pub user_sub: Option<String>,
    pub payload: String, // JSON-serialized AppEvent
    pub indexed_at: String,
}

/// Indexer error types.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("Source error: {0}")]
    Source(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Sink error: {0}")]
    Sink(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// The indexer — coordinates pulling events from sources, transforming, and sinking.
pub struct Indexer {
    sources: Vec<Box<dyn EventSource>>,
    transformers: Vec<Box<dyn EventTransformer>>,
    sinks: Vec<Box<dyn EventSink>>,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            transformers: Vec::new(),
            sinks: Vec::new(),
        }
    }

    /// Register an event source.
    pub fn add_source(&mut self, source: Box<dyn EventSource>) {
        self.sources.push(source);
    }

    /// Register an event transformer.
    pub fn add_transformer(&mut self, transformer: Box<dyn EventTransformer>) {
        self.transformers.push(transformer);
    }

    /// Register an event sink.
    pub fn add_sink(&mut self, sink: Box<dyn EventSink>) {
        self.sinks.push(sink);
    }

    /// Run a full indexing cycle.
    pub async fn run_cycle(&self) -> Result<usize, IndexerError> {
        let mut total_indexed = 0;

        // 1. Pull events from all sources
        let mut raw_events = Vec::new();
        for source in &self.sources {
            let events = source.pull_events().await?;
            raw_events.extend(events);
        }

        if raw_events.is_empty() {
            return Ok(0);
        }

        tracing::info!(count = raw_events.len(), "Pulled events from sources");

        // 2. Transform raw events to AppEvent
        let mut app_events = Vec::new();
        for raw in raw_events {
            for transformer in &self.transformers {
                if transformer.can_transform(&raw) {
                    let transformed = transformer
                        .transform(&raw)
                        .await?;
                    if let Some(event) = transformed {
                        app_events.push(event);
                    }
                    break; // Only transform with the first matching transformer
                }
            }
        }

        if app_events.is_empty() {
            return Ok(0);
        }

        // 3. Write to all sinks
        for event in &app_events {
            for sink in &self.sinks {
                sink.write(event).await?;
                total_indexed += 1;
            }
        }

        tracing::info!(count = total_indexed, "Indexing cycle complete");
        Ok(total_indexed)
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self::new()
    }
}
