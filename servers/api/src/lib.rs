//! Runtime Server — Axum HTTP adapter layer.
//!
//! This crate is the outermost layer in the Clean Architecture stack.
//! It owns HTTP concerns: routing, middleware, serialization boundaries.
//! Business logic lives in `domain` and `application` crates.

pub mod config;
pub mod error;
#[cfg(feature = "http3")]
pub mod h3_server;
pub mod middleware;
pub mod ports;
pub mod routes;
pub mod state;

use axum::{Router, middleware as axum_mw};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use contracts_api::{HealthResponse, InitTenantRequest, InitTenantResponse};
use state::AppState;

/// Generate time-ordered UUID v7 request IDs.
///
/// UUID v7 embeds a Unix timestamp in the first 48 bits, making IDs
/// sortable by creation time — useful for log correlation and database indexing.
#[derive(Clone)]
struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let id = uuid::Uuid::now_v7().to_string();
        Some(RequestId::new(axum::http::HeaderValue::from_str(&id).ok()?))
    }
}

/// OpenAPI documentation root.
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::healthz,
        routes::health::readyz,
        routes::tenant::init_tenant,
    ),
    components(schemas(
        HealthResponse,
        InitTenantRequest,
        InitTenantResponse,
    )),
    tags(
        (name = "health", description = "Health check and readiness probes"),
        (name = "tenant", description = "Tenant lifecycle operations"),
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development server"),
    ),
)]
struct ApiDoc;

/// Build the root router with shared state and middleware layers.
///
/// Middleware order (outermost → innermost):
/// 1. SetRequestId — assign x-request-id to every request
/// 2. TraceLayer — request/response logging with span context
/// 3. PropagateRequestId — echo x-request-id back in response
/// 4. TimeoutLayer — 30s default timeout
/// 5. CorsLayer — permissive for dev (tighten in production)
/// 6. Tenant middleware — JWT extraction (API routes only via route_layer)
/// 7. Routes — health check (public) + API routes (tenant-scoped)
pub fn create_router(state: AppState) -> Router {
    // Tenant-scoped routes — middleware extracts TenantId from JWT
    let api_routes =
        routes::api_router().route_layer(axum_mw::from_fn(middleware::tenant::tenant_middleware));

    // Public routes — health checks, no auth required
    let public_routes = routes::health::router();

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<_>| {
                let request_id = req
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("-");
                tracing::info_span!(
                    "http_request",
                    method = %req.method(),
                    uri = %req.uri(),
                    request_id,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            MakeRequestUuidV7,
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
}
