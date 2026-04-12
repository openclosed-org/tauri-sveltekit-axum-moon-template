//! Queue port — persistent message queue abstraction.
//!
//! This port defines the interface for enqueuing, dequeuing, and managing
//! messages in persistent queues, whether via NATS JetStream, RabbitMQ,
//! AWS SQS, or in-memory (for testing).
//!
//! ## Design principles
//! - Queues provide at-least-once delivery semantics
//! - Messages have visibility timeout to prevent duplicate processing
//! - Dead letter queues handle messages that fail repeatedly

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Error types for queue operations.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Failed to enqueue message: {0}")]
    EnqueueFailed(String),

    #[error("Failed to dequeue message: {0}")]
    DequeueFailed(String),

    #[error("Failed to acknowledge message: {0}")]
    AckFailed(String),

    #[error("Message processing timed out after {0:?}")]
    Timeout(Duration),

    #[error("Queue not found: {0}")]
    NotFound(String),

    #[error("Message expired in queue")]
    Expired,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// A message in the queue.
#[derive(Debug, Clone, Serialize)]
pub struct QueueMessage<Body = serde_json::Value> {
    /// Unique message identifier.
    pub message_id: String,
    /// Message body/payload.
    pub body: Body,
    /// Queue name this message belongs to.
    pub queue_name: String,
    /// Number of times this message has been dequeued.
    pub dequeue_count: u32,
    /// Timestamp when the message was enqueued (RFC3339).
    pub enqueued_at: String,
    /// Optional delay before the message becomes visible.
    pub visible_after: Option<String>,
    /// Optional time-to-live for the message.
    pub expires_at: Option<String>,
}

impl<Body> QueueMessage<Body> {
    pub fn new(queue_name: impl Into<String>, body: Body) -> Self {
        Self {
            message_id: uuid::Uuid::now_v7().to_string(),
            body,
            queue_name: queue_name.into(),
            dequeue_count: 0,
            enqueued_at: chrono::Utc::now().to_rfc3339(),
            visible_after: None,
            expires_at: None,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.visible_after = Some(
            (chrono::Utc::now() + delay).to_rfc3339(),
        );
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.expires_at = Some(
            (chrono::Utc::now() + ttl).to_rfc3339(),
        );
        self
    }
}

/// The Queue port — persistent message queue abstraction.
///
/// ## Usage
/// ```ignore
/// // Enqueue a message
/// queue.enqueue("task-queue", QueueMessage::new(
///     "task-queue",
///     IndexTask { resource_id: "123" },
/// )).await?;
///
/// // Dequeue with visibility timeout
/// let message = queue.dequeue("task-queue", Duration::from_secs(30)).await?;
///
/// // Process the message...
///
/// // Acknowledge successful processing
/// queue.ack(&message.message_id).await?;
/// ```
#[async_trait]
pub trait Queue: Send + Sync {
    /// Enqueue a message to a specific queue.
    ///
    /// Returns the message ID for tracking.
    async fn enqueue<Body: Serialize + Send>(
        &self,
        queue_name: &str,
        message: QueueMessage<Body>,
    ) -> Result<String, QueueError>;

    /// Dequeue a message from a queue with visibility timeout.
    ///
    /// The message becomes invisible to other consumers for the visibility timeout duration.
    /// If not acknowledged within this time, it will become visible again.
    async fn dequeue<Body: DeserializeOwned + Send>(
        &self,
        queue_name: &str,
        visibility_timeout: Duration,
    ) -> Result<Option<QueueMessage<Body>>, QueueError>;

    /// Acknowledge successful processing of a message.
    ///
    /// This removes the message from the queue permanently.
    async fn ack(&self, message_id: &str) -> Result<(), QueueError>;

    /// Reject a message and optionally send it to the dead letter queue.
    ///
    /// If `requeue` is true, the message becomes visible again for reprocessing.
    /// If false, it's sent to the dead letter queue after max retries.
    async fn nack(&self, message_id: &str, requeue: bool) -> Result<(), QueueError>;

    /// Get the approximate number of messages in a queue.
    async fn queue_depth(&self, queue_name: &str) -> Result<u64, QueueError>;
}
