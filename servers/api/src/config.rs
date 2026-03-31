//! Application configuration using figment.
//!
//! 2026 Rust Best Practices:
//! - figment for hierarchical configuration (TOML, ENV, JSON)
//! - Type-safe configuration with serde
//! - Profile-based configuration (dev, prod)

use figment::{
    Figment, Profile,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum CloudDbProvider {
    #[serde(rename = "surrealdb")]
    SurrealDB,
    #[serde(rename = "turso")]
    Turso,
}

impl Default for CloudDbProvider {
    fn default() -> Self {
        Self::SurrealDB
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub provider: CloudDbProvider,
    pub url: String,
    #[serde(default)]
    pub ns: String,
    #[serde(default)]
    pub db: String,
    #[serde(default)]
    pub auth_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub max_capacity: u64,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3001,
                request_timeout_secs: 30,
            },
            database: DatabaseConfig {
                provider: CloudDbProvider::SurrealDB,
                url: "memory".to_string(),
                ns: "app".to_string(),
                db: "main".to_string(),
                auth_token: String::new(),
            },
            cache: CacheConfig {
                max_capacity: 10_000,
                ttl_secs: 300,
            },
            auth: AuthConfig {
                jwt_secret: "dev-secret-change-in-production".to_string(),
                jwt_expiration_secs: 86400,
            },
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config.toml").nested())
            .merge(Env::prefixed("APP_").global())
            .extract()
    }

    pub fn from_profile(profile: impl Into<Profile>) -> Result<Self, figment::Error> {
        let profile = profile.into();
        let profile_str = profile.as_str();
        Figment::new()
            .merge(Toml::file("config.toml").profile(profile_str))
            .merge(Env::prefixed("APP_").profile(profile_str).global())
            .extract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3001);
        assert_eq!(config.cache.ttl_secs, 300);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("server"));
    }
}
