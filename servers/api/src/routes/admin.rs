//! Admin REST API routes.

use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use feature_admin::AdminService;

pub fn router() -> Router<AppState> {
    Router::new().route("/admin/stats", get(get_dashboard_stats))
}

async fn get_dashboard_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return Json(serde_json::json!({ "error": "Embedded database not initialized" })),
    };
    let tenant_service = usecases::tenant_service::LibSqlTenantService::new(db.clone());
    let counter_service = usecases::counter_service::LibSqlCounterService::new(db.clone());
    let admin_service =
        usecases::admin_service::LibSqlAdminService::new(db, tenant_service, counter_service);
    match admin_service.get_dashboard_stats().await {
        Ok(stats) => Json(serde_json::json!(stats)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
