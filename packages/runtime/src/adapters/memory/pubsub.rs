//! In-memory pub/sub adapter.
//!
//! Uses tokio broadcast channels for message distribution.
//! Subscribers receive messages in real-time; no persistence.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};
use tracing::debug;

use crate::ports::{MessageEnvelope, MessageHandler, PubSub, PubSubError, SubscriptionId};

/// In-memory pub/sub adapter for testing.
///
/// Messages are broadcast to all subscribers immediately.
/// No persistence — messages are lost if no subscriber is listening.
pub struct MemoryPubSub {
    sender: broadcast::Sender<MessageEnvelope>,
    handlers: RwLock<HashMap<String, (String, MessageHandler)>>,
}

impl MemoryPubSub {
    pub fn new() -> Self {
        let (sender, _rx) = broadcast::channel(1024);
        Self {
            sender,
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub fn subscribe_receiver(&self) -> broadcast::Receiver<MessageEnvelope> {
        self.sender.subscribe()
    }
}

impl Default for MemoryPubSub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PubSub for MemoryPubSub {
    async fn publish(&self, _topic: &str, envelope: MessageEnvelope) -> Result<(), PubSubError> {
        debug!(
            message_id = %envelope.message_id,
            topic = %envelope.topic,
            "publishing message",
        );

        if let Err(e) = self.sender.send(envelope.clone()) {
            tracing::warn!(error = %e, "broadcast send failed (likely all receivers dropped)");
        }

        // Also dispatch to registered handlers
        let handlers = self.handlers.read().await;
        for (id, (pattern, handler)) in handlers.iter() {
            // Simple wildcard matching: "events.*" matches "events.counter"
            let matches = pattern_matches(pattern, &envelope.topic);
            if matches {
                handler(envelope.clone());
                debug!(subscription_id = %id, "handler dispatched");
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
        debug!(subscription_id = %subscription_id.0, pattern = %topic_pattern, "subscription registered");
        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), PubSubError> {
        let mut handlers = self.handlers.write().await;
        handlers.remove(&subscription_id.0);
        Ok(())
    }
}

/// Simple pattern matching for topic subscriptions.
/// Supports `*` as a whole-suffix wildcard for multi-segment topics.
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
