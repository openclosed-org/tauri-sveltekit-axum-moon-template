//! Admin REST API handlers — migrated to web-bff.
//!
//! GET /api/admin/stats — dashboard statistics.

use axum::{Json, Router, extract::State, routing::get};
use contracts_api::AdminDashboardStats;
use feature_admin::AdminService;
use serde_json::json;
use utoipa::OpenApi;

use crate::state::BffState;
use crate::error::{BffError, BffResult};

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
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => {
            return Json(json!({ "error": "Embedded database not initialized" }));
        }
    };

    let tenant_service = usecases::tenant_service::LibSqlTenantService::new(db.clone());
    let counter_service = usecases::counter_service::LibSqlCounterService::new(db.clone());
    let admin_service =
        usecases::admin_service::LibSqlAdminService::new(db, tenant_service, counter_service);

    match admin_service.get_dashboard_stats().await {
        Ok(stats) => Json(json!(stats)),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
