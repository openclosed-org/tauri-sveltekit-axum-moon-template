//! Counter REST API handlers — web-bff version.
//!
//! These handlers use the counter-service implementation via its repository.
//! All responses use contract DTOs from `contracts_api` and `contracts_errors`.

use axum::{
    Json,
    extract::{Extension, State},
    http::HeaderMap,
};
use contracts_api::CounterResponse;
use contracts_errors::ErrorResponse;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::application::counter as counter_use_case;
use crate::error::{BffError, BffResult};
use crate::http::idempotency_key;
use crate::request_context::RequestContext;
use crate::state::BffState;

pub fn openapi_router() -> OpenApiRouter<BffState> {
    OpenApiRouter::new()
        .routes(routes!(increment))
        .routes(routes!(decrement))
        .routes(routes!(reset))
        .routes(routes!(get_value))
}

/// Increment the tenant's counter value.
#[utoipa::path(
    post,
    path = "/api/counter/increment",
    tag = "counter",
    security(("bearer_auth" = [])),
    params(("Idempotency-Key" = Option<String>, Header, description = "Optional idempotency key for safe retry/replay of the same mutation")),
    responses(
        (status = 200, description = "Counter incremented successfully", body = CounterResponse, content_type = "application/json"),
        (status = 400, description = "Bad request — invalid Idempotency-Key header", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 403, description = "Forbidden — user cannot mutate this counter", body = ErrorResponse),
        (status = 409, description = "Conflict — concurrent modification or idempotency key reuse with a different mutation", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn increment(
    State(state): State<BffState>,
    headers: HeaderMap,
    request_context: Option<Extension<RequestContext>>,
) -> BffResult<Json<CounterResponse>> {
    let request_context = extract_request_context(request_context)?;
    let idempotency_key = idempotency_key(&headers)?;
    let value =
        counter_use_case::increment(&state, &request_context, idempotency_key.as_deref()).await?;

    Ok(Json(CounterResponse { value }))
}

/// Decrement the tenant's counter value.
#[utoipa::path(
    post,
    path = "/api/counter/decrement",
    tag = "counter",
    security(("bearer_auth" = [])),
    params(("Idempotency-Key" = Option<String>, Header, description = "Optional idempotency key for safe retry/replay of the same mutation")),
    responses(
        (status = 200, description = "Counter decremented successfully", body = CounterResponse, content_type = "application/json"),
        (status = 400, description = "Bad request — invalid Idempotency-Key header", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 403, description = "Forbidden — user cannot mutate this counter", body = ErrorResponse),
        (status = 409, description = "Conflict — concurrent modification or idempotency key reuse with a different mutation", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn decrement(
    State(state): State<BffState>,
    headers: HeaderMap,
    request_context: Option<Extension<RequestContext>>,
) -> BffResult<Json<CounterResponse>> {
    let request_context = extract_request_context(request_context)?;
    let idempotency_key = idempotency_key(&headers)?;
    let value =
        counter_use_case::decrement(&state, &request_context, idempotency_key.as_deref()).await?;

    Ok(Json(CounterResponse { value }))
}

/// Reset the tenant's counter value to zero.
#[utoipa::path(
    post,
    path = "/api/counter/reset",
    tag = "counter",
    security(("bearer_auth" = [])),
    params(("Idempotency-Key" = Option<String>, Header, description = "Optional idempotency key for safe retry/replay of the same mutation")),
    responses(
        (status = 200, description = "Counter reset successfully", body = CounterResponse, content_type = "application/json"),
        (status = 400, description = "Bad request — invalid Idempotency-Key header", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing authenticated request context", body = ErrorResponse),
        (status = 403, description = "Forbidden — user cannot mutate this counter", body = ErrorResponse),
        (status = 409, description = "Conflict — concurrent modification or idempotency key reuse with a different mutation", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn reset(
    State(state): State<BffState>,
    headers: HeaderMap,
    request_context: Option<Extension<RequestContext>>,
) -> BffResult<Json<CounterResponse>> {
    let request_context = extract_request_context(request_context)?;
    let idempotency_key = idempotency_key(&headers)?;
    let value =
        counter_use_case::reset(&state, &request_context, idempotency_key.as_deref()).await?;

    Ok(Json(CounterResponse { value }))
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
        (status = 403, description = "Forbidden — user cannot read this counter", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn get_value(
    State(state): State<BffState>,
    request_context: Option<Extension<RequestContext>>,
) -> BffResult<Json<CounterResponse>> {
    let request_context = extract_request_context(request_context)?;
    let value = counter_use_case::get_value(&state, &request_context).await?;

    Ok(Json(CounterResponse { value }))
}

// ── Helpers ──────────────────────────────────────────────────

fn extract_request_context(
    request_context: Option<Extension<RequestContext>>,
) -> BffResult<RequestContext> {
    request_context
        .map(|Extension(context)| context)
        .ok_or_else(|| BffError::Unauthorized("Missing authenticated request context".to_string()))
}
