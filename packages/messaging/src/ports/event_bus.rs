//! EventBus trait — publish, subscribe, and consume domain events.
//!
//! ## Design principles
//! - Events are fire-and-forget by default (no guaranteed delivery)
//! - Use the Outbox pattern (see `outbox/`) for guaranteed delivery
//! - All events carry a unique ID, timestamp, and type discriminator

use async_trait::async_trait;
use contracts_events::AppEvent;

/// Error type for event bus operations.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Failed to subscribe: {0}")]
    SubscribeFailed(String),

    #[error("Event handler error: {0}")]
    HandlerError(String),
}

/// Unique identifier for an event envelope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub uuid::Uuid);

impl EventId {
    pub fn new_v7() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A typed event envelope wrapping the event payload.
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub id: EventId,
    pub event: AppEvent,
    pub source_service: String,
    /// Correlation ID for tracing across service boundaries.
    pub correlation_id: Option<String>,
}

impl EventEnvelope {
    pub fn new(event: AppEvent, source_service: impl Into<String>) -> Self {
        Self {
            id: EventId::new_v7(),
            event,
            source_service: source_service.into(),
            correlation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Type alias for event handlers.
pub type EventHandler = Box<dyn Fn(EventEnvelope) + Send + Sync>;

/// EventBus trait — the abstraction all services depend on.
///
/// ## Usage
/// ```ignore
/// // Publishing an event
/// event_bus.publish(EventEnvelope::new(
///     AppEvent::CounterChanged(CounterChanged { ... }),
///     "counter-service",
/// )).await?;
///
/// // Subscribing to all events
/// event_bus.subscribe("my-service", Box::new(|envelope| {
///     tracing::info!("Received event: {:?}", envelope.event);
/// })).await?;
/// ```
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to the bus.
    ///
    /// This is **fire-and-forget** — the event may be dropped
    /// if no subscriber is listening. Use the Outbox pattern
    /// (`outbox/`) for guaranteed delivery.
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventBusError>;

    /// Subscribe to all events with a handler callback.
    ///
    /// The handler is invoked synchronously for each published event.
    /// If the handler panics, the error is logged but does not block
    /// other handlers or future events.
    async fn subscribe(
        &self,
        subscriber_id: &str,
        handler: EventHandler,
    ) -> Result<(), EventBusError>;

    /// Unsubscribe a previously registered handler.
    async fn unsubscribe(&self, subscriber_id: &str) -> Result<(), EventBusError>;
}
