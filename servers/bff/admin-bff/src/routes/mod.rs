pub mod admin;
pub mod health;
pub mod metrics;
pub mod tenant;

use axum::Router;
use crate::state::AdminBffState;

/// Merge all admin BFF route modules
pub fn merge_all() -> Router<AdminBffState> {
    Router::new()
        .merge(health::router())
        .merge(admin::router())
        .merge(tenant::router())
        .merge(metrics::router())
}
