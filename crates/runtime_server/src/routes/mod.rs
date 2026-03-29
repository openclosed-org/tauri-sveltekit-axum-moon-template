//! Route module barrel — all feature route modules exported here.

pub mod health;
pub mod tenant;

use crate::state::AppState;
use axum::Router;

/// Public routes — no authentication required.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new().merge(health::router())
}

/// API routes — tenant-scoped, require JWT authentication.
/// Tenant middleware is applied as a route_layer in create_router().
pub fn api_router() -> Router<AppState> {
    Router::<AppState>::new().merge(tenant::router())
}
