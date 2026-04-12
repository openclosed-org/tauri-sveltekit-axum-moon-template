//! User REST API handlers — web-bff version.
//!
//! GET    /api/user/me       — get current user profile
//! GET    /api/user/tenants  — list user's tenant bindings

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
};
use domain::ports::TenantId;
use user_service::infrastructure::{LibSqlUserRepository, LibSqlUserTenantRepository};
use user_service::ports::{TenantRepository, UserRepository, UserTenantRepository};
use utoipa::OpenApi;

use crate::state::BffState;

pub fn router() -> Router<BffState> {
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
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_user_profile(
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    let user_sub = match extract_user_sub(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let user_repo = LibSqlUserRepository::new(db);

    match user_repo.find_by_sub(&user_sub).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": user.id,
                "user_sub": user.user_sub,
                "display_name": user.display_name,
                "email": user.email,
                "created_at": user.created_at.to_rfc3339(),
                "last_login_at": user.last_login_at.map(|dt| dt.to_rfc3339()),
            })),
        ),
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
    State(state): State<BffState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    let user_sub = match extract_user_sub(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let binding_repo = LibSqlUserTenantRepository::new(db.clone());
    let tenant_repo = user_service::infrastructure::LibSqlTenantRepository::new(db);

    match binding_repo.find_user_tenant(&user_sub).await {
        Ok(Some(binding)) => {
            match tenant_repo.find_by_id(&binding.tenant_id).await {
                Ok(Some(tenant_info)) => (
                    StatusCode::OK,
                    Json(serde_json::json!([{
                        "tenant_id": tenant_info.id,
                        "tenant_name": tenant_info.name,
                        "role": binding.role,
                        "joined_at": binding.joined_at.to_rfc3339(),
                    }])),
                ),
                Ok(None) | Err(_) => (
                    StatusCode::OK,
                    Json(serde_json::json!([{
                        "tenant_id": binding.tenant_id,
                        "role": binding.role,
                        "joined_at": binding.joined_at.to_rfc3339(),
                    }])),
                ),
            }
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!([]))),
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

fn extract_user_sub(
    tenant: Option<Extension<TenantId>>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    tenant
        .map(|Extension(id)| id.0)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing JWT — not authenticated" })),
            )
        })
}
