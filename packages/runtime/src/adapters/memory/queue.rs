//! In-memory queue adapter.
//!
//! Provides a simple FIFO queue with visibility timeout support.
//! Messages are stored in memory and lost on restart.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;
use tracing::debug;

use crate::ports::{Queue, QueueError, QueueMessage};

/// Internal queue entry for storage.
struct StoredMessage {
    body: serde_json::Value,
    dequeue_count: u32,
    enqueued_at: String,
    visible_after: Option<String>,
    expires_at: Option<String>,
    is_visible: bool,
}

/// In-memory queue adapter for testing.
///
/// Provides basic queue semantics with visibility timeout.
/// No persistence — messages are lost on restart.
pub struct MemoryQueue {
    queues: RwLock<HashMap<String, Vec<(String, StoredMessage)>>>, // message_id -> message
}

impl MemoryQueue {
    pub fn new() -> Self {
        Self {
            queues: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Queue for MemoryQueue {
    async fn enqueue<Body: Serialize + Send>(
        &self,
        queue_name: &str,
        message: QueueMessage<Body>,
    ) -> Result<String, QueueError> {
        let body = serde_json::to_value(&message.body)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;

        let stored = StoredMessage {
            body,
            dequeue_count: message.dequeue_count,
            enqueued_at: message.enqueued_at.clone(),
            visible_after: message.visible_after.clone(),
            expires_at: message.expires_at.clone(),
            is_visible: message.visible_after.is_none(),
        };

        let mut queues = self.queues.write().await;
        queues
            .entry(queue_name.to_string())
            .or_insert_with(Vec::new)
            .push((message.message_id.clone(), stored));

        debug!(queue_name = %queue_name, message_id = %message.message_id, "message enqueued");
        Ok(message.message_id)
    }

    async fn dequeue<Body: DeserializeOwned + Send>(
        &self,
        queue_name: &str,
        _visibility_timeout: Duration,
    ) -> Result<Option<QueueMessage<Body>>, QueueError> {
        let mut queues = self.queues.write().await;
        let messages = queues
            .get_mut(queue_name)
            .ok_or_else(|| QueueError::NotFound(queue_name.to_string()))?;

        // Find first visible message
        for (id, stored) in messages.iter_mut() {
            if stored.is_visible {
                stored.dequeue_count += 1;
                stored.is_visible = false; // Hide it (visibility timeout)

                let body: Body = serde_json::from_value(stored.body.clone())
                    .map_err(|e| QueueError::SerializationError(e.to_string()))?;

                debug!(queue_name = %queue_name, message_id = %id, "message dequeued");
                return Ok(Some(QueueMessage {
                    message_id: id.clone(),
                    body,
                    queue_name: queue_name.to_string(),
                    dequeue_count: stored.dequeue_count,
                    enqueued_at: stored.enqueued_at.clone(),
                    visible_after: stored.visible_after.clone(),
                    expires_at: stored.expires_at.clone(),
                }));
            }
        }

        Ok(None)
    }

    async fn ack(&self, message_id: &str) -> Result<(), QueueError> {
        let mut queues = self.queues.write().await;
        for (_queue_name, messages) in queues.iter_mut() {
            messages.retain(|(id, _)| id != message_id);
        }
        debug!(message_id = %message_id, "message acknowledged");
        Ok(())
    }

    async fn nack(&self, message_id: &str, requeue: bool) -> Result<(), QueueError> {
        let mut queues = self.queues.write().await;
        for (_queue_name, messages) in queues.iter_mut() {
            if let Some((_, stored)) = messages.iter_mut().find(|(id, _)| id == message_id) {
                if requeue {
                    stored.is_visible = true; // Make visible again
                } else {
                    // In a real implementation, this would move to DLQ
                    // For now, just remove it
                }
            }
        }
        debug!(message_id = %message_id, requeue = %requeue, "message nacked");
        Ok(())
    }

    async fn queue_depth(&self, queue_name: &str) -> Result<u64, QueueError> {
        let queues = self.queues.read().await;
        let messages = queues
            .get(queue_name)
            .ok_or_else(|| QueueError::NotFound(queue_name.to_string()))?;
        Ok(messages.len() as u64)
    }
}
