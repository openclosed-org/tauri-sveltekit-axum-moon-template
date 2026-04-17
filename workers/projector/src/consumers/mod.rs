//! Event consumers — consume events from the event bus and build read models.

use async_trait::async_trait;
use contracts_events::AppEvent;
use event_bus::ports::EventEnvelope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::debug;

use crate::ProjectorError;

/// Abstract event consumer for the projector.
#[async_trait]
pub trait EventConsumer: Send + Sync {
    /// Name of this consumer.
    fn name(&self) -> &str;

    /// Check if this consumer is interested in the event.
    fn is_interested(&self, event: &AppEvent) -> bool;

    /// Process the event and produce an optional read model update.
    async fn consume(&self, envelope: &EventEnvelope) -> Result<Option<String>, ProjectorError>;
}

/// Stub consumer for testing.
pub struct LoggingConsumer;

#[async_trait]
impl EventConsumer for LoggingConsumer {
    fn name(&self) -> &str {
        "logging"
    }

    fn is_interested(&self, _event: &AppEvent) -> bool {
        true
    }

    async fn consume(&self, envelope: &EventEnvelope) -> Result<Option<String>, ProjectorError> {
        debug!(
            source_service = %envelope.source_service,
            event_type = %envelope.metadata.event_type,
            correlation_id = ?envelope.metadata.correlation_id,
            trace_id = ?envelope.metadata.trace_id,
            span_id = ?envelope.metadata.span_id,
            "observed event in logging consumer"
        );
        Ok(None)
    }
}

/// Counter state read model update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterStateUpdate {
    pub tenant_id: String,
    pub counter_key: String,
    pub new_value: i64,
    pub version: u64,
    pub operation: String,
    pub projected_at: String,
}

/// Projects CounterChanged events into a counter state read model.
pub struct CounterStateConsumer {
    state: RwLock<HashMap<String, CounterStateUpdate>>,
}

impl CounterStateConsumer {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(HashMap::new()),
        }
    }

    /// Build a composite key for the counter.
    fn counter_key(tenant_id: &str, counter_key: &str) -> String {
        format!("{}:{}", tenant_id, counter_key)
    }
}

#[async_trait]
impl EventConsumer for CounterStateConsumer {
    fn name(&self) -> &str {
        "counter-state"
    }

    fn is_interested(&self, event: &AppEvent) -> bool {
        matches!(event, AppEvent::CounterChanged(_))
    }

    async fn consume(&self, envelope: &EventEnvelope) -> Result<Option<String>, ProjectorError> {
        let counter_changed = match &envelope.event {
            AppEvent::CounterChanged(event) => event,
            _ => return Ok(None),
        };

        let key = Self::counter_key(&counter_changed.tenant_id, &counter_changed.counter_key);

        let update = CounterStateUpdate {
            tenant_id: counter_changed.tenant_id.clone(),
            counter_key: counter_changed.counter_key.clone(),
            new_value: counter_changed.new_value,
            version: counter_changed.version as u64,
            operation: counter_changed.operation.clone(),
            projected_at: chrono::Utc::now().to_rfc3339(),
        };

        // Update in-memory state
        {
            let mut state = self.state.write().unwrap();
            state.insert(key.clone(), update.clone());
        }

        debug!(
            tenant_id = %counter_changed.tenant_id,
            counter_key = %counter_changed.counter_key,
            new_value = counter_changed.new_value,
            version = counter_changed.version,
            correlation_id = ?envelope.metadata.correlation_id,
            trace_id = ?envelope.metadata.trace_id,
            span_id = ?envelope.metadata.span_id,
            "projected counter state"
        );

        // Serialize the update for the read model
        serde_json::to_string(&update)
            .map(Some)
            .map_err(|e| ProjectorError::Internal(format!("serialize counter state: {e}")))
    }
}
