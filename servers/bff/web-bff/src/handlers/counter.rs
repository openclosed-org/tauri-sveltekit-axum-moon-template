//! Counter REST API handlers — web-bff version.
//!
//! These handlers use the counter-service implementation via its repository.

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
};
use counter_service::contracts::service::CounterService;
use kernel::TenantId;
use utoipa::OpenApi;

use crate::state::{BffState, DatabaseBackend};

pub fn router() -> Router<BffState> {
    Router::new()
        .route("/api/counter/increment", post(increment))
        .route("/api/counter/decrement", post(decrement))
        .route("/api/counter/reset", post(reset))
        .route("/api/counter/value", get(get_value))
}

/// Increment the tenant's counter value.
#[utoipa::path(
    post,
    path = "/api/counter/increment",
    tag = "counter",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Counter incremented successfully", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing tenant context"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn increment(
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let kernel_tid = kernel::TenantId(tenant_id.0.clone());

    let result = match state.db.clone() {
        Some(DatabaseBackend::Embedded(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.increment(&kernel_tid, None).await
        }
        Some(DatabaseBackend::Remote(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.increment(&kernel_tid, None).await
        }
        None => return db_not_ready(),
    };

    match result {
        Ok(value) => {
            // Invalidate cache on mutation
            let cache_key = format!("counter:{}", kernel_tid.as_str());
            state.counter_cache.invalidate(&cache_key).await;
            (StatusCode::OK, Json(serde_json::json!({ "value": value })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// Decrement the tenant's counter value.
#[utoipa::path(
    post,
    path = "/api/counter/decrement",
    tag = "counter",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Counter decremented successfully", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing tenant context"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn decrement(
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let kernel_tid = kernel::TenantId(tenant_id.0.clone());

    let result = match state.db.clone() {
        Some(DatabaseBackend::Embedded(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.decrement(&kernel_tid, None).await
        }
        Some(DatabaseBackend::Remote(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.decrement(&kernel_tid, None).await
        }
        None => return db_not_ready(),
    };

    match result {
        Ok(value) => {
            let cache_key = format!("counter:{}", kernel_tid.as_str());
            state.counter_cache.invalidate(&cache_key).await;
            (StatusCode::OK, Json(serde_json::json!({ "value": value })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// Reset the tenant's counter value to zero.
#[utoipa::path(
    post,
    path = "/api/counter/reset",
    tag = "counter",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Counter reset successfully", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing tenant context"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn reset(
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let kernel_tid = kernel::TenantId(tenant_id.0.clone());

    let result = match state.db.clone() {
        Some(DatabaseBackend::Embedded(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.reset(&kernel_tid, None).await
        }
        Some(DatabaseBackend::Remote(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.reset(&kernel_tid, None).await
        }
        None => return db_not_ready(),
    };

    match result {
        Ok(value) => {
            let cache_key = format!("counter:{}", kernel_tid.as_str());
            state.counter_cache.invalidate(&cache_key).await;
            (StatusCode::OK, Json(serde_json::json!({ "value": value })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// Get the current counter value for the authenticated tenant.
#[utoipa::path(
    get,
    path = "/api/counter/value",
    tag = "counter",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Current counter value", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing tenant context"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_value(
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let kernel_tid = kernel::TenantId(tenant_id.0.clone());

    let result = match state.db.clone() {
        Some(DatabaseBackend::Embedded(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.get_value(&kernel_tid).await
        }
        Some(DatabaseBackend::Remote(db)) => {
            let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
            let service = counter_service::application::TenantScopedCounterService::new(repo);
            service.get_value(&kernel_tid).await
        }
        None => return db_not_ready(),
    };

    let cache_key = format!("counter:{}", kernel_tid.as_str());

    // Check cache first
    if let Some(cached) = state.counter_cache.get(&cache_key).await {
        return (StatusCode::OK, Json(serde_json::json!({ "value": cached })));
    }

    match result {
        Ok(value) => {
            // Populate cache on read
            state.counter_cache.insert(cache_key, value).await;
            (StatusCode::OK, Json(serde_json::json!({ "value": value })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ── Helpers ──────────────────────────────────────────────────

fn db_not_ready() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Embedded database not initialized" })),
    )
}

fn extract_tenant(
    tenant: Option<Extension<TenantId>>,
) -> Result<TenantId, (StatusCode, Json<serde_json::Value>)> {
    tenant.map(|Extension(id)| id).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing tenant context" })),
        )
    })
}
