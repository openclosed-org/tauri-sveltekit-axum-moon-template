//! Admin REST API handlers — migrated to web-bff.
//!
//! GET /api/admin/stats — dashboard statistics.

use axum::{Json, Router, extract::State, routing::get};
use async_trait::async_trait;
use contracts_api::AdminDashboardStats;
use feature_admin::AdminService;
use kernel::TenantId;
use serde_json::json;
use utoipa::OpenApi;
use storage_turso::EmbeddedTurso;
use tenant_service::application::TenantServiceTrait;
use feature_counter::CounterService;

use crate::state::BffState;
use crate::error::{BffError, BffResult};

/// Adapter: TenantService → admin_service::ports::TenantRepository
pub struct TenantServiceAdapter {
    tenant_svc: tenant_service::application::TenantService<
        tenant_service::infrastructure::LibSqlTenantRepository<storage_turso::EmbeddedTurso>,
    >,
}

impl TenantServiceAdapter {
    pub fn new(tenant_svc: tenant_service::application::TenantService<
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
pub struct CounterServiceAdapter {
    counter_svc: counter_service::application::RepositoryBackedCounterService<
        counter_service::infrastructure::LibSqlCounterRepository<storage_turso::EmbeddedTurso>,
    >,
}

impl CounterServiceAdapter {
    pub fn new(counter_svc: counter_service::application::RepositoryBackedCounterService<
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

pub fn router() -> Router<BffState> {
    Router::new().route("/api/admin/stats", get(get_dashboard_stats))
}

/// Get dashboard statistics (tenant count, counter value, etc.).
#[utoipa::path(
    get,
    path = "/api/admin/stats",
    tag = "admin",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Dashboard statistics retrieved successfully", body = AdminDashboardStats, content_type = "application/json"),
        (status = 500, description = "Internal server error", body = serde_json::Value, content_type = "application/json"),
    ),
)]
async fn get_dashboard_stats(State(state): State<BffState>) -> Json<serde_json::Value> {
    let db: EmbeddedTurso = match state.embedded_db.clone() {
        Some(db) => db,
        None => {
            return Json(json!({ "error": "Embedded database not initialized" }));
        }
    };

    // Build tenant service
    let tenant_repo = tenant_service::infrastructure::LibSqlTenantRepository::new(db.clone());
    let tenant_svc = tenant_service::application::TenantService::new(tenant_repo);
    let tenant_adapter = TenantServiceAdapter::new(tenant_svc);

    // Build counter service
    let counter_repo = counter_service::infrastructure::LibSqlCounterRepository::new(db.clone());
    let counter_svc = counter_service::application::RepositoryBackedCounterService::new(counter_repo);
    let counter_adapter = CounterServiceAdapter::new(counter_svc);

    // Build admin service with adapted ports
    let admin_svc = admin_service::application::AdminDashboardService::new(tenant_adapter, counter_adapter);

    match admin_svc.get_dashboard_stats().await {
        Ok(stats) => Json(json!(stats)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
