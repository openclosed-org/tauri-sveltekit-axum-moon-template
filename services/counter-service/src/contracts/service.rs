//! Counter service trait — moved from packages/features/counter

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Counter state — DTO for external consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    pub id: String,
    pub value: i64,
    pub version: i64,
    pub updated_at: String,
}

/// Counter operations trait.
#[async_trait]
pub trait CounterService: Send + Sync {
    async fn get_value(&self) -> Result<i64, CounterError>;
    /// Increment with optional idempotency key.
    /// If the key was already processed, returns the cached result.
    async fn increment(&self, idempotency_key: Option<&str>) -> Result<i64, CounterError>;
    async fn decrement(&self, idempotency_key: Option<&str>) -> Result<i64, CounterError>;
    async fn reset(&self, idempotency_key: Option<&str>) -> Result<i64, CounterError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CounterError {
    #[error("Database error: {0}")]
    Database(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Counter not found")]
    NotFound,
}
