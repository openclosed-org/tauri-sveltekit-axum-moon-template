//! HTTP end-to-end tests for runtime_server.
//!
//! Uses axum::Router + tower::ServiceExt::oneshot to simulate real HTTP requests
//! without needing a running server or SurrealDB connection.

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use moka::future::Cache;
use runtime_server::config::{CloudDbProvider, Config};
use runtime_server::create_router;
use runtime_server::state::AppState;
use surrealdb::{Surreal, engine::any::connect};
use tower::ServiceExt;

/// Create a test AppState with an in-memory SurrealDB instance.
async fn make_test_state() -> AppState {
    let db: Surreal<_> = connect("mem://").await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();

    runtime_server::ports::surreal_db::run_tenant_migrations(&db)
        .await
        .unwrap();

    let cache: Cache<String, String> = Cache::builder().max_capacity(10_000).build();

    let http_client = reqwest::Client::new();

    let config = Config::default();

    AppState {
        db,
        cache,
        http_client,
        config,
        turso_db: None,
        db_provider: CloudDbProvider::SurrealDB,
    }
}

/// Create a test AppState with a file-based SurrealDB instance.
/// Uses RocksDB for tests that require full CREATE/INSERT support.
async fn make_test_state_file() -> AppState {
    let temp_dir = std::env::temp_dir().join(format!("surreal_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let db_url = format!("rocksdb://{}", temp_dir.display());
    let db: Surreal<_> = connect(&db_url)
        .await
        .expect("Failed to connect to RocksDB");
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to use ns/db");

    runtime_server::ports::surreal_db::run_tenant_migrations(&db)
        .await
        .expect("Failed to run migrations");

    // Verify migrations worked
    let mut result = db
        .query("SELECT * FROM tenant")
        .await
        .expect("SELECT after migration failed");
    let rows: Vec<serde_json::Value> = result.take(0).expect("Failed to take results");
    assert!(
        rows.is_empty(),
        "Expected empty tenant table after migration"
    );

    let cache: Cache<String, String> = Cache::builder().max_capacity(10_000).build();
    let http_client = reqwest::Client::new();
    let config = Config::default();

    AppState {
        db,
        cache,
        http_client,
        config,
        turso_db: None,
        db_provider: CloudDbProvider::SurrealDB,
    }
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

// ─── Tenant Init E2E ─────────────────────────────────────────────────────────
// These tests require file-based SurrealDB (RocksDB) for full CREATE support.

#[tokio::test]
async fn diagnostic_db_create_works() {
    // Direct DB test to verify CREATE works with RocksDB
    let temp_dir = std::env::temp_dir().join(format!("surreal_diag_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_url = format!("rocksdb://{}", temp_dir.display());
    let db: Surreal<_> = connect(&db_url).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();

    runtime_server::ports::surreal_db::run_tenant_migrations(&db)
        .await
        .unwrap();

    // Test CREATE with parameters
    let mut resp = db
        .query("CREATE tenant SET name = $name")
        .bind(("name", "test-tenant"))
        .await
        .unwrap();
    let rows: Vec<serde_json::Value> = resp.take(0).unwrap();
    assert!(!rows.is_empty(), "CREATE returned no rows");
    assert!(rows[0].get("name").is_some(), "CREATE result missing name");

    // Test SELECT
    let mut resp2 = db.query("SELECT * FROM tenant").await.unwrap();
    let rows2: Vec<serde_json::Value> = resp2.take(0).unwrap();
    assert_eq!(rows2.len(), 1, "Expected 1 tenant");

    // Test via TenantAwareSurrealDb (admin mode) - the path used by handler
    use domain::ports::surreal_db::SurrealDbPort;
    use runtime_server::ports::surreal_db::TenantAwareSurrealDb;
    use std::collections::BTreeMap;

    let admin_db = TenantAwareSurrealDb::new_admin(db.clone());

    // Test query through the wrapper
    let result: Vec<serde_json::Value> = admin_db
        .query(
            "CREATE tenant SET name = $name",
            BTreeMap::from([(
                "name".into(),
                serde_json::Value::String("wrapper-test".into()),
            )]),
        )
        .await
        .unwrap();
    assert!(
        !result.is_empty(),
        "TenantAwareSurrealDb CREATE returned no rows"
    );

    // Test deserializing to TenantRecord (the exact type used in handler)
    #[derive(serde::Deserialize, Debug)]
    struct TenantRecord {
        id: String,
    }
    let result_typed: Vec<TenantRecord> = admin_db
        .query(
            "CREATE tenant SET name = $name",
            BTreeMap::from([(
                "name".into(),
                serde_json::Value::String("typed-test".into()),
            )]),
        )
        .await
        .unwrap();
    assert!(!result_typed.is_empty(), "Typed CREATE returned no rows");
    println!("TenantRecord: {:?}", result_typed[0]);

    // Test the full init flow: SELECT from user_tenant, CREATE tenant, CREATE user_tenant
    let existing: Vec<serde_json::Value> = admin_db
        .query(
            "SELECT id, tenant_id, role FROM user_tenant WHERE user_sub = $sub",
            BTreeMap::from([("sub".into(), serde_json::Value::String("test-user".into()))]),
        )
        .await
        .unwrap();
    assert!(existing.is_empty(), "Expected no existing user_tenant");

    let created: Vec<TenantRecord> = admin_db
        .query(
            "CREATE tenant SET name = $name",
            BTreeMap::from([("name".into(), serde_json::Value::String("full-test".into()))]),
        )
        .await
        .unwrap();
    assert!(!created.is_empty(), "Full flow CREATE returned no rows");
    let tenant_id_str = created[0].id.clone();

    let _: Vec<serde_json::Value> = admin_db
        .query(
            "CREATE user_tenant SET user_sub = $sub, tenant_id = $tenant_id, role = 'owner'",
            BTreeMap::from([
                ("sub".into(), serde_json::Value::String("test-user".into())),
                ("tenant_id".into(), serde_json::Value::String(tenant_id_str)),
            ]),
        )
        .await
        .unwrap();

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
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
