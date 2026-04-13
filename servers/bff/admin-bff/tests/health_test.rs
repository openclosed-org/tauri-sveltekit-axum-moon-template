//! HTTP health endpoint tests for admin-bff.
//!
//! Uses axum::Router + tower::ServiceExt::oneshot to simulate real HTTP requests
//! without needing a running server.

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use admin_bff::{create_router, state::AdminBffState, config::Config};
use tower::ServiceExt;

/// Helper: collect body bytes and deserialize to JSON Value
async fn body_to_json(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Helper: build minimal test state
async fn build_test_state() -> AdminBffState {
    // Create minimal config — use defaults, no database for health tests
    let config = Config::default();

    AdminBffState::new_with_config(&config)
        .await
        .expect("Failed to build test state")
}

// ─── Health Endpoints ────────────────────────────────────────────────────────

#[tokio::test]
async fn healthz_returns_200_with_ok_status() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .method(http::Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("status").unwrap(), "ok");
    assert!(body.get("version").is_some());
}

#[tokio::test]
async fn readyz_returns_200_with_ok_status() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .method(http::Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("status").unwrap(), "ok");
    assert!(body.get("version").is_some());
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .method(http::Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
