//! BFF 状态 — 组合所有服务依赖并注入 Axum State。
//!
//! Supports both embedded Turso (local SQLite) and remote Turso cloud.
//! When `turso_url` is configured, connects to Turso cloud.
//! Otherwise falls back to embedded database.

use crate::config::Config;
use authz::MockAuthzAdapter;
use counter_service::infrastructure::LibSqlCounterRepository;
use moka::future::Cache;
use std::sync::Arc;
use storage_turso::EmbeddedTurso;
use storage_turso::TursoCloud;

/// Database backend — either embedded or remote Turso.
#[derive(Clone)]
pub enum DatabaseBackend {
    Embedded(EmbeddedTurso),
    Remote(TursoCloud),
}

/// Web BFF 应用状态。
///
/// All fields are cheaply cloneable (Arc-wrapped internally where needed).
#[derive(Clone)]
pub struct BffState {
    pub config: Config,

    /// Database backend — embedded or remote Turso.
    pub db: Option<DatabaseBackend>,

    /// In-process cache for counter values (tenant_id → value).
    pub counter_cache: Cache<String, i64>,

    /// Shared HTTP client for external service calls.
    pub http_client: reqwest::Client,

    /// Authorization adapter — mock for dev, OpenFGA for prod.
    pub authz: MockAuthzAdapter,
}

impl BffState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Database initialization — prefer remote Turso if configured
        let db = if let (Some(url), Some(token)) = (&config.turso_url, &config.turso_auth_token) {
            let db = TursoCloud::new(url, token)
                .await
                .map_err(anyhow::Error::msg)?;
            storage_turso::remote::run_tenant_migrations(&db)
                .await
                .map_err(anyhow::Error::msg)?;
            LibSqlCounterRepository::new(db.clone())
                .migrate()
                .await
                .map_err(anyhow::Error::msg)?;
            Some(DatabaseBackend::Remote(db))
        } else if let Some(url) = &config.database_url {
            let db = EmbeddedTurso::new(url).await.map_err(anyhow::Error::msg)?;
            storage_turso::embedded::run_tenant_migrations(&db)
                .await
                .map_err(anyhow::Error::msg)?;
            LibSqlCounterRepository::new(db.clone())
                .migrate()
                .await
                .map_err(anyhow::Error::msg)?;
            Some(DatabaseBackend::Embedded(db))
        } else {
            None
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        let counter_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(300))
            .build();

        Ok(Self {
            config,
            db,
            counter_cache,
            http_client,
            authz: MockAuthzAdapter::new(),
        })
    }

    /// Create BffState with a pre-initialized database instance.
    /// Used for testing with in-memory databases.
    pub async fn new_with_db(db: EmbeddedTurso) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        Self {
            config: Config::default(),
            db: Some(DatabaseBackend::Embedded(db)),
            counter_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(std::time::Duration::from_secs(300))
                .build(),
            http_client,
            authz: MockAuthzAdapter::new(),
        }
    }
}
