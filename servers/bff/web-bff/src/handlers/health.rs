//! Health check handlers — 存活探针 + 就绪探针。
//!
//! Phase 0: 无外部依赖检查。Phase 1+: readyz 检查 DB, EventBus 等。

use axum::{Json, extract::State, http::StatusCode};
use contracts_api::HealthResponse;
use serde::Serialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::BffState;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unavailable: Vec<String>,
}

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
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service dependencies are not ready", body = ReadinessResponse)
    )
)]
pub async fn readyz(State(state): State<BffState>) -> (StatusCode, Json<ReadinessResponse>) {
    let readiness = state.readiness();
    if readiness.is_ready() {
        return (
            StatusCode::OK,
            Json(ReadinessResponse {
                status: "ready".to_string(),
                unavailable: Vec::new(),
            }),
        );
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ReadinessResponse {
            status: "not_ready".to_string(),
            unavailable: readiness.unavailable().to_vec(),
        }),
    )
}

/// 健康检查路由 — 无状态路由。
pub fn openapi_router() -> OpenApiRouter<BffState> {
    OpenApiRouter::new()
        .routes(routes!(healthz))
        .routes(routes!(readyz))
}
