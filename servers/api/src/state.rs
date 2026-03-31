//! Axum shared state — holds database connections, cache, and HTTP client.
//!
//! Injected via Router::with_state() so all route handlers can access
//! DB, cache, and HTTP client through the State extractor.

use crate::config::{CloudDbProvider, Config};
use crate::error::AppError;
use crate::ports::surreal_db::run_tenant_migrations as run_surreal_migrations;
use crate::ports::turso_db::TursoDb;
use crate::ports::turso_db::run_tenant_migrations as run_turso_migrations;
use moka::future::Cache;
use std::time::Duration;
use surrealdb::{Surreal, engine::any::Any};

/// Shared application state for Axum routes.
///
/// All fields are cheaply cloneable (Arc-wrapped internally).
#[derive(Clone)]
pub struct AppState {
    /// SurrealDB connection pool (server-side database).
    pub db: Surreal<Any>,

    /// Turso cloud database connection (alternative to SurrealDB).
    pub turso_db: Option<TursoDb>,

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
}

impl AppState {
    /// Initialize AppState with in-memory SurrealDB for development.
    ///
    /// Production: use `rocksdb://path` or remote SurrealDB endpoint.
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
        })
    }

    /// Initialize AppState with configuration (production-ready)
    pub async fn new_with_config(config: Config) -> Result<Self, AppError> {
        let turso_db = match &config.database.provider {
            CloudDbProvider::Turso => {
                let turso = TursoDb::new(&config.database.url, &config.database.auth_token)
                    .await
                    .map_err(|e| AppError::Database(e.into()))?;

                run_turso_migrations(&turso)
                    .await
                    .map_err(|e| AppError::Database(e.into()))?;

                Some(turso)
            }
            CloudDbProvider::SurrealDB => None,
        };

        // SurrealDB connection (always initialized for compatibility)
        let db = Surreal::<Any>::init();
        db.connect(&config.database.url)
            .await
            .map_err(|e| AppError::Database(e.into()))?;
        db.use_ns(&config.database.ns)
            .use_db(&config.database.db)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

        // Run tenant schema migrations for SurrealDB
        if config.database.provider == CloudDbProvider::SurrealDB {
            run_surreal_migrations(&db)
                .await
                .map_err(AppError::Database)?;
        }

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
            turso_db,
            db_provider: config.database.provider.clone(),
            cache,
            http_client,
            config,
        })
    }
}
