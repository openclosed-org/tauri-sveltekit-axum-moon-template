//! Counter service trait — moved from packages/features/counter
//!
//! ## Boundary Mapping
//! - Domain `Counter` (from `domain::entity`) is the aggregate root with strong typing.
//! - This module re-exports `CounterResponse` from `packages/contracts/api` for external DTOs.
//! - The service trait operates on domain types internally and maps to contract DTOs at the boundary.

use async_trait::async_trait;
pub use contracts_api::CounterResponse;
use contracts_events::ActorRef;

use crate::domain::CounterId;

/// Request-scoped command context propagated into event metadata.
#[derive(Debug, Clone, Default)]
pub struct CounterCommandContext {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub actor: Option<ActorRef>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Counter operations trait.
#[async_trait]
pub trait CounterService: Send + Sync {
    async fn get_value(&self, tenant_id: &CounterId) -> Result<i64, CounterError>;
    /// Increment with optional idempotency key.
    /// If the key was already processed, returns the cached result.
    async fn increment(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;
    async fn increment_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        _context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        self.increment(tenant_id, idempotency_key).await
    }
    async fn decrement(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;
    async fn decrement_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        _context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        self.decrement(tenant_id, idempotency_key).await
    }
    async fn reset(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError>;
    async fn reset_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        _context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        self.reset(tenant_id, idempotency_key).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CounterError {
    #[error("Database error: {0}")]
    Database(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Counter not found: {0}")]
    NotFound(String),
    #[error("CAS conflict: counter was modified by another writer")]
    CasConflict,
    /// CAS conflict with version details — used for detailed error responses.
    #[error("CAS conflict: expected version {expected}, actual {actual}")]
    CasConflictWithDetails { expected: i64, actual: i64 },
    #[error("Idempotency key was reused for a different counter command")]
    IdempotencyConflict,
}
