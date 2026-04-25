//! Deduplication store — prevents double-processing of messages.

use async_trait::async_trait;

/// Error type for deduplication operations.
#[derive(Debug, thiserror::Error)]
pub enum DedupeStoreError {
    #[error("Failed to check duplicate: {0}")]
    CheckFailed(String),

    #[error("Failed to mark processed: {0}")]
    MarkFailed(String),
}

/// Deduplication store trait — tracks recently processed message IDs.
#[async_trait]
pub trait DedupeStore: Send + Sync {
    /// Check if a message ID has already been processed.
    async fn is_duplicate(&self, id: &str) -> Result<bool, DedupeStoreError>;

    /// Mark a message ID as processed.
    /// Returns true if this is the first time we've seen this ID.
    async fn mark_processed(&self, id: &str) -> Result<bool, DedupeStoreError>;
}
