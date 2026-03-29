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

use axum::Router;
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use state::AppState;

/// Build the root router with shared state and middleware layers.
///
/// Middleware order (outermost → innermost):
/// 1. TraceLayer — request/response logging
/// 2. TimeoutLayer — 30s default timeout
/// 3. CorsLayer — permissive for dev (tighten in production)
/// 4. Routes — health check + future API routes
pub fn create_router(state: AppState) -> Router {
    routes::router()
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
}
