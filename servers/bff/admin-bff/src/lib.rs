use axum::Router;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, SetRequestIdLayer, RequestId},
    timeout::TimeoutLayer,
};
use std::time::Duration;

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

use crate::state::AdminBffState;

/// Generate UUID v7 request IDs
#[derive(Clone)]
struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let id = uuid::Uuid::now_v7().to_string();
        Some(RequestId::new(axum::http::HeaderValue::from_str(&id).ok()?))
    }
}

/// Create the admin BFF router with all middleware
pub fn create_router(state: AdminBffState) -> Router {
    let cors = build_cors_layer();

    // Public routes — no auth required
    let public_routes = routes::health::router();

    // Admin API routes — tenant-scoped, require JWT auth
    let admin_api_routes = Router::new()
        .merge(routes::admin::router())
        .merge(routes::tenant::router())
        .merge(routes::metrics::router())
        .route_layer(axum::middleware::from_fn(middleware::tenant::admin_tenant_middleware));

    Router::new()
        .merge(public_routes)
        .merge(admin_api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            MakeRequestUuidV7,
        ))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .with_state(state)
}

fn build_cors_layer() -> CorsLayer {
    CorsLayer::very_permissive()
}
