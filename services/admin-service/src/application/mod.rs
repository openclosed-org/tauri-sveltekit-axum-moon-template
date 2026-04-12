//! Admin service — dashboard stats aggregation.
//!
//! Admin is a composition layer: it aggregates data from tenant-service
//! and counter-service to produce `AdminDashboardStats`.

use async_trait::async_trait;
use feature_admin::{AdminDashboardStats, AdminError, AdminService};

// Trait aliases for the services this admin depends on.
// In a fully decoupled architecture these would be injected via the ports layer.
// For Phase 0 they reference the concrete service traits directly.

/// Trait alias for tenant list capability (satisfied by tenant-service).
#[async_trait]
pub trait TenantLister: Send + Sync {
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, AdminError>;
}

/// Summary row returned by tenant list.
#[derive(Debug, Clone)]
pub struct TenantSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Trait alias for counter value capability (satisfied by counter-service).
#[async_trait]
pub trait CounterReader: Send + Sync {
    async fn get_value(&self) -> Result<i64, AdminError>;
}

// ── Blanket implementations for concrete service types ──────────────────────

use tenant_service::application::TenantServiceTrait;
use tenant_service::domain::Tenant as TenantEntity;

#[async_trait]
impl<R> TenantLister for tenant_service::application::TenantService<R>
where
    R: tenant_service::ports::TenantRepository + Send + Sync,
    tenant_service::application::TenantServiceError: std::fmt::Display,
{
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, AdminError> {
        let tenants = TenantServiceTrait::list_tenants(self)
            .await
            .map_err(|e| AdminError::Database(format!("tenant-service error: {e}").into()))?;
        Ok(tenants
            .into_iter()
            .map(|t| TenantSummary {
                id: t.id,
                name: t.name,
                created_at: t.created_at,
            })
            .collect())
    }
}

use counter_service::application::TenantScopedCounterService;
use counter_service::application::RepositoryBackedCounterService;
use kernel::TenantId;

#[async_trait]
impl<R> CounterReader for TenantScopedCounterService<R>
where
    R: counter_service::ports::CounterRepository + Send + Sync,
{
    async fn get_value(&self) -> Result<i64, AdminError> {
        // Admin dashboard gets the global counter (no specific tenant)
        let tenant_id = TenantId("admin".to_string());
        TenantScopedCounterService::get_value(self, &tenant_id)
            .await
            .map_err(|e| AdminError::Counter(format!("counter-service error: {e}")))
    }
}

#[async_trait]
impl<R> CounterReader for RepositoryBackedCounterService<R>
where
    R: counter_service::ports::CounterRepository + Send + Sync,
{
    async fn get_value(&self) -> Result<i64, AdminError> {
        // Use the default counter (no specific tenant)
        RepositoryBackedCounterService::get_value(self)
            .await
            .map_err(|e| AdminError::Counter(format!("counter-service error: {e}")))
    }
}

// ── AdminDashboardService ────────────────────────────────────────────────────

/// AdminService that composes TenantLister + CounterReader.
pub struct AdminDashboardService<T: TenantLister, C: CounterReader> {
    tenant_service: T,
    counter_service: C,
}

impl<T: TenantLister, C: CounterReader> AdminDashboardService<T, C> {
    pub fn new(tenant_service: T, counter_service: C) -> Self {
        Self {
            tenant_service,
            counter_service,
        }
    }
}

#[async_trait]
impl<T: TenantLister, C: CounterReader> AdminService for AdminDashboardService<T, C> {
    async fn get_dashboard_stats(&self) -> Result<AdminDashboardStats, AdminError> {
        let tenants = self.tenant_service.list_tenants().await?;
        let counter_value = self.counter_service.get_value().await?;

        Ok(AdminDashboardStats {
            tenant_count: tenants.len() as u64,
            counter_value,
            last_login: tenants.first().map(|t| t.created_at.clone()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
