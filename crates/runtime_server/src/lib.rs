//! Runtime Server — Axum HTTP adapter layer.
//!
//! This crate is the outermost layer in the Clean Architecture stack.
//! It owns HTTP concerns: routing, middleware, serialization boundaries.
//! Business logic lives in `domain` and `application` crates.

pub mod h3_server;
pub mod middleware;
pub mod ports;
pub mod routes;
pub mod state;

use axum::{middleware as axum_mw, Router};
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use state::AppState;

/// Build the root router with shared state and middleware layers.
///
/// Middleware order (outermost → innermost):
/// 1. TraceLayer — request/response logging
/// 2. TimeoutLayer — 30s default timeout
/// 3. CorsLayer — permissive for dev (tighten in production)
/// 4. Tenant middleware — JWT extraction (API routes only via route_layer)
/// 5. Routes — health check (public) + API routes (tenant-scoped)
pub fn create_router(state: AppState) -> Router {
    // Tenant-scoped routes — middleware extracts TenantId from JWT
    let api_routes =
        routes::api_router().route_layer(axum_mw::from_fn(middleware::tenant::tenant_middleware));

    // Public routes — health checks, no auth required
    let public_routes = routes::health::router();

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
}
