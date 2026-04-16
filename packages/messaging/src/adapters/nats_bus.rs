//! NATS-backed event bus adapter.

use std::collections::HashMap;

use async_nats::Client;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::ports::{EventBus, EventBusError, EventEnvelope, EventHandler};
use contracts_events::{NATS_EVENT_SUBJECT_PREFIX, event_type_name};

/// Event bus backed by NATS subjects.
pub struct NatsEventBus {
    client: Client,
    subject_prefix: String,
    subscribers: RwLock<HashMap<String, EventHandler>>,
}

impl NatsEventBus {
    pub async fn connect(nats_url: &str, subject_prefix: &str) -> Result<Self, EventBusError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| EventBusError::PublishFailed(format!("connect nats: {e}")))?;

        Ok(Self {
            client,
            subject_prefix: normalize_subject_prefix(subject_prefix),
            subscribers: RwLock::new(HashMap::new()),
        })
    }

    fn subject_for_envelope(&self, envelope: &EventEnvelope) -> String {
        let suffix = event_type_name(&envelope.event);
        if self.subject_prefix.is_empty()
            || suffix.starts_with(&format!("{}.", self.subject_prefix))
        {
            suffix.to_string()
        } else {
            format!("{}.{}", self.subject_prefix, suffix)
        }
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventBusError> {
        let subject = self.subject_for_envelope(&envelope);
        let payload = serde_json::to_vec(&envelope)
            .map_err(|e| EventBusError::PublishFailed(format!("serialize event: {e}")))?;

        self.client
            .publish(subject.clone(), payload.into())
            .await
            .map_err(|e| EventBusError::PublishFailed(format!("publish to '{subject}': {e}")))?;

        debug!(subject = %subject, event_id = %envelope.id, "published event to nats");

        let subscribers = self.subscribers.read().await;
        for (subscriber_id, handler) in subscribers.iter() {
            handler(envelope.clone());
            debug!(subscriber_id = %subscriber_id, "handler dispatched");
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        subscriber_id: &str,
        handler: EventHandler,
    ) -> Result<(), EventBusError> {
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(subscriber_id.to_string(), handler);
        Ok(())
    }

    async fn unsubscribe(&self, subscriber_id: &str) -> Result<(), EventBusError> {
        let mut subscribers = self.subscribers.write().await;
        if subscribers.remove(subscriber_id).is_none() {
            warn!(
                subscriber_id,
                "nats event bus unsubscribe requested for missing subscriber"
            );
        }
        Ok(())
    }
}

fn normalize_subject_prefix(subject_prefix: &str) -> String {
    subject_prefix.trim_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_subject_prefix;
    use contracts_events::NATS_EVENT_SUBJECT_PREFIX;

    #[test]
    fn trims_surrounding_dots() {
        assert_eq!(normalize_subject_prefix("counter"), "counter");
        assert_eq!(normalize_subject_prefix(".counter."), "counter");
        assert_eq!(normalize_subject_prefix(""), "");
    }

    #[test]
    fn keeps_canonical_event_prefix() {
        assert_eq!(
            normalize_subject_prefix(NATS_EVENT_SUBJECT_PREFIX),
            "events"
        );
    }
}
