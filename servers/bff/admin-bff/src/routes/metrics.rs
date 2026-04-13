//! Admin metrics routes — placeholder for observability integration.
//!
//! ⚠️ This endpoint returns placeholder data.
//! In production, this should integrate with the observability stack
//! (OpenObserve, Prometheus, or similar metrics backend).

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

/// GET /api/admin/metrics — System metrics for admin monitoring.
///
/// ⚠️ PLACEHOLDER: Returns zero values.
/// TODO: Integrate with observability backend (OpenObserve/Prometheus).
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
    // TODO: Replace with actual observability integration
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
