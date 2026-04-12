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
pub mod routes;
pub mod state;

use axum::{Router, middleware as axum_mw};
use std::time::Duration;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use contracts_api::{
    HealthResponse, InitTenantRequest, InitTenantResponse, CounterResponse, ErrorResponse,
    ChatMessage, ToolCall, AgentConfig, ConversationSummary, ConversationDetail,
    CreateConversationRequest, ChatRequest, AdminDashboardStats,
};
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
        routes::counter::increment,
        routes::counter::decrement,
        routes::counter::reset,
        routes::counter::get_value,
        routes::admin::get_dashboard_stats,
        routes::agent::list_conversations,
        routes::agent::create_conversation,
        routes::agent::get_messages,
        routes::agent::chat_handler,
        routes::settings::get_settings,
        routes::settings::update_settings,
    ),
    components(schemas(
        HealthResponse,
        InitTenantRequest,
        InitTenantResponse,
        CounterResponse,
        ErrorResponse,
        ChatMessage,
        ToolCall,
        AgentConfig,
        ConversationSummary,
        ConversationDetail,
        CreateConversationRequest,
        ChatRequest,
        AdminDashboardStats,
    )),
    tags(
        (name = "health", description = "Health check and readiness probes"),
        (name = "tenant", description = "Tenant lifecycle operations"),
        (name = "counter", description = "Counter operations"),
        (name = "admin", description = "Admin dashboard operations"),
        (name = "agent", description = "Agent chat and conversation management"),
        (name = "settings", description = "User settings and preferences"),
        (name = "user", description = "User profile and tenant memberships"),
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development server"),
    ),
    security(
        ("tenant_auth" = []),
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
    // CORS: if cors_allowed_origins is empty → permissive (dev mode).
    // If set → enforce explicit allowlist with credentials.
    let cors = build_cors_layer(&state.config.server.cors_allowed_origins);
    // Tenant-scoped routes — middleware extracts TenantId from JWT
    // jwt_secret is injected as Extension<String> so tenant_middleware can read it
    //
    // Layer order (outermost → innermost):
    // 1. Extension(jwt_secret) — adds String to request extensions
    // 2. tenant_middleware — reads jwt_secret from extensions
    let api_routes = routes::api_router()
        .route_layer(axum_mw::from_fn(middleware::tenant::tenant_middleware))
        .route_layer(axum::Extension(state.config.auth.jwt_secret.clone()));

    // Public routes — health checks, no auth required
    let public_routes = routes::health::router();

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
        .layer(cors)
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

/// Build CORS layer from allowed origins list.
///
/// - Empty list → `CorsLayer::permissive()` (dev mode fallback)
/// - Non-empty → explicit allowlist with credentials, methods, and headers
fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    if allowed_origins.is_empty() {
        tracing::info!("CORS: no allowed origins configured — using permissive mode (dev)");
        return CorsLayer::permissive();
    }

    let origins: Vec<axum::http::HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    tracing::info!(count = origins.len(), "CORS: enforcing origin allowlist");

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, header},
    };
    use futures_util::FutureExt;
    use tower::ServiceExt;

    #[test]
    fn build_cors_layer_empty_origins_returns_permissive() {
        let layer = build_cors_layer(&[]);
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .layer(layer);

        let request = Request::builder()
            .header(header::ORIGIN, "https://any-origin.com")
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).now_or_never().unwrap().unwrap();
        // Permissive mode sends ACAO: *
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "*"
        );
    }

    #[test]
    fn build_cors_layer_with_origins_allows_matching_origin() {
        let origins = vec![
            "https://example.com".to_string(),
            "https://app.example.com".to_string(),
        ];
        let layer = build_cors_layer(&origins);

        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .layer(layer);

        let request = Request::builder()
            .header(header::ORIGIN, "https://example.com")
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).now_or_never().unwrap().unwrap();
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }

    #[test]
    fn build_cors_layer_with_origins_rejects_non_matching_origin() {
        let origins = vec!["https://example.com".to_string()];
        let layer = build_cors_layer(&origins);

        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .layer(layer);

        let request = Request::builder()
            .header(header::ORIGIN, "https://evil.com")
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).now_or_never().unwrap().unwrap();
        // Non-matching origin should NOT have ACAO header
        assert!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    #[test]
    fn build_cors_layer_preflight_allows_configured_methods_and_headers() {
        let origins = vec!["https://example.com".to_string()];
        let layer = build_cors_layer(&origins);

        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .layer(layer);

        let request = Request::builder()
            .header(header::ORIGIN, "https://example.com")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
            .method("OPTIONS")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).now_or_never().unwrap().unwrap();
        // Preflight response should include CORS headers
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        // Allow-Methods should include POST
        let allow_methods = response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(allow_methods.contains("POST"));
    }
}
