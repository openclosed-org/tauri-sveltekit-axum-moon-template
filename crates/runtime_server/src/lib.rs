//! Runtime Server — Axum HTTP adapter layer.
//!
//! This crate is the outermost layer in the Clean Architecture stack.
//! It owns HTTP concerns: routing, middleware, serialization boundaries.
//! Business logic lives in `domain` and `application` crates.

pub mod routes;

use axum::Router;
use std::time::Duration;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

/// Build the root router with all middleware layers applied.
///
/// Middleware order (outermost → innermost):
/// 1. TraceLayer — request/response logging
/// 2. TimeoutLayer — 30s default timeout
/// 3. CorsLayer — permissive for dev (tighten in production)
/// 4. Routes — health check + future API routes
pub fn create_router() -> Router {
    routes::router()
        .layer(CorsLayer::permissive())     // D-01: dev permissive CORS
        .layer(TraceLayer::new_for_http())   // D-01: request tracing
        .layer(TimeoutLayer::with_status_code(Duration::from_secs(30), axum::http::StatusCode::REQUEST_TIMEOUT))
    // D-01: 30s timeout
}
