//! Axum shared state — holds database connections, cache, and HTTP client.
//!
//! Injected via Router::with_state() so all route handlers can access
//! DB, cache, and HTTP client through the State extractor.

use crate::config::{CloudDbProvider, Config, DatabasePathSource};
use crate::error::AppError;
use domain::ports::lib_sql::LibSqlPort;
use moka::future::Cache;
use std::path::PathBuf;
use std::time::Duration;
#[cfg(test)]
use storage_surrealdb::run_tenant_migrations as run_surreal_migrations;
use storage_turso::EmbeddedTurso;
use surrealdb::{Surreal, engine::any::Any};

#[derive(Debug, Clone)]
pub struct StartupDatabaseInfo {
    pub provider: CloudDbProvider,
    pub absolute_path: PathBuf,
    pub source: DatabasePathSource,
}

/// Shared application state for Axum routes.
///
/// All fields are cheaply cloneable (Arc-wrapped internally).
#[derive(Clone)]
pub struct AppState {
    /// SurrealDB connection pool (server-side database).
    pub db: Surreal<Any>,

    /// Turso cloud database connection (legacy field, currently unused).
    pub turso_db: Option<()>,

    /// Which database provider is active.
    pub db_provider: CloudDbProvider,

    /// Moka in-memory cache (replaces Redis per D-10/D-12).
    /// - 10,000 entries max
    /// - 5-minute TTL per entry
    pub cache: Cache<String, String>,

    /// Shared reqwest HTTP client (connection pool, per D-13/D-14).
    /// - 30s timeout
    /// - 10 max idle connections per host
    pub http_client: reqwest::Client,

    /// Application configuration
    pub config: Config,

    /// Embedded libsql for local features (counter, admin).
    /// Initialized in dev mode; None in production if only SurrealDB is used.
    pub embedded_db: Option<EmbeddedTurso>,
}

impl AppState {
    async fn run_embedded_runtime_migrations(embedded_db: &EmbeddedTurso) -> Result<(), AppError> {
        storage_turso::embedded::run_tenant_migrations(embedded_db)
            .await
            .map_err(AppError::Database)?;

        // Defensive migration step: guarantee both tenant tables exist even if
        // adapter-level batch execution semantics vary across engines.
        embedded_db
            .execute(
                "CREATE TABLE IF NOT EXISTS tenant (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
                vec![],
            )
            .await
            .map_err(AppError::Database)?;
        embedded_db
            .execute(
                "CREATE TABLE IF NOT EXISTS user_tenant (id TEXT PRIMARY KEY, user_sub TEXT NOT NULL UNIQUE, tenant_id TEXT NOT NULL REFERENCES tenant(id), role TEXT NOT NULL DEFAULT 'member', joined_at TEXT NOT NULL DEFAULT (datetime('now')))",
                vec![],
            )
            .await
            .map_err(AppError::Database)?;
        embedded_db
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_user_tenant_tenant_id ON user_tenant(tenant_id)",
                vec![],
            )
            .await
            .map_err(AppError::Database)?;

        embedded_db
            .execute(counter_service::application::COUNTER_MIGRATION, vec![])
            .await
            .map_err(AppError::Database)?;

        for migration in agent_service::application::migrations::AGENT_MIGRATIONS {
            embedded_db
                .execute(migration, vec![])
                .await
                .map_err(AppError::Database)?;
        }

        // Clone embedded_db to get an owned handle for the settings repository
        let settings_db = embedded_db.clone();
        let settings_repo = settings_service::infrastructure::LibSqlSettingsRepository::new(settings_db);
        settings_repo.migrate().await.map_err(|e| AppError::Database(e.into()))?;

        Ok(())
    }

    /// Initialize AppState with in-memory SurrealDB for development.
    ///
    /// Production: use `rocksdb://path` or remote SurrealDB endpoint.
    #[cfg(test)]
    pub async fn new_dev() -> Result<Self, AppError> {
        let config = Config::default();

        // SurrealDB — in-memory for dev (per D-04)
        let db = Surreal::<Any>::init();
        db.connect("memory").await?;
        db.use_ns("app").use_db("main").await?;

        // Run tenant schema migrations (tenant + user_tenant tables)
        run_surreal_migrations(&db)
            .await
            .map_err(AppError::Database)?;

        // Embedded libsql for local features (counter, admin)
        let embedded_db = EmbeddedTurso::new(":memory:")
            .await
            .map_err(AppError::Database)?;
        Self::run_embedded_runtime_migrations(&embedded_db).await?;

        // Moka cache — 10k entries, 5min TTL (per D-10/D-11)
        let cache: Cache<String, String> = Cache::builder()
            .max_capacity(config.cache.max_capacity)
            .time_to_live(Duration::from_secs(config.cache.ttl_secs))
            .build();

        // reqwest client — 30s timeout, 10 idle per host (per D-13/D-14)
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.server.request_timeout_secs))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| AppError::Config(e.to_string()))?;

        Ok(Self {
            db,
            turso_db: None,
            db_provider: CloudDbProvider::SurrealDB,
            cache,
            http_client,
            config,
            embedded_db: Some(embedded_db),
        })
    }

    /// Resolve startup database metadata for logging and initialization.
    pub fn startup_database_info(config: &Config) -> Result<StartupDatabaseInfo, AppError> {
        let resolved = config
            .resolved_db_path()
            .map_err(|e| AppError::Config(format!("failed to resolve database path: {e}")))?;

        Ok(StartupDatabaseInfo {
            provider: config.database.provider.clone(),
            absolute_path: resolved.absolute_path,
            source: resolved.source,
        })
    }

    /// Initialize AppState with configuration (production-ready)
    pub async fn new_with_config(config: Config) -> Result<Self, AppError> {
        if config.database.provider != CloudDbProvider::Turso {
            return Err(AppError::Config(
                "runtime_server startup only supports turso provider".to_string(),
            ));
        }

        let db_info = Self::startup_database_info(&config)?;

        if let Some(parent) = db_info.absolute_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                AppError::Config(format!("failed to create database directory: {e}"))
            })?;
        }

        let db_path = db_info.absolute_path.to_string_lossy().to_string();
        let embedded_db = EmbeddedTurso::new(&db_path)
            .await
            .map_err(AppError::Database)?;

        Self::run_embedded_runtime_migrations(&embedded_db).await?;

        // Keep Surreal handle allocated for compatibility with existing AppState shape.
        let db = Surreal::<Any>::init();

        // Moka cache
        let cache: Cache<String, String> = Cache::builder()
            .max_capacity(config.cache.max_capacity)
            .time_to_live(Duration::from_secs(config.cache.ttl_secs))
            .build();

        // reqwest client
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.server.request_timeout_secs))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| AppError::Config(e.to_string()))?;

        Ok(Self {
            db,
            turso_db: None,
            db_provider: config.database.provider.clone(),
            cache,
            http_client,
            config,
            embedded_db: Some(embedded_db),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_with_config_creates_parent_directory_and_embedded_db() {
        let temp_root =
            std::env::temp_dir().join(format!("runtime_server_state_{}", uuid::Uuid::new_v4()));
        let db_path = temp_root.join("nested/runtime_server.db");

        let mut config = Config::default();
        config.database.provider = CloudDbProvider::Turso;
        config.database.url = db_path.to_string_lossy().to_string();

        let state = AppState::new_with_config(config)
            .await
            .expect("state should initialize");

        assert!(db_path.parent().unwrap().exists());
        assert!(state.embedded_db.is_some());

        let _ = std::fs::remove_dir_all(temp_root);
    }

    #[tokio::test]
    async fn new_with_config_fails_fast_for_memory_database_url() {
        let mut config = Config::default();
        config.database.provider = CloudDbProvider::Turso;
        config.database.url = ":memory:".to_string();

        let err = match AppState::new_with_config(config).await {
            Ok(_) => panic!("memory path must be rejected"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("memory"));
    }
}
