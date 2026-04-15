//! Event publisher — publishes outbox entries to the event bus and runtime pubsub.

use async_trait::async_trait;
use contracts_events::AppEvent;
use event_bus::ports::{EventBus, EventEnvelope};
use runtime::ports::{MessageEnvelope as RuntimeMessageEnvelope, PubSub};
use tracing::{debug, warn};

use crate::polling::PendingOutboxEntry;

/// Error type for publishing.
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Failed to deserialize event: {0}")]
    Deserialize(String),

    #[error("Failed to publish to event bus: {0}")]
    Bus(String),

    #[error("Failed to publish to pubsub: {0}")]
    PubSub(String),
}

/// Publishes outbox entries to both event bus and runtime pubsub.
pub struct OutboxPublisher<E: EventBus, P: PubSub> {
    event_bus: E,
    pubsub: P,
}

impl<E: EventBus, P: PubSub> OutboxPublisher<E, P> {
    pub fn new(event_bus: E, pubsub: P) -> Self {
        Self { event_bus, pubsub }
    }

    /// Publish a single outbox entry to both event bus and pubsub.
    pub async fn publish(&self, entry: &PendingOutboxEntry) -> Result<(), PublishError> {
        let event: AppEvent = serde_json::from_str(&entry.payload)
            .map_err(|e| PublishError::Deserialize(e.to_string()))?;

        // Publish to event bus (for service-to-service communication)
        let envelope = EventEnvelope::new(event.clone(), &entry.source_service);

        self.event_bus
            .publish(envelope)
            .await
            .map_err(|e| PublishError::Bus(e.to_string()))?;

        // Publish to runtime pubsub (for workers and external consumers)
        let runtime_envelope = RuntimeMessageEnvelope::new(
            event,
            format!("outbox.{}", entry.event_type),
            &entry.source_service,
        );

        self.pubsub
            .publish(&format!("outbox.{}", entry.event_type), runtime_envelope)
            .await
            .map_err(|e| PublishError::PubSub(e.to_string()))?;

        debug!(entry_id = %entry.id, "published outbox entry to event bus and pubsub");
        Ok(())
    }

    /// Publish a batch of outbox entries, returning successes and failures.
    pub async fn publish_batch(
        &self,
        entries: &[PendingOutboxEntry],
    ) -> (Vec<String>, Vec<(String, PublishError)>) {
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for entry in entries {
            match self.publish(entry).await {
                Ok(()) => successes.push(entry.id.clone()),
                Err(e) => {
                    warn!(entry_id = %entry.id, error = %e, "failed to publish outbox entry");
                    failures.push((entry.id.clone(), e));
                }
            }
        }

        (successes, failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_bus::adapters::memory_bus::InMemoryEventBus;
    use runtime::adapters::memory::MemoryPubSub;

    fn test_entry(id: &str, payload: &str) -> PendingOutboxEntry {
        PendingOutboxEntry {
            id: id.to_string(),
            sequence: 1,
            event_type: "counter.changed".to_string(),
            payload: payload.to_string(),
            source_service: "counter-service".to_string(),
            retry_count: 0,
        }
    }

    #[tokio::test]
    async fn publishes_valid_event() {
        let bus = InMemoryEventBus::new();
        let pubsub = MemoryPubSub::new();
        let publisher = OutboxPublisher::new(bus, pubsub);

        // CounterChanged event with valid JSON (AppEvent uses internally tagged format)
        let event = contracts_events::AppEvent::CounterChanged(contracts_events::CounterChanged {
            tenant_id: "test-tenant".to_string(),
            counter_key: "default".to_string(),
            operation: "increment".to_string(),
            new_value: 42,
            delta: 1,
            version: 1,
        });
        let payload = serde_json::to_string(&event).unwrap();

        let entry = test_entry("entry-1", &payload);
        let result = publisher.publish(&entry).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn returns_error_for_invalid_json() {
        let bus = InMemoryEventBus::new();
        let pubsub = MemoryPubSub::new();
        let publisher = OutboxPublisher::new(bus, pubsub);

        let entry = test_entry("entry-1", "not valid json");
        let result = publisher.publish(&entry).await;
        assert!(matches!(result, Err(PublishError::Deserialize(_))));
    }
}
