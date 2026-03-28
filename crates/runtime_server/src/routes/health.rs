//! Health check endpoints for load balancer and orchestrator probes.

use axum::{http::StatusCode, routing::get, Json, Router};
use serde_json::{json, Value};

/// Liveness probe — is the process alive?
/// GET /healthz → {"status": "ok"}
pub async fn healthz() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Readiness probe — can the server accept traffic?
/// GET /readyz → {"status": "ready"} or {"status": "degraded"}
pub async fn readyz() -> (StatusCode, Json<Value>) {
    // TODO Phase 5: check database connection pool health
    (StatusCode::OK, Json(json!({"status": "ready"})))
}

/// Health route module router.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}
