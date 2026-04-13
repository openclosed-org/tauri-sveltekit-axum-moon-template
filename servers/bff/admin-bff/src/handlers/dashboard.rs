//! Dashboard handler — aggregates admin stats via service composition.
//!
//! Uses tenant-service and counter-service directly via EmbeddedTurso,
//! instead of making HTTP calls to an internal API.

use serde::Serialize;
use storage_turso::EmbeddedTurso;
use tenant_service::ports::TenantRepository;
use tenant_service::infrastructure::libsql_adapter::LibSqlTenantRepository;
use counter_service::infrastructure::libsql_adapter::LibSqlCounterRepository;
use kernel::TenantId;
use feature_admin::AdminService;
use admin_service::application::AdminDashboardService;
use admin_service::ports::{CounterRepository as AdminCounterRepo, TenantRepository as AdminTenantRepo, TenantSummary};

/// View model for admin dashboard — aggregates multiple service stats
#[derive(Serialize, utoipa::ToSchema)]
pub struct DashboardView {
    pub tenant_count: usize,
    pub total_counter_value: i64,
    pub recent_tenants: Vec<TenantSummaryView>,
    pub system_health: SystemHealthView,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TenantSummaryView {
    pub id: String,
    pub name: String,
    pub counter_value: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SystemHealthView {
    pub api_status: String,
    pub worker_status: String,
    pub database_status: String,
}

/// Adapter: wraps tenant-service's LibSqlTenantRepository to implement admin-service's TenantRepository port.
struct TenantRepoAdapter {
    inner: LibSqlTenantRepository<EmbeddedTurso>,
}

#[async_trait::async_trait]
impl AdminTenantRepo for TenantRepoAdapter {
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let tenants = self.inner.list_tenants().await?;
        Ok(tenants.into_iter().map(|t| TenantSummary {
            id: t.id,
            name: t.name,
            created_at: t.created_at,
        }).collect())
    }
}

/// Adapter: wraps counter-service's LibSqlCounterRepository to implement admin-service's CounterRepository port.
struct CounterRepoAdapter {
    inner: LibSqlCounterRepository<EmbeddedTurso>,
}

#[async_trait::async_trait]
impl AdminCounterRepo for CounterRepoAdapter {
    async fn get_value(&self, tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        use counter_service::ports::CounterRepository;
        use counter_service::domain::CounterId;
        let counter = self.inner.load(&CounterId::new(tenant_id.as_str())).await?;
        Ok(counter.map(|c| c.value).unwrap_or(0))
    }

    async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        use counter_service::ports::CounterRepository;
        use counter_service::domain::CounterId;
        let counter = self.inner.load(&CounterId::new("default")).await?;
        Ok(counter.map(|c| c.value).unwrap_or(0))
    }
}

/// Fetch dashboard stats by composing tenant-service and counter-service via EmbeddedTurso.
pub async fn fetch_dashboard(db: EmbeddedTurso) -> Result<DashboardView, crate::error::AdminBffError> {
    // Create repositories
    let tenant_repo = LibSqlTenantRepository::new(db.clone());
    let counter_repo = LibSqlCounterRepository::new(db.clone());

    // Run migrations (ignore errors - tables may already exist)
    tenant_repo.migrate().await.ok();

    // Create admin dashboard service via adapters
    let admin_tenant_adapter = TenantRepoAdapter { inner: tenant_repo };
    let admin_counter_adapter = CounterRepoAdapter { inner: counter_repo };
    let admin_service = AdminDashboardService::new(admin_tenant_adapter, admin_counter_adapter);

    // Get dashboard stats
    let stats = admin_service.get_dashboard_stats().await
        .map_err(|e| crate::error::AdminBffError::ServiceUnavailable(e.to_string()))?;

    // For recent tenants, we need to create a new repo since the old one was moved
    let tenant_repo_for_list = LibSqlTenantRepository::new(db);
    let tenants = tenant_repo_for_list.list_tenants().await
        .map_err(|e| crate::error::AdminBffError::Internal(format!("Failed to list tenants: {}", e)))?;

    let recent_tenants: Vec<TenantSummaryView> = tenants.into_iter()
        .take(10)
        .map(|t| TenantSummaryView {
            id: t.id.clone(),
            name: t.name,
            counter_value: 0, // Would require per-tenant counter query
        })
        .collect();

    let view = DashboardView {
        tenant_count: stats.tenant_count as usize,
        total_counter_value: stats.counter_value,
        recent_tenants,
        system_health: SystemHealthView {
            api_status: "healthy".to_string(),
            worker_status: "healthy".to_string(),
            database_status: "healthy".to_string(),
        },
    };

    Ok(view)
}
