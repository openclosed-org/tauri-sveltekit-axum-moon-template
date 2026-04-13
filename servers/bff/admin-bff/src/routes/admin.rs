use crate::error::AdminBffResult;
use crate::handlers::dashboard::fetch_dashboard;
use crate::state::AdminBffState;
use axum::{extract::State, routing::get, Json, Router};

pub use crate::handlers::dashboard::DashboardView;

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
    let db = state.embedded_db.clone().ok_or_else(|| {
        crate::error::AdminBffError::Internal("Embedded database not initialized".to_string())
    })?;

    let dashboard = fetch_dashboard(db).await?;
    Ok(Json(dashboard))
}

pub fn router() -> Router<AdminBffState> {
    Router::new().route("/api/admin/dashboard", get(get_dashboard_stats))
}
