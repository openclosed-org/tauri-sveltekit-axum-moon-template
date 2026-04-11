//! BFF 配置 — 环境变量 + figment 加载。

use figment::{Figment, providers::Env};
use serde::Deserialize;

/// Web BFF 应用配置。
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub jwt_secret: String,
    /// Embedded Turso database URL (e.g., ":memory:" or "file:path.db").
    pub database_url: Option<String>,
}

impl Config {
    /// 从环境变量加载配置（APP_ 前缀）。
    pub fn from_env() -> anyhow::Result<Self> {
        let config: Config = Figment::new()
            .merge(Env::prefixed("APP_").global())
            .extract()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 3010,
            cors_allowed_origins: vec![],
            jwt_secret: "dev-secret-change-in-production".to_string(),
            database_url: None,
        }
    }
}
