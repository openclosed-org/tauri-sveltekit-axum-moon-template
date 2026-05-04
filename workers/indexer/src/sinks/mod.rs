//! Event sinks — write indexed events to storage.

use std::any::Any;

use async_trait::async_trait;
use contracts_events::{AppEvent, AppEvent::CounterChanged, EventMetadata, event_type_name};

use crate::IndexerError;

/// Indexed event record — stored for query.
#[derive(Debug, Clone)]
pub struct IndexedEvent {
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub payload: String, // JSON-serialized AppEvent
    pub metadata: EventMetadata,
    pub indexed_at: String,
}

/// Event sink trait — writes events to a storage layer.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Name of this sink (e.g., "turso-events").
    fn name(&self) -> &str;

    /// Downcast hook used by tests and diagnostics.
    fn as_any(&self) -> &dyn Any;

    /// Write an event to the sink.
    async fn write(&self, event: &IndexedEvent) -> Result<(), IndexerError>;
}

/// In-memory stub sink for testing — collects events.
pub struct MemoryEventSink {
    pub events: tokio::sync::Mutex<Vec<IndexedEvent>>,
}

impl MemoryEventSink {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MemoryEventSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventSink for MemoryEventSink {
    fn name(&self) -> &str {
        "memory-sink"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn write(&self, event: &IndexedEvent) -> Result<(), IndexerError> {
        self.events.lock().await.push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_sink_collects_events() {
        let sink = MemoryEventSink::new();
        let event = IndexedEvent {
            id: "evt-1".to_string(),
            event_type: event_type_name(&CounterChanged(contracts_events::CounterChanged {
                tenant_id: "tenant-a".to_string(),
                counter_key: "tenant-a".to_string(),
                operation: contracts_events::CounterOperation::Increment,
                new_value: 1,
                delta: 1,
                version: 1,
            }))
            .to_string(),
            source: "test".to_string(),
            payload: "{}".to_string(),
            metadata: EventMetadata::for_event(&AppEvent::CounterChanged(
                contracts_events::CounterChanged {
                    tenant_id: "tenant-a".to_string(),
                    counter_key: "tenant-a".to_string(),
                    operation: contracts_events::CounterOperation::Increment,
                    new_value: 1,
                    delta: 1,
                    version: 1,
                },
            )),
            indexed_at: "now".to_string(),
        };

        sink.write(&event).await.unwrap();
        let events = sink.events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "evt-1");
    }
}
