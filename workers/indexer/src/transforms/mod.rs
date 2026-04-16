//! Event transformers — raw protocol events → business DTOs.

use async_trait::async_trait;
use contracts_events::{AppEvent, EventEnvelope};

use crate::IndexerError;
use crate::sources::RawEvent;

/// Canonical transformed event representation used inside the indexer.
#[derive(Debug, Clone)]
pub struct TransformedEvent {
    pub envelope: EventEnvelope,
}

/// Event transformer trait.
#[async_trait]
pub trait EventTransform: Send + Sync {
    /// Name of this transformer.
    fn name(&self) -> &str;

    /// Check if this transformer can handle the raw event.
    fn can_transform(&self, raw: &RawEvent) -> bool;

    /// Transform a raw event to the canonical EventEnvelope.
    async fn transform(&self, raw: &RawEvent) -> Result<Option<TransformedEvent>, IndexerError>;
}

/// Stub transformer for testing.
pub struct PassthroughTransform;

#[async_trait]
impl EventTransform for PassthroughTransform {
    fn name(&self) -> &str {
        "passthrough"
    }

    fn can_transform(&self, _raw: &RawEvent) -> bool {
        true
    }

    async fn transform(&self, raw: &RawEvent) -> Result<Option<TransformedEvent>, IndexerError> {
        if let Ok(mut envelope) = serde_json::from_str::<EventEnvelope>(&raw.raw_payload) {
            if let Some(correlation_id) = raw.metadata.get("correlation_id")
                && envelope.metadata.correlation_id.is_none()
            {
                envelope.metadata.correlation_id = Some(correlation_id.clone());
            }
            if let Some(causation_id) = raw.metadata.get("causation_id")
                && envelope.metadata.causation_id.is_none()
            {
                envelope.metadata.causation_id = Some(causation_id.clone());
            }
            if let Some(trace_id) = raw.metadata.get("trace_id")
                && envelope.metadata.trace_id.is_none()
            {
                envelope.metadata.trace_id = Some(trace_id.clone());
            }
            if let Some(span_id) = raw.metadata.get("span_id")
                && envelope.metadata.span_id.is_none()
            {
                envelope.metadata.span_id = Some(span_id.clone());
            }
            return Ok(Some(TransformedEvent { envelope }));
        }

        // Backward-compatible fallback: accept bare AppEvent payloads.
        if let Ok(event) = serde_json::from_str::<AppEvent>(&raw.raw_payload) {
            let mut envelope = EventEnvelope::new(event, raw.source.clone());
            if let Some(correlation_id) = raw.metadata.get("correlation_id") {
                envelope.metadata.correlation_id = Some(correlation_id.clone());
            }
            if let Some(causation_id) = raw.metadata.get("causation_id") {
                envelope.metadata.causation_id = Some(causation_id.clone());
            }
            if let Some(trace_id) = raw.metadata.get("trace_id") {
                envelope.metadata.trace_id = Some(trace_id.clone());
            }
            if let Some(span_id) = raw.metadata.get("span_id") {
                envelope.metadata.span_id = Some(span_id.clone());
            }
            return Ok(Some(TransformedEvent { envelope }));
        }

        // Last-resort placeholder for stub mode.
        let fallback = AppEvent::CounterChanged(contracts_events::CounterChanged {
            tenant_id: raw.metadata.get("tenant_id").cloned().unwrap_or_default(),
            counter_key: "default".to_string(),
            operation: "unknown".to_string(),
            new_value: 0,
            delta: 0,
            version: 0,
        });
        Ok(Some(TransformedEvent {
            envelope: EventEnvelope::new(fallback, raw.source.clone()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn passthrough_prefers_event_envelope_payload() {
        let raw = RawEvent {
            source: "test-source".to_string(),
            raw_payload: serde_json::to_string(&EventEnvelope::new(
                AppEvent::CounterChanged(contracts_events::CounterChanged {
                    tenant_id: "tenant-a".to_string(),
                    counter_key: "counter-a".to_string(),
                    operation: "increment".to_string(),
                    new_value: 1,
                    delta: 1,
                    version: 1,
                }),
                "counter-service",
            ))
            .unwrap(),
            timestamp: "now".to_string(),
            metadata: HashMap::new(),
        };

        let transformed = PassthroughTransform
            .transform(&raw)
            .await
            .unwrap()
            .expect("expected transformed event");

        assert_eq!(transformed.envelope.source_service, "counter-service");
        assert_eq!(transformed.envelope.metadata.event_type, "counter.changed");
    }

    #[tokio::test]
    async fn passthrough_falls_back_to_bare_app_event() {
        let raw = RawEvent {
            source: "legacy-source".to_string(),
            raw_payload: serde_json::to_string(&AppEvent::CounterChanged(
                contracts_events::CounterChanged {
                    tenant_id: "tenant-a".to_string(),
                    counter_key: "counter-a".to_string(),
                    operation: "increment".to_string(),
                    new_value: 1,
                    delta: 1,
                    version: 1,
                },
            ))
            .unwrap(),
            timestamp: "now".to_string(),
            metadata: HashMap::from([("correlation_id".to_string(), "req-legacy-1".to_string())]),
        };

        let transformed = PassthroughTransform
            .transform(&raw)
            .await
            .unwrap()
            .expect("expected transformed event");

        assert_eq!(transformed.envelope.source_service, "legacy-source");
        assert_eq!(
            transformed.envelope.metadata.correlation_id.as_deref(),
            Some("req-legacy-1")
        );
    }
}
