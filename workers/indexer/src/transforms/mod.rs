//! Event transformers — raw protocol events → business DTOs.

use async_trait::async_trait;
use contracts_events::AppEvent;

use crate::IndexerError;
use crate::sources::RawEvent;

/// Event transformer trait.
#[async_trait]
pub trait EventTransform: Send + Sync {
    /// Name of this transformer.
    fn name(&self) -> &str;

    /// Check if this transformer can handle the raw event.
    fn can_transform(&self, raw: &RawEvent) -> bool;

    /// Transform a raw event to an AppEvent.
    async fn transform(&self, raw: &RawEvent) -> Result<Option<AppEvent>, IndexerError>;
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

    async fn transform(&self, raw: &RawEvent) -> Result<Option<AppEvent>, IndexerError> {
        // For testing: try to parse the raw payload as JSON AppEvent
        match serde_json::from_str::<AppEvent>(&raw.raw_payload) {
            Ok(event) => Ok(Some(event)),
            Err(_) => {
                // Return a placeholder counter-changed event
                Ok(Some(AppEvent::CounterChanged(
                    contracts_events::CounterChanged {
                        tenant_id: raw.metadata.get("tenant_id").cloned().unwrap_or_default(),
                        counter_key: "default".to_string(),
                        operation: "unknown".to_string(),
                        new_value: 0,
                        delta: 0,
                        version: 0,
                    },
                )))
            }
        }
    }
}
