//! SDK Counter Embedded — offline-first counter backend for Tauri desktop.
//!
//! This crate wraps `counter-service` internals behind the `sdk_counter::CounterService`
//! trait, so app shells never import `counter-service` directly.
//!
//! ## Usage
//! ```ignore
//! use sdk_counter_embedded::EmbeddedCounterClient;
//! use sdk_counter::CounterService;
//!
//! let client = EmbeddedCounterClient::new(db).await?;
//! let value = client.increment(&CounterId::new("tenant-1"), None).await?;
//! ```

use async_trait::async_trait;
use counter_service::application::TenantScopedCounterService;
use counter_service::infrastructure::LibSqlCounterRepository;
use sdk_counter::{CounterError, CounterId, CounterService};
use storage_turso::EmbeddedTurso;

/// Embedded counter client — bridges counter-service to the SDK trait.
///
/// Internally holds a `TenantScopedCounterService<LibSqlCounterRepository<EmbeddedTurso>>`
/// and maps `CounterId` ↔ `kernel::TenantId` at the boundary.
pub struct EmbeddedCounterClient {
    inner: TenantScopedCounterService<LibSqlCounterRepository<EmbeddedTurso>>,
}

impl EmbeddedCounterClient {
    pub async fn new(db: EmbeddedTurso) -> Result<Self, String> {
        let repo = LibSqlCounterRepository::new(db);
        repo.migrate().await.map_err(|e| e.to_string())?;
        Ok(Self {
            inner: TenantScopedCounterService::new(repo),
        })
    }
}

#[async_trait]
impl CounterService for EmbeddedCounterClient {
    async fn get_value(&self, counter_id: &CounterId) -> Result<i64, CounterError> {
        let tenant_id = kernel::TenantId(counter_id.as_str().to_string());
        self.inner
            .get_value(&tenant_id)
            .await
            .map_err(|e| CounterError::Database(e.to_string()))
    }

    async fn increment(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let tenant_id = kernel::TenantId(counter_id.as_str().to_string());
        self.inner
            .increment(&tenant_id, idempotency_key)
            .await
            .map_err(|e| CounterError::Database(e.to_string()))
    }

    async fn decrement(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let tenant_id = kernel::TenantId(counter_id.as_str().to_string());
        self.inner
            .decrement(&tenant_id, idempotency_key)
            .await
            .map_err(|e| CounterError::Database(e.to_string()))
    }

    async fn reset(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let tenant_id = kernel::TenantId(counter_id.as_str().to_string());
        self.inner
            .reset(&tenant_id, idempotency_key)
            .await
            .map_err(|e| CounterError::Database(e.to_string()))
    }
}
