use axum::Router;
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

use crate::state::AdminBffState;

/// Unified OpenAPI documentation for admin-bff.
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::admin::get_dashboard_stats,
        routes::tenant::list_tenants,
        routes::metrics::get_system_metrics,
        routes::health::healthz,
    ),
    components(
        schemas(
            routes::admin::DashboardView,
            routes::tenant::TenantListView,
            routes::tenant::TenantItemView,
            routes::metrics::MetricsView,
        )
    )
)]
pub struct ApiDoc;

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
        .route_layer(axum::middleware::from_fn(
            middleware::tenant::admin_tenant_middleware,
        ));

    let scalar_html: String = Scalar::new(ApiDoc::openapi()).to_html();

    Router::new()
        .merge(public_routes)
        .merge(admin_api_routes)
        .route(
            "/scalar",
            axum::routing::get(move || {
                let html = scalar_html.clone();
                async move { axum::response::Html(html) }
            }),
        )
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
