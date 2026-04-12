use axum::{
    routing::get,
    Json,
    Router,
    extract::State,
};
use serde::Serialize;
use utoipa::OpenApi;
use crate::state::AdminBffState;
use crate::error::AdminBffResult;

#[derive(Serialize, utoipa::ToSchema)]
pub struct MetricsView {
    pub active_tenants: usize,
    pub total_users: usize,
    pub requests_per_minute: f64,
    pub error_rate: f64,
}

#[derive(OpenApi)]
#[openapi(
    paths(get_system_metrics),
    components(schemas(MetricsView))
)]
pub struct MetricsOpenApi;

/// GET /api/admin/metrics — System metrics for admin monitoring
#[utoipa::path(
    get,
    path = "/api/admin/metrics",
    responses(
        (status = 200, description = "Metrics retrieved successfully", body = MetricsView),
        (status = 401, description = "Unauthorized")
    ),
    tag = "admin"
)]
pub async fn get_system_metrics(
    State(_state): State<AdminBffState>,
) -> AdminBffResult<Json<MetricsView>> {
    // Placeholder — in production, aggregate from telemetry/monitoring
    let metrics = MetricsView {
        active_tenants: 0,
        total_users: 0,
        requests_per_minute: 0.0,
        error_rate: 0.0,
    };
    Ok(Json(metrics))
}

pub fn router() -> Router<AdminBffState> {
    Router::new()
        .route("/api/admin/metrics", get(get_system_metrics))
}
