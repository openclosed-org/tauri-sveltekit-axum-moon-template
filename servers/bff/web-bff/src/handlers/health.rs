//! Health check handlers — 存活探针 + 就绪探针。
//!
//! Phase 0: 无外部依赖检查。Phase 1+: readyz 检查 DB, EventBus 等。

use axum::Json;
use contracts_api::HealthResponse;
use utoipa_axum::{router::OpenApiRouter, routes};

/// GET /healthz — 存活探针。
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "Service is alive", body = HealthResponse)
    )
)]
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse::new("ok"))
}

/// GET /readyz — 就绪探针。
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", body = HealthResponse)
    )
)]
pub async fn readyz() -> (axum::http::StatusCode, Json<HealthResponse>) {
    (
        axum::http::StatusCode::OK,
        Json(HealthResponse::new("ready")),
    )
}

/// 健康检查路由 — 无状态路由。
pub fn openapi_router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S> {
    OpenApiRouter::new()
        .routes(routes!(healthz))
        .routes(routes!(readyz))
}
