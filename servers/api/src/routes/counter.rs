//! Counter REST API routes.

use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
};
use domain::ports::TenantId;
use feature_counter::CounterService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/counter/increment", post(increment))
        .route("/api/counter/decrement", post(decrement))
        .route("/api/counter/reset", post(reset))
        .route("/api/counter/value", get(get_value))
}

fn get_db(
    state: &AppState,
) -> Result<storage_turso::EmbeddedTurso, (StatusCode, Json<serde_json::Value>)> {
    state.embedded_db.clone().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Embedded database not initialized" })),
        )
    })
}

fn get_tenant(
    tenant: Option<Extension<TenantId>>,
) -> Result<TenantId, (StatusCode, Json<serde_json::Value>)> {
    tenant.map(|Extension(id)| id).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing tenant context" })),
        )
    })
}

async fn increment(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let tenant_id = match get_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.increment_for_tenant(&tenant_id).await {
        Ok(value) => (StatusCode::OK, Json(serde_json::json!({ "value": value }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn decrement(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let tenant_id = match get_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.decrement_for_tenant(&tenant_id).await {
        Ok(value) => (StatusCode::OK, Json(serde_json::json!({ "value": value }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn reset(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let tenant_id = match get_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.reset_for_tenant(&tenant_id).await {
        Ok(value) => (StatusCode::OK, Json(serde_json::json!({ "value": value }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_value(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let tenant_id = match get_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.get_value_for_tenant(&tenant_id).await {
        Ok(value) => (StatusCode::OK, Json(serde_json::json!({ "value": value }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
