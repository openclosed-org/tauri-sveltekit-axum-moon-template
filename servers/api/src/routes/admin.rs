//! Admin REST API routes.

use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use contracts_api::AdminDashboardStats;
use feature_admin::AdminService;
use utoipa::OpenApi;

pub fn router() -> Router<AppState> {
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
async fn get_dashboard_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return Json(serde_json::json!({ "error": "Embedded database not initialized" })),
    };

    // Build tenant service via new service location
    let tenant_repo = tenant_service::infrastructure::LibSqlTenantRepository::new(db.clone());
    let tenant_svc = tenant_service::application::TenantService::new(tenant_repo);

    // Build counter service
    let counter_repo = counter_service::infrastructure::LibSqlCounterRepository::new(db.clone());
    let counter_svc = counter_service::application::RepositoryBackedCounterService::new(counter_repo);

    // Build admin service
    let admin_svc = admin_service::application::AdminDashboardService::new(tenant_svc, counter_svc);

    match admin_svc.get_dashboard_stats().await {
        Ok(stats) => Json(serde_json::json!(stats)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
