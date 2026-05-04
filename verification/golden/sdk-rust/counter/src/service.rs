//! CounterService trait — the sole interface app shells depend on.

use async_trait::async_trait;

use super::types::{CounterError, CounterId};

/// Counter operations trait — implemented by all SDK backends.
///
/// App shells receive a `&dyn CounterService` and never know
/// whether the backend is embedded Turso, HTTP, or a mock.
#[async_trait]
pub trait CounterService: Send + Sync {
    /// Get the current counter value.
    async fn get_value(&self, counter_id: &CounterId) -> Result<i64, CounterError>;

    /// Increment with optional idempotency key.
    async fn increment(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;

    /// Decrement with optional idempotency key.
    async fn decrement(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;

    /// Reset counter to zero with optional idempotency key.
    async fn reset(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;
}
