//! PubSub port — publish/subscribe messaging abstraction.
//!
//! This port defines the interface for asynchronous event publishing and subscription,
//! whether via NATS, Redis, Dapr, or in-memory channels.
//!
//! ## Design principles
//! - Services depend on this port, NOT on concrete message brokers
//! - All events carry correlation IDs for distributed tracing
//! - Subscribers can filter by topic patterns

use async_trait::async_trait;
use contracts_events::AppEvent;
use serde::{Deserialize, Serialize};

/// Error types for pub/sub operations.
#[derive(Debug, thiserror::Error)]
pub enum PubSubError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Failed to subscribe to topic '{topic}': {reason}")]
    SubscribeFailed { topic: String, reason: String },

    #[error("Failed to unsubscribe from topic '{topic}': {reason}")]
    UnsubscribeFailed { topic: String, reason: String },

    #[error("Message handler error: {0}")]
    HandlerError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// A message envelope wrapping published events with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message identifier.
    pub message_id: String,
    /// Event payload.
    pub event: AppEvent,
    /// Topic the message was published to.
    pub topic: String,
    /// Source service identifier (e.g., "counter-service").
    pub source_service: String,
    /// Shared event metadata used to propagate correlation and ownership context.
    pub metadata: contracts_events::EventMetadata,
    /// Timestamp when the message was created (RFC3339).
    pub timestamp: String,
}

impl MessageEnvelope {
    pub fn new(
        event: AppEvent,
        topic: impl Into<String>,
        source_service: impl Into<String>,
    ) -> Self {
        let metadata = contracts_events::EventMetadata::for_event(&event);
        Self {
            message_id: uuid::Uuid::now_v7().to_string(),
            event,
            topic: topic.into(),
            source_service: source_service.into(),
            metadata,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_correlation_id(correlation_id);
        self
    }

    pub fn with_metadata(mut self, metadata: contracts_events::EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Handler callback type for message subscriptions.
pub type MessageHandler = Box<dyn Fn(MessageEnvelope) + Send + Sync>;

/// The PubSub port — publish/subscribe messaging abstraction.
///
/// ## Usage
/// ```ignore
/// // Publishing an event
/// pubsub.publish("events.counter", MessageEnvelope::new(
///     AppEvent::CounterChanged(CounterChanged { ... }),
///     "events.counter",
///     "counter-service",
/// )).await?;
///
/// // Subscribing to events
/// pubsub.subscribe("events.*", Box::new(|envelope| {
///     tracing::info!("Received event: {:?}", envelope.event);
/// })).await?;
/// ```
#[async_trait]
pub trait PubSub: Send + Sync {
    /// Publish a message to a topic.
    ///
    /// This is fire-and-forget — the message may be dropped
    /// if no subscriber is listening. Use persistent queues
    /// for guaranteed delivery.
    async fn publish(&self, topic: &str, envelope: MessageEnvelope) -> Result<(), PubSubError>;

    /// Subscribe to messages matching a topic pattern.
    ///
    /// Topic patterns support wildcards (e.g., "events.*", "events.counter.>").
    /// The handler is invoked for each matching message.
    async fn subscribe(
        &self,
        topic_pattern: &str,
        handler: MessageHandler,
    ) -> Result<SubscriptionId, PubSubError>;

    /// Unsubscribe from a previously registered subscription.
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), PubSubError>;
}

/// Unique identifier for a subscription.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub String);

impl SubscriptionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
}

impl Default for SubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}
