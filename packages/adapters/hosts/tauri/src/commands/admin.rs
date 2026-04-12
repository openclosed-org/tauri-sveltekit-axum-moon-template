//! Admin Tauri commands — bridge to AdminService.

use async_trait::async_trait;
use feature_admin::AdminService;
use kernel::TenantId;
use storage_turso::EmbeddedTurso;
use tauri::Manager;
use tenant_service::application::TenantServiceTrait;
use feature_counter::CounterService;

/// Adapter: TenantService → admin_service::ports::TenantRepository
struct TenantServiceAdapter {
    tenant_svc: tenant_service::application::TenantService<
        tenant_service::infrastructure::LibSqlTenantRepository<storage_turso::EmbeddedTurso>,
    >,
}

impl TenantServiceAdapter {
    fn new(tenant_svc: tenant_service::application::TenantService<
        tenant_service::infrastructure::LibSqlTenantRepository<storage_turso::EmbeddedTurso>,
    >) -> Self {
        Self { tenant_svc }
    }
}

#[async_trait]
impl admin_service::ports::TenantRepository for TenantServiceAdapter {
    async fn list_tenants(&self) -> Result<Vec<admin_service::ports::TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let tenants = TenantServiceTrait::list_tenants(&self.tenant_svc).await.map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
        })?;
        Ok(tenants
            .into_iter()
            .map(|t| admin_service::ports::TenantSummary {
                id: t.id,
                name: t.name,
                created_at: t.created_at,
            })
            .collect())
    }
}

/// Adapter: CounterService → admin_service::ports::CounterRepository
struct CounterServiceAdapter {
    counter_svc: counter_service::application::RepositoryBackedCounterService<
        counter_service::infrastructure::LibSqlCounterRepository<storage_turso::EmbeddedTurso>,
    >,
}

impl CounterServiceAdapter {
    fn new(counter_svc: counter_service::application::RepositoryBackedCounterService<
        counter_service::infrastructure::LibSqlCounterRepository<storage_turso::EmbeddedTurso>,
    >) -> Self {
        Self { counter_svc }
    }
}

#[async_trait]
impl admin_service::ports::CounterRepository for CounterServiceAdapter {
    async fn get_value(&self, _tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        CounterService::get_value(&self.counter_svc).await.map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
        })
    }

    async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        CounterService::get_value(&self.counter_svc).await.map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

#[tauri::command]
pub async fn admin_get_dashboard_stats(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();

    let tenant_repo = tenant_service::infrastructure::LibSqlTenantRepository::new(db.clone());
    let tenant_svc = tenant_service::application::TenantService::new(tenant_repo);
    let tenant_adapter = TenantServiceAdapter::new(tenant_svc);

    let counter_repo = counter_service::infrastructure::LibSqlCounterRepository::new(db.clone());
    let counter_svc = counter_service::application::RepositoryBackedCounterService::new(counter_repo);
    let counter_adapter = CounterServiceAdapter::new(counter_svc);

    let admin_svc = admin_service::application::AdminDashboardService::new(tenant_adapter, counter_adapter);

    match admin_svc.get_dashboard_stats().await {
        Ok(stats) => Ok(serde_json::json!(stats)),
        Err(e) => Err(e.to_string()),
    }
}
