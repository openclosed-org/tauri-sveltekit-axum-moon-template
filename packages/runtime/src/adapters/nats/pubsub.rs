//! NATS-backed runtime pub/sub adapter.

use std::collections::HashMap;

use async_nats::Client;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::ports::{MessageEnvelope, MessageHandler, PubSub, PubSubError, SubscriptionId};
use contracts_events::NATS_OUTBOX_TOPIC_PREFIX;

/// Pub/Sub adapter backed by NATS subjects.
pub struct NatsPubSub {
    client: Client,
    subject_prefix: String,
    handlers: RwLock<HashMap<String, (String, MessageHandler)>>,
}

impl NatsPubSub {
    pub async fn connect(nats_url: &str, subject_prefix: &str) -> Result<Self, PubSubError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| PubSubError::ConnectionError(format!("connect nats: {e}")))?;

        Ok(Self {
            client,
            subject_prefix: normalize_subject_prefix(subject_prefix),
            handlers: RwLock::new(HashMap::new()),
        })
    }

    fn subject_for_topic(&self, topic: &str) -> String {
        let topic = topic.trim_matches('.');
        if self.subject_prefix.is_empty() || topic.starts_with(&self.subject_prefix) {
            topic.to_string()
        } else if topic.is_empty() {
            self.subject_prefix.clone()
        } else {
            format!("{}.{}", self.subject_prefix, topic)
        }
    }
}

#[async_trait]
impl PubSub for NatsPubSub {
    async fn publish(&self, topic: &str, envelope: MessageEnvelope) -> Result<(), PubSubError> {
        let subject = self.subject_for_topic(topic);
        let payload = serde_json::to_vec(&envelope)
            .map_err(|e| PubSubError::PublishFailed(format!("serialize message: {e}")))?;

        self.client
            .publish(subject.clone(), payload.into())
            .await
            .map_err(|e| PubSubError::PublishFailed(format!("publish to '{subject}': {e}")))?;

        debug!(subject = %subject, message_id = %envelope.message_id, "published message to nats");

        let handlers = self.handlers.read().await;
        for (subscription_id, (pattern, handler)) in handlers.iter() {
            if pattern_matches(pattern, topic) {
                handler(envelope.clone());
                debug!(subscription_id, "handler dispatched");
            }
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        topic_pattern: &str,
        handler: MessageHandler,
    ) -> Result<SubscriptionId, PubSubError> {
        let subscription_id = SubscriptionId::new();
        let mut handlers = self.handlers.write().await;
        handlers.insert(
            subscription_id.0.clone(),
            (topic_pattern.to_string(), handler),
        );
        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), PubSubError> {
        let mut handlers = self.handlers.write().await;
        if handlers.remove(&subscription_id.0).is_none() {
            warn!(subscription_id = %subscription_id.0, "nats pubsub unsubscribe requested for missing handler");
        }
        Ok(())
    }
}

fn normalize_subject_prefix(subject_prefix: &str) -> String {
    subject_prefix.trim_matches('.').to_string()
}

fn pattern_matches(pattern: &str, topic: &str) -> bool {
    if pattern == topic {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix("*") {
        return topic.starts_with(prefix);
    }

    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    let topic_parts: Vec<&str> = topic.split('.').collect();

    if pattern_parts.len() != topic_parts.len() {
        return false;
    }

    pattern_parts
        .iter()
        .zip(topic_parts.iter())
        .all(|(p, t)| *p == "*" || *p == *t)
}

#[cfg(test)]
mod tests {
    use super::{normalize_subject_prefix, pattern_matches};
    use contracts_events::NATS_OUTBOX_TOPIC_PREFIX;

    #[test]
    fn trims_surrounding_dots() {
        assert_eq!(normalize_subject_prefix("outbox"), "outbox");
        assert_eq!(normalize_subject_prefix(".outbox."), "outbox");
    }

    #[test]
    fn wildcard_pattern_matches_single_topic_segment() {
        assert!(pattern_matches("outbox.*", "outbox.counter.changed"));
        assert!(!pattern_matches("outbox.*", "indexer.counter.changed"));
    }

    #[test]
    fn avoids_double_prefixing_subjects() {
        let subject_prefix = normalize_subject_prefix("counter");
        let topic = "counter.changed";
        let subject = if subject_prefix.is_empty() || topic.starts_with(&subject_prefix) {
            topic.to_string()
        } else {
            format!("{}.{}", subject_prefix, topic)
        };

        assert_eq!(subject, "counter.changed");
    }

    #[test]
    fn wildcard_matches_canonical_outbox_topics() {
        assert!(pattern_matches(
            &format!("{}.*", NATS_OUTBOX_TOPIC_PREFIX),
            "outbox.counter.changed"
        ));
    }
}
