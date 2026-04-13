//! BFF 状态 — 组合所有服务依赖并注入 Axum State。
//!
//! Phase 0: 最小状态 — 仅含配置。Phase 1+: 注入 services/ 实例。

use crate::config::Config;
use storage_turso::EmbeddedTurso;

/// Web BFF 应用状态。
///
/// All fields are cheaply cloneable (Arc-wrapped internally where needed).
#[derive(Clone)]
pub struct BffState {
    pub config: Config,

    /// Embedded libsql for local features (counter, admin, tenant, agent).
    pub embedded_db: Option<EmbeddedTurso>,

    /// Shared HTTP client for external service calls.
    pub http_client: reqwest::Client,
}

impl BffState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Embedded libsql initialization
        let embedded_db = match &config.database_url {
            Some(url) => {
                let db = EmbeddedTurso::new(url).await.ok();
                if let Some(ref db) = db {
                    // Run tenant migrations
                    storage_turso::embedded::run_tenant_migrations(db)
                        .await
                        .ok();
                }
                db
            }
            None => None,
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        Ok(Self {
            config,
            embedded_db,
            http_client,
        })
    }

    /// Create BffState with a pre-initialized EmbeddedTurso instance.
    /// Used for testing with in-memory databases.
    pub async fn new_with_db(db: EmbeddedTurso) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        Self {
            config: Config::default(),
            embedded_db: Some(db),
            http_client,
        }
    }
}
