//! Counter REST API handlers — web-bff version.
//!
//! These handlers use the counter-service implementation via its repository.
//! All responses use contract DTOs from `contracts_api` and `contracts_errors`.

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
};
use contracts_api::CounterResponse;
use contracts_errors::{ErrorCode, ErrorResponse};
use counter_service::contracts::service::CounterCommandContext;
use counter_service::contracts::service::{CounterError, CounterService};
use counter_service::domain::CounterId;
use user_service::ports::UserTenantRepository;

use crate::composition::CounterServiceHandle;
use crate::middleware::tenant::RequestContext;
use crate::state::BffState;

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
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Counter incremented successfully", body = CounterResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 409, description = "CAS conflict — concurrent modification", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
async fn increment(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<CounterResponse>), (StatusCode, Json<ErrorResponse>)> {
    let request_context = extract_request_context(request_context)?;
    let tenant_id = resolve_tenant_id(&state, &request_context).await?;

    // Authz check: user must have can_write on the counter resource
    check_authz(
        &state,
        &request_context.user_sub,
        "can_write",
        &format!("counter:{}", tenant_id.as_str()),
    )
    .await?;

    let command_context = build_command_context(request_context);
    let service = build_service(&state)?;

    let value = service
        .increment_with_context(&CounterId::new(tenant_id.as_str()), None, &command_context)
        .await
        .map_err(map_counter_error)?;

    // Invalidate cache on mutation
    let cache_key = format!("counter:{}", tenant_id.as_str());
    state.counter_cache.invalidate(&cache_key).await;

    Ok((StatusCode::OK, Json(CounterResponse { value })))
}

/// Decrement the tenant's counter value.
#[utoipa::path(
    post,
    path = "/api/counter/decrement",
    tag = "counter",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Counter decremented successfully", body = CounterResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 409, description = "CAS conflict — concurrent modification", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
async fn decrement(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<CounterResponse>), (StatusCode, Json<ErrorResponse>)> {
    let request_context = extract_request_context(request_context)?;
    let tenant_id = resolve_tenant_id(&state, &request_context).await?;

    check_authz(
        &state,
        &request_context.user_sub,
        "can_write",
        &format!("counter:{}", tenant_id.as_str()),
    )
    .await?;

    let command_context = build_command_context(request_context);
    let service = build_service(&state)?;

    let value = service
        .decrement_with_context(&CounterId::new(tenant_id.as_str()), None, &command_context)
        .await
        .map_err(map_counter_error)?;

    // Invalidate cache on mutation
    let cache_key = format!("counter:{}", tenant_id.as_str());
    state.counter_cache.invalidate(&cache_key).await;

    Ok((StatusCode::OK, Json(CounterResponse { value })))
}

/// Reset the tenant's counter value to zero.
#[utoipa::path(
    post,
    path = "/api/counter/reset",
    tag = "counter",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Counter reset successfully", body = CounterResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 409, description = "CAS conflict — concurrent modification", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
async fn reset(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<CounterResponse>), (StatusCode, Json<ErrorResponse>)> {
    let request_context = extract_request_context(request_context)?;
    let tenant_id = resolve_tenant_id(&state, &request_context).await?;

    check_authz(
        &state,
        &request_context.user_sub,
        "can_write",
        &format!("counter:{}", tenant_id.as_str()),
    )
    .await?;

    let command_context = build_command_context(request_context);
    let service = build_service(&state)?;

    let value = service
        .reset_with_context(&CounterId::new(tenant_id.as_str()), None, &command_context)
        .await
        .map_err(map_counter_error)?;

    // Invalidate cache on mutation
    let cache_key = format!("counter:{}", tenant_id.as_str());
    state.counter_cache.invalidate(&cache_key).await;

    Ok((StatusCode::OK, Json(CounterResponse { value })))
}

/// Get the current counter value for the authenticated tenant.
/// Cache-first: checks cache before hitting the database.
#[utoipa::path(
    get,
    path = "/api/counter/value",
    tag = "counter",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current counter value", body = CounterResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
async fn get_value(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<CounterResponse>), (StatusCode, Json<ErrorResponse>)> {
    let request_context = extract_request_context(request_context)?;
    let tenant_id = resolve_tenant_id(&state, &request_context).await?;
    let cache_key = format!("counter:{}", tenant_id.as_str());

    // Authz check: user must have can_read on the counter resource
    check_authz(
        &state,
        &request_context.user_sub,
        "can_read",
        &format!("counter:{}", tenant_id.as_str()),
    )
    .await?;

    // Cache-first: check cache before hitting database
    if let Some(cached) = state.counter_cache.get(&cache_key).await {
        return Ok((StatusCode::OK, Json(CounterResponse { value: cached })));
    }

    let service = build_service(&state)?;

    let value = service
        .get_value(&CounterId::new(tenant_id.as_str()))
        .await
        .map_err(map_counter_error)?;

    // Populate cache on read
    state.counter_cache.insert(cache_key.clone(), value).await;

    Ok((StatusCode::OK, Json(CounterResponse { value })))
}

// ── Helpers ──────────────────────────────────────────────────

/// Perform an authorization check against the authz adapter.
/// Returns 403 Forbidden if the check fails.
async fn check_authz(
    state: &BffState,
    user: &str,
    relation: &str,
    object: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let user_key = format!("user:{user}");
    state
        .authz
        .check(&user_key, relation, object)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "authz check failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    ErrorCode::InternalError,
                    "Authorization check failed",
                )),
            )
        })?
        .then_some(())
        .ok_or_else(|| {
            tracing::warn!(
                user = user,
                relation = relation,
                object = object,
                "authz: permission denied"
            );
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse::new(
                    ErrorCode::Forbidden,
                    format!("Permission denied: user {user} cannot {relation} {object}"),
                )),
            )
        })
}

/// Build a boxed CounterService from the BFF state.
/// Abstracts over embedded and remote database backends.
fn build_service(
    state: &BffState,
) -> Result<CounterServiceHandle, (StatusCode, Json<ErrorResponse>)> {
    state.counter_service().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                ErrorCode::InternalError,
                "Embedded database not initialized",
            )),
        )
    })
}

fn extract_request_context(
    request_context: Option<Extension<RequestContext>>,
) -> Result<RequestContext, (StatusCode, Json<ErrorResponse>)> {
    request_context
        .map(|Extension(context)| context)
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

fn build_command_context(request_context: RequestContext) -> CounterCommandContext {
    CounterCommandContext {
        correlation_id: request_context.request_id.clone(),
        causation_id: request_context.request_id.clone(),
        actor: Some(request_context.actor.clone()),
        trace_id: request_context.trace_id.clone(),
        span_id: request_context.span_id.clone(),
    }
}

async fn resolve_tenant_id(
    state: &BffState,
    request_context: &RequestContext,
) -> Result<kernel::TenantId, (StatusCode, Json<ErrorResponse>)> {
    let user_sub = &request_context.user_sub;
    let binding_repo = state.user_tenant_repository().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                ErrorCode::InternalError,
                "Database not initialized",
            )),
        )
    })?;
    let tenant_id = binding_repo
        .find_user_tenant(user_sub)
        .await
        .map_err(map_tenant_resolution_error)?
        .map(|binding| binding.tenant_id);

    let resolved = tenant_id.map(kernel::TenantId).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                ErrorCode::Unauthorized,
                "No tenant binding found for authenticated user",
            )),
        )
    })?;

    if let Some(claim_tenant_id) = request_context.tenant_id.as_deref()
        && claim_tenant_id != resolved.as_str()
    {
        tracing::warn!(
            user_sub = %request_context.user_sub,
            claim_tenant_id,
            resolved_tenant_id = %resolved,
            "tenant claim does not match persisted tenant binding"
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                ErrorCode::Forbidden,
                "Tenant claim does not match authenticated user binding",
            )),
        ));
    }

    Ok(resolved)
}

fn map_tenant_resolution_error(
    error: user_service::domain::error::UserError,
) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    if message.contains("no such table: user_tenant") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                ErrorCode::Unauthorized,
                "No tenant binding found for authenticated user",
            )),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            ErrorCode::DatabaseError,
            format!("Failed to resolve tenant binding: {message}"),
        )),
    )
}

/// Map CounterError to HTTP status code and ErrorResponse.
fn map_counter_error(err: CounterError) -> (StatusCode, Json<ErrorResponse>) {
    match err {
        CounterError::CasConflict | CounterError::CasConflictWithDetails { .. } => (
            StatusCode::CONFLICT,
            Json(ErrorResponse::new(
                ErrorCode::Conflict,
                "Counter was modified concurrently",
            )),
        ),
        CounterError::NotFound(msg) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(ErrorCode::NotFound, &msg)),
        ),
        CounterError::Database(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                ErrorCode::DatabaseError,
                format!("Database error: {}", e),
            )),
        ),
    }
}
