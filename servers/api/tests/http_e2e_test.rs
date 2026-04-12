//! HTTP End-to-End tests for runtime_server.
//!
//! Uses axum::Router + tower::ServiceExt::oneshot to simulate real HTTP requests
//! without needing a running server. SurrealDB runs in-memory for isolation.

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use moka::future::Cache;
use runtime_server::config::{CloudDbProvider, Config};
use runtime_server::create_router;
use runtime_server::routes;
use runtime_server::state::AppState;
use storage_turso::EmbeddedTurso;
use tower::ServiceExt;

async fn build_state_with_embedded_db(db_path: &std::path::Path) -> AppState {
    let embedded_db = EmbeddedTurso::new(db_path.to_str().expect("non-utf8 path"))
        .await
        .expect("Failed to initialize embedded Turso");
    storage_turso::embedded::run_tenant_migrations(&embedded_db)
        .await
        .expect("Failed to run tenant migrations");
    domain::ports::lib_sql::LibSqlPort::execute(
        &embedded_db,
        "CREATE TABLE IF NOT EXISTS tenant (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
        vec![],
    )
    .await
    .expect("Failed to ensure tenant table");
    domain::ports::lib_sql::LibSqlPort::execute(
        &embedded_db,
        "CREATE TABLE IF NOT EXISTS user_tenant (id TEXT PRIMARY KEY, user_sub TEXT NOT NULL UNIQUE, tenant_id TEXT NOT NULL REFERENCES tenant(id), role TEXT NOT NULL DEFAULT 'member', joined_at TEXT NOT NULL DEFAULT (datetime('now')))",
        vec![],
    )
    .await
    .expect("Failed to ensure user_tenant table");
    domain::ports::lib_sql::LibSqlPort::execute(
        &embedded_db,
        "CREATE INDEX IF NOT EXISTS idx_user_tenant_tenant_id ON user_tenant(tenant_id)",
        vec![],
    )
    .await
    .expect("Failed to ensure tenant index");
    domain::ports::lib_sql::LibSqlPort::execute(
        &embedded_db,
        counter_service::application::COUNTER_MIGRATION,
        vec![],
    )
    .await
    .expect("Failed to run counter migration");

    for migration in agent_service::application::migrations::AGENT_MIGRATIONS {
        domain::ports::lib_sql::LibSqlPort::execute(&embedded_db, migration, vec![])
            .await
            .expect("Failed to run agent migration");
    }

    let cache: Cache<String, String> = Cache::builder().max_capacity(10_000).build();
    let http_client = reqwest::Client::new();
    let mut config = Config::default();
    config.database.provider = CloudDbProvider::Turso;
    config.database.url = db_path.to_string_lossy().to_string();

    AppState {
        db: surrealdb::Surreal::<surrealdb::engine::any::Any>::init(),
        cache,
        http_client,
        config,
        turso_db: None,
        db_provider: CloudDbProvider::Turso,
        embedded_db: Some(embedded_db),
    }
}

/// Create a test AppState with a file-based embedded Turso instance.
async fn make_test_state() -> AppState {
    let temp_dir =
        std::env::temp_dir().join(format!("runtime_server_http_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let db_path = temp_dir.join("runtime_server.db");
    build_state_with_embedded_db(&db_path).await
}

/// Create a test AppState for tenant-route scenarios.
async fn make_test_state_file() -> AppState {
    let temp_dir =
        std::env::temp_dir().join(format!("runtime_server_http_file_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let db_path = temp_dir.join("runtime_server.db");
    build_state_with_embedded_db(&db_path).await
}

/// Create a test AppState with file-based embedded Turso.
/// Used for counter route end-to-end tests.
async fn make_test_state_with_counter() -> AppState {
    let temp_dir = std::env::temp_dir().join(format!("counter_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let db_path = temp_dir.join("counter.db");
    build_state_with_embedded_db(&db_path).await
}

/// Extract JSON body from an axum Response.
async fn body_to_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(
        !bytes.is_empty(),
        "Response body is empty — handler likely returned an error status"
    );
    serde_json::from_slice(&bytes).unwrap()
}

/// Create a test JWT token for the given sub.
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state_file().await;
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
    let state = make_test_state_file().await;
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

// ─── Cross-Tenant Isolation ──────────────────────────────────────────────────

#[tokio::test]
async fn two_users_get_different_tenants() {
    let state = make_test_state_file().await;
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

// ─── Request Validation ──────────────────────────────────────────────────────

#[tokio::test]
async fn tenant_init_rejects_empty_user_sub() {
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state().await;
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
    let state = make_test_state_file().await;
    let app = create_router(state);
    let token = make_test_token("attacker");

    // Attempt SQL injection via user_sub
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

    // Should succeed (parameterized query treats injection as literal string)
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(body.get("created").unwrap(), true);

    // Verify the tenant table still exists and is queryable
    let response2 = app
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

    assert_eq!(response2.status(), StatusCode::OK);
    let body2: serde_json::Value = body_to_json(response2).await;
    assert_eq!(body2.get("created").unwrap(), false);
}

#[tokio::test]
async fn tenant_init_with_special_characters_in_user_name() {
    let state = make_test_state_file().await;
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
    let state = make_test_state().await;
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
async fn tenant_init_with_expired_jwt() {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    #[derive(serde::Serialize)]
    struct ExpiredClaims {
        sub: String,
        exp: usize,
    }
    let expired_token = encode(
        &Header::new(Algorithm::HS256),
        &ExpiredClaims {
            sub: "expired-user".into(),
            exp: 1, // Expired in 1970
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap();

    let state = make_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tenant/init")
                .method(http::Method::POST)
                .header(
                    http::header::AUTHORIZATION,
                    format!("Bearer {expired_token}"),
                )
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"user_sub":"expired-user","user_name":"Expired"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // The insecure_decode doesn't check exp, so it should still work
    // (This documents current v1 behavior — v2 should reject expired tokens)
    assert_eq!(response.status(), StatusCode::OK);
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

    let state = make_test_state().await;
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
    let state = make_test_state_file().await;
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

// ─── Counter Tenant Isolation ────────────────────────────────────────────────

#[tokio::test]
async fn counter_routes_return_401_when_tenant_context_missing() {
    let state = make_test_state_with_counter().await;
    let app = routes::counter::router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/counter/value")
                .method(http::Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = body_to_json(response).await;
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("Missing tenant context")
    );
}

#[tokio::test]
async fn counter_mutation_isolated_between_two_tenants() {
    let state = make_test_state_with_counter().await;
    let app = create_router(state);

    let token_a = make_test_token("counter-tenant-a");
    let token_b = make_test_token("counter-tenant-b");

    // deterministic baseline
    for token in [&token_a, &token_b] {
        let reset_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/counter/reset")
                    .method(http::Method::POST)
                    .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(reset_resp.status(), StatusCode::OK);
    }

    let inc_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/counter/increment")
                .method(http::Method::POST)
                .header(http::header::AUTHORIZATION, format!("Bearer {token_a}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inc_resp.status(), StatusCode::OK);
    let inc_body: serde_json::Value = body_to_json(inc_resp).await;
    assert_eq!(inc_body.get("value").and_then(|v| v.as_i64()), Some(1));

    let read_a = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/counter/value")
                .method(http::Method::GET)
                .header(http::header::AUTHORIZATION, format!("Bearer {token_a}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read_a.status(), StatusCode::OK);
    let body_a: serde_json::Value = body_to_json(read_a).await;

    let read_b = app
        .oneshot(
            Request::builder()
                .uri("/api/counter/value")
                .method(http::Method::GET)
                .header(http::header::AUTHORIZATION, format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read_b.status(), StatusCode::OK);
    let body_b: serde_json::Value = body_to_json(read_b).await;

    let value_a = body_a.get("value").and_then(|v| v.as_i64());
    let value_b = body_b.get("value").and_then(|v| v.as_i64());
    assert_eq!(
        value_a,
        Some(1),
        "tenant-A expected value 1 after increment"
    );
    assert_eq!(
        value_b,
        Some(0),
        "tenant-B leaked value after tenant-A mutation; expected 0, got {:?}",
        value_b
    );
}

#[tokio::test]
async fn counter_isolation_repeated_run_stays_stable() {
    let state = make_test_state_with_counter().await;
    let app = create_router(state);

    let token_a = make_test_token("counter-repeat-a");
    let token_b = make_test_token("counter-repeat-b");

    for run in 1..=2 {
        for token in [&token_a, &token_b] {
            let reset_resp = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/counter/reset")
                        .method(http::Method::POST)
                        .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                reset_resp.status(),
                StatusCode::OK,
                "run-{run} reset should succeed"
            );
        }

        let inc_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/counter/increment")
                    .method(http::Method::POST)
                    .header(http::header::AUTHORIZATION, format!("Bearer {token_a}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            inc_resp.status(),
            StatusCode::OK,
            "run-{run} increment failed"
        );

        let read_a = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/counter/value")
                    .method(http::Method::GET)
                    .header(http::header::AUTHORIZATION, format!("Bearer {token_a}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_a: serde_json::Value = body_to_json(read_a).await;

        let read_b = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/counter/value")
                    .method(http::Method::GET)
                    .header(http::header::AUTHORIZATION, format!("Bearer {token_b}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_b: serde_json::Value = body_to_json(read_b).await;

        assert_eq!(
            body_a.get("value").and_then(|v| v.as_i64()),
            Some(1),
            "run-{run} tenant-A value mismatch"
        );
        assert_eq!(
            body_b.get("value").and_then(|v| v.as_i64()),
            Some(0),
            "run-{run} tenant-B value mismatch"
        );
    }
}
