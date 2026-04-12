//! User REST API routes.
//!
//! GET /api/user/me — get current user profile
//! GET /api/user/tenants — list user's tenant bindings

use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
};
use domain::ports::TenantId;
use user_service::infrastructure::{LibSqlUserRepository, LibSqlUserTenantRepository};
use user_service::ports::{UserRepository, UserTenantRepository, TenantRepository};
use utoipa::OpenApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/user/me", get(get_user_profile))
        .route("/api/user/tenants", get(get_user_tenants))
}

/// Get current user profile.
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "user",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "User profile retrieved", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing user context"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_user_profile(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // tenant_id.0 contains the JWT `sub` claim (extracted by tenant_middleware)
    let user_sub = tenant_id.0;

    // Build user service
    let user_repo = LibSqlUserRepository::new(db);

    match user_repo.find_by_sub(&user_sub).await {
        Ok(Some(user)) => (StatusCode::OK, Json(serde_json::json!({
            "id": user.id,
            "user_sub": user.user_sub,
            "display_name": user.display_name,
            "email": user.email,
            "created_at": user.created_at.to_rfc3339(),
            "last_login_at": user.last_login_at.map(|dt| dt.to_rfc3339()),
        }))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// List user's tenant bindings.
#[utoipa::path(
    get,
    path = "/api/user/tenants",
    tag = "user",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "User tenants retrieved", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_user_tenants(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // tenant_id.0 contains the JWT `sub` claim
    let user_sub = tenant_id.0;

    // Build user service
    let binding_repo = LibSqlUserTenantRepository::new(db.clone());
    let tenant_repo = user_service::infrastructure::LibSqlTenantRepository::new(db);

    match binding_repo.find_user_tenant(&user_sub).await {
        Ok(Some(binding)) => {
            // Fetch tenant details
            match tenant_repo.find_by_id(&binding.tenant_id).await {
                Ok(Some(tenant)) => (StatusCode::OK, Json(serde_json::json!([{
                    "tenant_id": tenant.id,
                    "tenant_name": tenant.name,
                    "role": binding.role,
                    "joined_at": binding.joined_at.to_rfc3339(),
                }]))),
                Ok(None) | Err(_) => (
                    StatusCode::OK,
                    Json(serde_json::json!([{
                        "tenant_id": binding.tenant_id,
                        "role": binding.role,
                        "joined_at": binding.joined_at.to_rfc3339(),
                    }])),
                ),
            }
        },
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!([])),
        ),
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
