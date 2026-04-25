//! Idempotency store — ensures operations are executed exactly once.

use async_trait::async_trait;

/// Idempotency status for a given key.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyStatus {
    /// The key has not been seen before.
    Unknown,
    /// The operation is currently being processed.
    InProgress,
    /// The operation has been successfully completed.
    Completed,
    /// The operation failed with an error.
    Failed(String),
}

/// Error type for idempotency operations.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyStoreError {
    #[error("Failed to check idempotency: {0}")]
    CheckFailed(String),

    #[error("Failed to update idempotency: {0}")]
    UpdateFailed(String),
}

/// Idempotency store trait — tracks operation execution status.
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// Check the status of an idempotency key.
    async fn check(&self, key: &str) -> Result<IdempotencyStatus, IdempotencyStoreError>;

    /// Mark an idempotency key as in progress.
    /// Returns false if the key is already in progress or completed.
    async fn start(&self, key: &str) -> Result<bool, IdempotencyStoreError>;

    /// Mark an idempotency key as successfully completed.
    async fn complete(&self, key: &str) -> Result<(), IdempotencyStoreError>;

    /// Mark an idempotency key as failed with an error message.
    async fn fail(&self, key: &str, error: String) -> Result<(), IdempotencyStoreError>;

    /// Check if a key has already been processed (completed).
    async fn is_already_processed(&self, key: &str) -> Result<bool, IdempotencyStoreError>;
}
