//! BFF 配置 — 环境变量 + figment 加载。

use serde::{Deserialize, Serialize};

/// Web BFF 应用配置。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub jwt_secret: String,
    /// Zitadel issuer URL (e.g., "https://zitadel.example.com").
    /// When set, the middleware validates JWTs against Zitadel's OIDC discovery.
    /// Dev fallback: empty string → uses `jwt_secret` for HS256 validation.
    pub zitadel_issuer: String,
    /// Expected audience in JWT `aud` claim.
    /// Dev fallback: empty string → audience check skipped.
    pub zitadel_audience: String,
    /// OpenFGA API endpoint (e.g., "http://localhost:8081").
    /// When set, the BFF uses the real OpenFGA adapter for authorization.
    /// Dev fallback: empty string → uses MockAuthzAdapter (allow-all).
    pub openfga_endpoint: String,
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
            jwt_secret: "dev-secret-change-in-production".to_string(),
            zitadel_issuer: String::new(),
            zitadel_audience: String::new(),
            openfga_endpoint: String::new(),
            database_url: None,
            turso_url: None,
            turso_auth_token: None,
        }
    }
}
