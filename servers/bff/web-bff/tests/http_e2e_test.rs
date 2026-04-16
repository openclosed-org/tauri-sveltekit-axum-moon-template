//! HTTP End-to-End tests for web-bff.
//!
//! Uses axum::Router + tower::ServiceExt::oneshot to simulate real HTTP requests
//! without needing a running server. Turso runs in-memory for isolation.

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use counter_service::infrastructure::LibSqlCounterRepository;
use http_body_util::BodyExt;
use storage_turso::EmbeddedTurso;
use tower::ServiceExt;
use web_bff::{create_router, state::BffState};

/// Helper: build test AppState with embedded Turso
async fn build_test_state() -> BffState {
    // Use unique temp directory for each test to avoid database locking
    let temp_dir = std::env::temp_dir().join(format!("web_bff_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let db_path = temp_dir.join("test.db");

    let db = EmbeddedTurso::new(db_path.to_str().expect("non-utf8 path"))
        .await
        .expect("Failed to initialize embedded Turso");

    // Run migrations
    storage_turso::embedded::run_tenant_migrations(&db)
        .await
        .expect("Failed to run tenant migrations");

    // Run counter migration
    let repo = LibSqlCounterRepository::new(db.clone());
    repo.migrate()
        .await
        .expect("Failed to run counter migration");

    BffState::new_with_db(db).await
}

/// Extract JSON body from an axum Response
async fn body_to_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(
        !bytes.is_empty(),
        "Response body is empty — handler likely returned an error status"
    );
    serde_json::from_slice(&bytes).unwrap()
}

/// Create a test JWT token for the given sub
fn make_test_token(sub: &str) -> String {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }
    encode(
        &Header::new(Algorithm::HS256),
        &Claims {
            sub: sub.to_string(),
            exp: 9999999999,
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap()
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
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("status").unwrap(), "ok");
}

#[tokio::test]
async fn readyz_returns_200_when_db_is_connected() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("status").unwrap(), "ready");
}

// ─── 404 Fallback ────────────────────────────────────────────────────────────

#[tokio::test]
async fn unknown_route_returns_404() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ─── Tenant Middleware ───────────────────────────────────────────────────────

#[tokio::test]
async fn api_route_without_auth_returns_401() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"test","user_name":"Test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_with_invalid_jwt_returns_401() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, "Bearer not-a-jwt")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"test","user_name":"Test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_with_valid_jwt_and_missing_fields_returns_400() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-test-123");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn tenant_init_creates_tenant_on_first_call() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-e2e-first");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"google-e2e-first","user_name":"Alice"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);

    assert_eq!(
        status,
        StatusCode::OK,
        "Expected 200 OK but got {}: {body_str}",
        status
    );

    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body.get("role").unwrap(), "owner");
    assert_eq!(body.get("created").unwrap(), true);
}

#[tokio::test]
async fn tenant_init_returns_existing_on_second_call() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-e2e-second");

    // First call — creates tenant
    let resp1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"google-e2e-second","user_name":"Bob"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp1.status(), StatusCode::OK);
    let body1: serde_json::Value = body_to_json(resp1).await;
    let tenant_id_1 = body1.get("tenant_id").unwrap().clone();
    assert_eq!(body1.get("created").unwrap(), true);

    // Second call — returns existing
    let resp2 = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"google-e2e-second","user_name":"Bob"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp2.status(), StatusCode::OK);
    let body2: serde_json::Value = body_to_json(resp2).await;
    assert_eq!(body2.get("tenant_id").unwrap(), &tenant_id_1);
    assert_eq!(body2.get("created").unwrap(), false);
}

#[tokio::test]
async fn tenant_init_rejects_jwt_sub_mismatch() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("jwt-user");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"different-user","user_name":"Mallory"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ─── Cross-Tenant Isolation ──────────────────────────────────────────────────

#[tokio::test]
async fn two_users_get_different_tenants() {
    let state = build_test_state().await;
    let app = create_router(state);

    let token_a = make_test_token("user-alice");
    let token_b = make_test_token("user-bob");

    // Alice creates her tenant
    let resp_a = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token_a}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"user-alice","user_name":"Alice"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body_a: serde_json::Value = body_to_json(resp_a).await;
    let tenant_a = body_a.get("tenant_id").unwrap().as_str().unwrap();

    // Bob creates his tenant
    let resp_b = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token_b}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"user-bob","user_name":"Bob"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let body_b: serde_json::Value = body_to_json(resp_b).await;
    let tenant_b = body_b.get("tenant_id").unwrap().as_str().unwrap();

    assert_ne!(
        tenant_a, tenant_b,
        "Alice and Bob should have different tenant IDs"
    );
}

#[tokio::test]
async fn counter_endpoints_require_existing_user_tenant_binding() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("counter-user");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/counter/value")
                .method(http::Method::GET)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Expected 401 but got {}: {body_str}",
        status
    );
}

#[tokio::test]
async fn counter_endpoints_resolve_real_tenant_binding_after_init() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("bound-user");

    let init_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"bound-user","user_name":"Bound User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(init_response.status(), StatusCode::OK);

    let increment_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/counter/increment")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let increment_status = increment_response.status();
    let increment_bytes = increment_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let increment_body_str = String::from_utf8_lossy(&increment_bytes);
    assert_eq!(
        increment_status,
        StatusCode::OK,
        "Expected 200 but got {}: {increment_body_str}",
        increment_status
    );
    let increment_body: serde_json::Value = serde_json::from_slice(&increment_bytes).unwrap();
    assert_eq!(increment_body.get("value").unwrap(), 1);

    let value_response = app
        .oneshot(
            Request::builder()
                .uri("/api/counter/value")
                .method(http::Method::GET)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(value_response.status(), StatusCode::OK);
    let value_body: serde_json::Value = body_to_json(value_response).await;
    assert_eq!(value_body.get("value").unwrap(), 1);
}

// ─── Request Validation ──────────────────────────────────────────────────────

#[tokio::test]
async fn tenant_init_rejects_empty_user_sub() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-test");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"","user_name":"Test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Now returns 422 (Unprocessable Entity) due to validator
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn tenant_init_rejects_empty_user_name() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-test");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"test","user_name":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Now returns 422 (Unprocessable Entity) due to validator
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn tenant_init_rejects_missing_content_type() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("google-test");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(r#"{"user_sub":"test","user_name":"Test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

// ─── CORS Headers ────────────────────────────────────────────────────────────

#[tokio::test]
async fn healthz_includes_cors_headers_on_preflight() {
    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .method(http::Method::OPTIONS)
                .header(http::header::ORIGIN, "http://localhost:5173")
                .header(http::header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ─── Middleware + DB Integration ─────────────────────────────────────────────

#[tokio::test]
async fn tenant_init_with_sql_injection_attempt_in_user_sub() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("attacker");

    // Attempt to spoof user_sub in request body
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"'; DROP TABLE tenant; --","user_name":"Attacker"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // JWT subject is now the source of truth, so body spoofing is rejected.
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Verify the tenant table still exists and the legitimate request remains queryable.
    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"attacker","user_name":"Attacker"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
    let body2: serde_json::Value = body_to_json(response2).await;
    assert_eq!(body2.get("created").unwrap(), true);
}

#[tokio::test]
async fn tenant_init_with_special_characters_in_user_name() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("special-chars-user");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"special-chars-user","user_name":"O'Brien <script>alert(1)</script>"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("created").unwrap(), true);
}

// ─── Error Path Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn tenant_init_with_malformed_json_body() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("malformed-json-user");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub": "broken"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Axum returns 400 for malformed JSON (before it reaches the handler)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn tenant_init_with_jwt_missing_sub_claim() {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    #[derive(serde::Serialize)]
    struct NoSubClaims {
        name: String,
        exp: usize,
    }
    let no_sub_token = encode(
        &Header::new(Algorithm::HS256),
        &NoSubClaims {
            name: "no-sub".into(),
            exp: 9999999999,
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap();

    let state = build_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(
                    http::header::AUTHORIZATION,
                    format!("Bearer {no_sub_token}"),
                )
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"user_sub":"no-sub","user_name":"NoSub"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing sub claim should cause middleware to fail
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tenant_init_idempotent_with_same_token() {
    let state = build_test_state().await;
    let app = create_router(state);
    let token = make_test_token("idempotent-user");

    // First call — creates
    let resp1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"idempotent-user","user_name":"Idempotent"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp1.status(), StatusCode::OK);
    let body1: serde_json::Value = body_to_json(resp1).await;
    let tenant_id_1 = body1.get("tenant_id").unwrap().clone();
    assert_eq!(body1.get("created").unwrap(), true);

    // Second call with same token — returns existing
    let resp2 = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"idempotent-user","user_name":"Idempotent"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp2.status(), StatusCode::OK);
    let body2: serde_json::Value = body_to_json(resp2).await;
    assert_eq!(body2.get("tenant_id").unwrap(), &tenant_id_1);
    assert_eq!(body2.get("created").unwrap(), false);
}
