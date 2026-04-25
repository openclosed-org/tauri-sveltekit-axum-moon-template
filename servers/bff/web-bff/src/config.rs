//! BFF 配置 — 环境变量 + figment 加载。

use serde::{Deserialize, Serialize};

/// Web BFF 应用配置。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    /// Authentication mode for protected API routes.
    /// - `jwt` (default): require `Authorization: Bearer <token>`.
    /// - `dev_headers`: allow local identity injection via `x-dev-user-sub` and optional
    ///   `x-dev-tenant-id` / `x-dev-user-roles` before falling back to Bearer auth.
    pub auth_mode: String,
    pub jwt_secret: String,
    /// Zitadel issuer URL (e.g., "https://zitadel.example.com").
    /// When set, the middleware validates JWTs against Zitadel's OIDC discovery.
    /// Dev fallback: empty string → uses `jwt_secret` for HS256 validation.
    pub zitadel_issuer: String,
    /// Expected audience in JWT `aud` claim.
    /// Dev fallback: empty string → audience check skipped.
    pub zitadel_audience: String,
    /// Optional Zitadel introspection client id.
    /// When set together with `zitadel_introspection_client_secret`, the BFF
    /// validates access tokens through Zitadel introspection instead of local JWKS verification.
    pub zitadel_introspection_client_id: String,
    /// Optional Zitadel introspection client secret.
    pub zitadel_introspection_client_secret: String,
    /// OpenFGA API endpoint (e.g., "http://localhost:8081").
    /// When set, the BFF uses the real OpenFGA adapter for authorization.
    /// Dev fallback: empty string → uses MockAuthzAdapter (allow-all).
    pub openfga_endpoint: String,
    /// OpenFGA store id used by the real adapter.
    pub openfga_store_id: String,
    /// Optional OpenFGA authorization model id.
    pub openfga_authorization_model_id: String,
    /// Embedded Turso database URL (e.g., "file:path.db" or "memory").
    /// Used when turso_url is not set.
    pub database_url: Option<String>,
    /// Remote Turso database URL (e.g., "libsql://your-db.turso.io").
    /// When set, the BFF connects to Turso cloud instead of embedded mode.
    pub turso_url: Option<String>,
    /// Turso authentication token for remote connections.
    pub turso_auth_token: Option<String>,
}

impl Config {
    /// 从环境变量加载配置（APP_ 前缀）。
    pub fn from_env() -> anyhow::Result<Self> {
        platform::load_config(Self::default(), "APP_", Some("APP_CONFIG_FILE")).map_err(Into::into)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 3010,
            cors_allowed_origins: vec![],
            auth_mode: "jwt".to_string(),
            jwt_secret: "dev-secret-change-in-production".to_string(),
            zitadel_issuer: String::new(),
            zitadel_audience: String::new(),
            zitadel_introspection_client_id: String::new(),
            zitadel_introspection_client_secret: String::new(),
            openfga_endpoint: String::new(),
            openfga_store_id: String::new(),
            openfga_authorization_model_id: String::new(),
            database_url: None,
            turso_url: None,
            turso_auth_token: None,
        }
    }
}

impl Config {
    pub fn allows_dev_headers(&self) -> bool {
        self.auth_mode.eq_ignore_ascii_case("dev_headers")
    }
}
