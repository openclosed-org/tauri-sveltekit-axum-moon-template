//! CounterService implementation backed by CounterRepository.
//!
//! This is the **only** place that implements `feature_counter::CounterService`
//! for the repository-backed path.

use async_trait::async_trait;
use chrono::Utc;
use feature_counter::{CounterError, CounterService};
use tracing::debug;

use crate::domain::{Counter, CounterId};
use crate::ports::CounterRepository;

/// SQL migration for the counter table.
///
/// This is exported so the composition root (server binary) can
/// run migrations at startup without depending on a specific adapter.
pub const COUNTER_MIGRATION: &str = "CREATE TABLE IF NOT EXISTS counter (\
        tenant_id TEXT PRIMARY KEY,\
        value INTEGER NOT NULL DEFAULT 0,\
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))\
    )";

/// `CounterService` backed by any `CounterRepository` implementation.
///
/// ## Type parameters
/// - `R`: The repository implementation. Allows swapping libsql,
///   Turso cloud, SurrealDB, or in-memory stubs without touching this code.
pub struct RepositoryBackedCounterService<R: CounterRepository> {
    repo: R,
}

impl<R: CounterRepository> RepositoryBackedCounterService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: CounterRepository> CounterService for RepositoryBackedCounterService<R> {
    async fn get_value(&self) -> Result<i64, CounterError> {
        let id = CounterId::new("default");
        let counter = self.repo.load(&id).await.map_err(CounterError::Database)?;

        let value = counter.map(|c| c.value).unwrap_or(0);
        debug!(counter_id = %id, value, "counter.get_value");
        Ok(value)
    }

    async fn increment(&self) -> Result<i64, CounterError> {
        let id = CounterId::new("default");
        let now = Utc::now();
        let value = self
            .repo
            .increment(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(counter_id = %id, value, "counter.increment");
        Ok(value)
    }

    async fn decrement(&self) -> Result<i64, CounterError> {
        let id = CounterId::new("default");
        let now = Utc::now();
        let value = self
            .repo
            .decrement(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(counter_id = %id, value, "counter.decrement");
        Ok(value)
    }

    async fn reset(&self) -> Result<i64, CounterError> {
        let id = CounterId::new("default");
        let now = Utc::now();
        self.repo
            .reset(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(counter_id = %id, "counter.reset");
        Ok(0)
    }
}

/// Tenant-scoped CounterService — accepts TenantId, isolates by tenant.
///
/// This is the **primary** service used by the BFF layer.
/// Each tenant gets an independent counter with no cross-tenant leakage.
pub struct TenantScopedCounterService<R: CounterRepository> {
    repo: R,
}

impl<R: CounterRepository> TenantScopedCounterService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Get counter value for a specific tenant.
    pub async fn get_value(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = CounterId::new(tenant_id.as_str());
        let counter = self.repo.load(&id).await.map_err(CounterError::Database)?;

        let value = counter.map(|c| c.value).unwrap_or(0);
        debug!(tenant_id = %tenant_id, value, "counter.get_value_for_tenant");
        Ok(value)
    }

    /// Increment counter for a specific tenant.
    pub async fn increment(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();
        let value = self
            .repo
            .increment(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(tenant_id = %tenant_id, value, "counter.increment_for_tenant");
        Ok(value)
    }

    /// Decrement counter for a specific tenant.
    pub async fn decrement(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();
        let value = self
            .repo
            .decrement(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(tenant_id = %tenant_id, value, "counter.decrement_for_tenant");
        Ok(value)
    }

    /// Reset counter for a specific tenant.
    pub async fn reset(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();
        self.repo
            .reset(&id, now)
            .await
            .map_err(CounterError::Database)?;

        debug!(tenant_id = %tenant_id, "counter.reset_for_tenant");
        Ok(0)
    }
}
