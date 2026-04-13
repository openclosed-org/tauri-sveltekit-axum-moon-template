use axum::{
    routing::get,
    Json,
    Router,
    extract::State,
};
use utoipa::OpenApi;
use crate::state::AdminBffState;
use crate::error::AdminBffResult;
use crate::handlers::dashboard::{DashboardView, fetch_dashboard};

#[derive(OpenApi)]
#[openapi(
    paths(get_dashboard_stats),
    components(schemas(DashboardView))
)]
pub struct AdminOpenApi;

/// GET /api/admin/dashboard — Aggregated admin dashboard view
#[utoipa::path(
    get,
    path = "/api/admin/dashboard",
    responses(
        (status = 200, description = "Dashboard stats retrieved successfully", body = DashboardView),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Internal API unavailable")
    ),
    tag = "admin"
)]
pub async fn get_dashboard_stats(
    State(state): State<AdminBffState>,
) -> AdminBffResult<Json<DashboardView>> {
    let db = state.embedded_db.clone()
        .ok_or_else(|| crate::error::AdminBffError::Internal("Embedded database not initialized".to_string()))?;

    let dashboard = fetch_dashboard(db).await?;
    Ok(Json(dashboard))
}

pub fn router() -> Router<AdminBffState> {
    Router::new()
        .route("/api/admin/dashboard", get(get_dashboard_stats))
}
