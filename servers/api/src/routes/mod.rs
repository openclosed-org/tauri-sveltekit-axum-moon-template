//! Route module barrel — all feature route modules exported here.

pub mod admin;
pub mod agent;
pub mod counter;
pub mod health;
pub mod settings;
pub mod tenant;
pub mod user;

use crate::state::AppState;
use axum::Router;

/// Public routes — no authentication required.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new().merge(health::router())
}

/// API routes — tenant-scoped, require JWT authentication.
/// Tenant middleware is applied as a route_layer in create_router().
pub fn api_router() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(tenant::router())
        .merge(counter::router())
        .merge(admin::router())
        .merge(agent::router())
        .merge(settings::router())
        .merge(user::router())
}
