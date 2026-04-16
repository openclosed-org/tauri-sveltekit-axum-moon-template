//! EventBus trait — publish, subscribe, and consume domain events.
//!
//! ## Design principles
//! - Events are fire-and-forget by default (no guaranteed delivery)
//! - Use the Outbox pattern (see `outbox/`) for guaranteed delivery
//! - All events carry a unique ID, timestamp, and type discriminator

use async_trait::async_trait;
pub use contracts_events::{EventEnvelope, EventId};

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
