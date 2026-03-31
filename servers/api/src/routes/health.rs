//! Health check endpoints for load balancer and orchestrator probes.

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::{Value, json};
use utoipa::ToSchema;

use crate::state::AppState;

/// Health check response.
#[derive(Debug, ToSchema)]
pub struct HealthResponse {
    /// Server status: "ok"
    pub status: String,
}

/// Liveness probe — is the process alive?
///
/// Returns 200 OK if the process is running.
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "Server is alive", body = HealthResponse, content_type = "application/json"),
    ),
)]
pub async fn healthz() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Readiness probe — can the server accept traffic?
///
/// Checks: SurrealDB connection health.
/// Returns 200 if ready, 503 if database is unavailable.
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    responses(
        (status = 200, description = "Server is ready", body = HealthResponse, content_type = "application/json"),
        (status = 503, description = "Service unavailable", body = serde_json::Value, content_type = "application/json"),
    ),
)]
pub async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let db_ok = state.db.health().await.is_ok();

    if db_ok {
        (StatusCode::OK, Json(json!({"status": "ready"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "degraded", "reason": "database_unavailable"})),
        )
    }
}

/// Health route module router.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}
