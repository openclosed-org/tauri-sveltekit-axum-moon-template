//! User REST API handlers — web-bff version.
//!
//! GET    /api/user/me       — get current user profile
//! GET    /api/user/tenants  — list user's tenant bindings

use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use contracts_api::{UserProfileResponse, UserTenantResponse};
use contracts_errors::{ErrorCode, ErrorResponse};
use user_service::ports::{TenantRepository, UserRepository, UserTenantRepository};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::middleware::tenant::RequestContext;
use crate::state::BffState;

pub fn openapi_router() -> OpenApiRouter<BffState> {
    OpenApiRouter::new()
        .routes(routes!(get_user_profile))
        .routes(routes!(get_user_tenants))
}

/// Get current user profile.
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "user",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User profile retrieved", body = UserProfileResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn get_user_profile(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<UserProfileResponse>), (StatusCode, Json<ErrorResponse>)> {
    let user_sub = match extract_user_sub(request_context) {
        Ok(id) => id,
        Err(error) => return Err(error),
    };

    let user_repo = match state.user_profile_repository() {
        Some(repo) => repo,
        None => return Err(db_not_ready()),
    };
    let result = user_repo.find_by_sub(&user_sub).await;

    match result {
        Ok(Some(user)) => Ok((
            StatusCode::OK,
            Json(UserProfileResponse {
                id: user.id,
                user_sub: user.user_sub,
                display_name: user.display_name,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
                last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
            }),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(ErrorCode::NotFound, "User not found")),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to load user profile: {e}"),
            )),
        )),
    }
}

/// List user's tenant bindings.
#[utoipa::path(
    get,
    path = "/api/user/tenants",
    tag = "user",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User tenants retrieved", body = Vec<UserTenantResponse>, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn get_user_tenants(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<Vec<UserTenantResponse>>), (StatusCode, Json<ErrorResponse>)> {
    let user_sub = match extract_user_sub(request_context) {
        Ok(id) => id,
        Err(error) => return Err(error),
    };

    let (binding_repo, tenant_repo) = match state.user_read_repositories() {
        Some(repos) => repos,
        None => return Err(db_not_ready()),
    };

    match binding_repo.find_user_tenant(&user_sub).await {
        Ok(Some(binding)) => match tenant_repo.find_by_id(&binding.tenant_id).await {
            Ok(Some(tenant_info)) => Ok((
                StatusCode::OK,
                Json(vec![UserTenantResponse {
                    tenant_id: tenant_info.id,
                    tenant_name: Some(tenant_info.name),
                    role: binding.role,
                    joined_at: binding.joined_at.to_rfc3339(),
                }]),
            )),
            Ok(None) => Ok((
                StatusCode::OK,
                Json(vec![UserTenantResponse {
                    tenant_id: binding.tenant_id,
                    tenant_name: None,
                    role: binding.role,
                    joined_at: binding.joined_at.to_rfc3339(),
                }]),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to load tenant details: {e}"),
                )),
            )),
        },
        Ok(None) => Ok((StatusCode::OK, Json(Vec::new()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to load user tenant bindings: {e}"),
            )),
        )),
    }
}

// ── Helpers ──────────────────────────────────────────────────

fn db_not_ready() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            ErrorCode::InternalError,
            "Embedded database not initialized",
        )),
    )
}

fn extract_user_sub(
    request_context: Option<Extension<RequestContext>>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    request_context
        .map(|Extension(context)| context.user_sub)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new(
                    ErrorCode::Unauthorized,
                    "Missing authenticated request context",
                )),
            )
        })
}
