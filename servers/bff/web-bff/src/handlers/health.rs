//! Health check handlers — 存活探针 + 就绪探针。
//!
//! Phase 0: 无外部依赖检查。Phase 1+: readyz 检查 DB, EventBus 等。

use axum::{Json, Router, routing::get};
use serde_json::json;

/// GET /healthz — 存活探针。
pub async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

/// GET /readyz — 就绪探针。
pub async fn readyz() -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (axum::http::StatusCode::OK, Json(json!({ "status": "ready" })))
}

/// 健康检查路由 — 无状态路由。
pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}
