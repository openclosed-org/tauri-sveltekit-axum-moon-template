//! In-memory EventBus using tokio broadcast channels.
//!
//! Events are dispatched synchronously to all subscribers.
//! If a subscriber's handler is slow, it may block the publish path.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, warn};

use crate::ports::{EventBus, EventBusError, EventEnvelope, EventHandler};

/// Default channel capacity — max buffered events before slow subscribers lag.
const DEFAULT_CAPACITY: usize = 1024;

/// In-memory event bus using tokio broadcast channels.
///
/// ## Thread safety
/// `Arc<InMemoryEventBus>` is `Clone` and safe to share across threads.
/// The broadcast sender handles concurrency internally.
pub struct InMemoryEventBus {
    tx: broadcast::Sender<EventEnvelope>,
    subscribers: RwLock<HashMap<String, EventHandler>>,
}

impl InMemoryEventBus {
    /// Create a new in-memory event bus with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new in-memory event bus with custom channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self {
            tx,
            subscribers: RwLock::new(HashMap::new()),
        }
    }

    /// Return a receiver that can listen to events.
    /// Useful for testing or for building a custom consumer.
    pub fn subscribe_receiver(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx.subscribe()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventBusError> {
        debug!(
            event_id = %envelope.id,
            source = %envelope.source_service,
            "publishing event",
        );

        // Send via broadcast — delivers to all current receivers
        if let Err(e) = self.tx.send(envelope.clone()) {
            warn!(error = %e, "broadcast send failed (likely all receivers dropped)");
        }

        // Also dispatch to registered handlers
        let handlers = self.subscribers.read().await;
        for (id, handler) in handlers.iter() {
            // Handlers run inline — slow handlers block the publish path
            handler(envelope.clone());
            debug!(subscriber_id = %id, "handler dispatched");
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
        debug!(subscriber_id = %subscriber_id, "subscriber registered");
        Ok(())
    }

    async fn unsubscribe(&self, subscriber_id: &str) -> Result<(), EventBusError> {
        let mut subscribers = self.subscribers.write().await;
        if subscribers.remove(subscriber_id).is_some() {
            debug!(subscriber_id = %subscriber_id, "subscriber unregistered");
        }
        Ok(())
    }
}
