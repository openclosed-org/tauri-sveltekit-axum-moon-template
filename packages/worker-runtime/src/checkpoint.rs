//! Checkpoint store — tracks the last processed sequence number.
//!
//! Workers use this to resume from where they left off after restart or crash.

use async_trait::async_trait;

/// Error type for checkpoint operations.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointStoreError {
    #[error("Failed to load checkpoint: {0}")]
    LoadFailed(String),

    #[error("Failed to persist checkpoint: {0}")]
    PersistFailed(String),
}

/// Checkpoint store trait — abstracts checkpoint persistence.
#[async_trait]
pub trait CheckpointStore: Send + Sync {
    /// Get the current checkpoint value.
    async fn get(&self) -> Result<u64, CheckpointStoreError>;

    /// Advance the checkpoint to a new value.
    async fn advance(&self, new_value: u64) -> Result<(), CheckpointStoreError>;
}
