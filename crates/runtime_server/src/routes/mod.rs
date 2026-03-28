//! Route module barrel — all feature route modules exported here.

pub mod health;

use axum::Router;

/// Merge all route modules into a single router.
pub fn router() -> Router {
    Router::new().merge(health::router())
}
