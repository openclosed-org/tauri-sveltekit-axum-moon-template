//! Health check endpoints for load balancer and orchestrator probes.

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Liveness probe — is the process alive?
/// GET /healthz → {"status": "ok"}
pub async fn healthz() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Readiness probe — can the server accept traffic?
/// GET /readyz → {"status": "ready"} or {"status": "degraded"}
///
/// Checks: SurrealDB connection health.
pub async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    // Check SurrealDB connection
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
///
/// NOTE: With AppState, routes must use `Router::<AppState>::new()`.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}
