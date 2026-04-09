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
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

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
    /// CORS allowed origins. Empty = permissive (dev mode). Set to enforce whitelist (prod).
    /// Read from `APP_SERVER__CORS_ALLOWED_ORIGINS` env var (comma-separated).
    #[serde(default, deserialize_with = "deserialize_comma_separated_vec")]
    pub cors_allowed_origins: Vec<String>,
}

/// Deserialize a comma-separated string into a `Vec<String>`.
///
/// Supports both TOML array (`["a","b"]`) and env var comma-separated (`a,b`).
/// When the input is already a sequence (TOML array), falls back to default Vec deserialization.
fn deserialize_comma_separated_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CommaSeparatedVec;

    impl<'de> serde::de::Visitor<'de> for CommaSeparatedVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a comma-separated string or a sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
        where
            E: serde::de::Error,
        {
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<String>, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                vec.push(item);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(CommaSeparatedVec)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub enum CloudDbProvider {
    #[serde(rename = "surrealdb")]
    SurrealDB,
    #[serde(rename = "turso")]
    #[default]
    Turso,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabasePathSource {
    Default,
    Env,
}

#[derive(Debug, Clone)]
pub struct ResolvedDatabasePath {
    pub absolute_path: PathBuf,
    pub source: DatabasePathSource,
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
                cors_allowed_origins: Vec::new(),
            },
            database: DatabaseConfig {
                provider: CloudDbProvider::Turso,
                url: "servers/api/.data/runtime_server.db".to_string(),
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

    pub fn resolved_db_path(&self) -> Result<ResolvedDatabasePath, std::io::Error> {
        let source = if std::env::var_os("APP_DATABASE__URL").is_some() {
            DatabasePathSource::Env
        } else {
            DatabasePathSource::Default
        };

        if is_memory_database_url(&self.database.url) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "database url '{}' is not allowed: memory databases are forbidden",
                    self.database.url
                ),
            ));
        }

        let input_path = Path::new(&self.database.url);
        let absolute_path = if input_path.exists() {
            input_path.canonicalize()?
        } else if input_path.is_absolute() {
            input_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(input_path)
        };

        Ok(ResolvedDatabasePath {
            absolute_path,
            source,
        })
    }
}

fn is_memory_database_url(url: &str) -> bool {
    let normalized = url.trim().to_lowercase();
    normalized == "memory" || normalized == ":memory:" || normalized.contains(":memory:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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

    #[test]
    fn default_database_uses_turso_file_path() {
        let config = Config::default();

        assert_eq!(config.database.provider, CloudDbProvider::Turso);
        assert!(!config.database.url.contains("memory"));
        assert!(!config.database.url.contains(":memory:"));
        assert_eq!(
            PathBuf::from(&config.database.url),
            PathBuf::from("servers/api/.data/runtime_server.db")
        );
    }

    #[test]
    fn resolved_db_path_reports_default_source() {
        let _guard = env_lock().lock().unwrap();
        let original = std::env::var("APP_DATABASE__URL").ok();
        unsafe {
            std::env::remove_var("APP_DATABASE__URL");
        }

        let config = Config::default();
        let resolved = config.resolved_db_path().unwrap();

        assert_eq!(resolved.source, DatabasePathSource::Default);
        assert!(resolved.absolute_path.is_absolute());
        assert!(
            resolved
                .absolute_path
                .ends_with("servers/api/.data/runtime_server.db")
        );

        if let Some(value) = original {
            unsafe {
                std::env::set_var("APP_DATABASE__URL", value);
            }
        }
    }

    #[test]
    fn resolved_db_path_prefers_env_and_reports_env_source() {
        let _guard = env_lock().lock().unwrap();
        let original = std::env::var("APP_DATABASE__URL").ok();
        let env_path = "servers/api/.data/from-env.db";
        unsafe {
            std::env::set_var("APP_DATABASE__URL", env_path);
        }

        let mut config = Config::default();
        config.database.url = env_path.to_string();
        let resolved = config.resolved_db_path().unwrap();

        assert_eq!(config.database.url, env_path);
        assert_eq!(resolved.source, DatabasePathSource::Env);
        assert!(
            resolved
                .absolute_path
                .ends_with("servers/api/.data/from-env.db")
        );

        if let Some(value) = original {
            unsafe {
                std::env::set_var("APP_DATABASE__URL", value);
            }
        } else {
            unsafe {
                std::env::remove_var("APP_DATABASE__URL");
            }
        }
    }

    #[test]
    fn resolved_db_path_rejects_memory_urls() {
        let mut config = Config::default();
        config.database.url = ":memory:".to_string();

        let err = config.resolved_db_path().unwrap_err().to_string();
        assert!(err.contains("memory"));
    }
}
