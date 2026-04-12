use axum::{
    routing::get,
    Json,
    Router,
};
use serde::Serialize;
use crate::state::AdminBffState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// GET /healthz — Liveness probe
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /readyz — Readiness probe
pub async fn readyz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub fn router() -> Router<AdminBffState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}
