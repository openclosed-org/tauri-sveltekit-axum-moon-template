//! Admin BFF application state — composition root for admin services.

use domain::ports::lib_sql::LibSqlPort;
use reqwest::Client;
use std::sync::Arc;

use storage_turso::EmbeddedTurso;

use crate::config::Config;

/// Admin BFF application state
#[derive(Clone)]
pub struct AdminBffState {
    pub config: Arc<Config>,
    pub http_client: Client,
    pub embedded_db: Option<EmbeddedTurso>,
}

impl AdminBffState {
    pub async fn new_with_config(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Embedded libsql initialization
        let embedded_db = match &config.database_url {
            Some(url) => {
                let db = EmbeddedTurso::new(url).await.ok();
                if let Some(ref db) = db {
                    // Run migrations for tenant and counter tables
                    storage_turso::embedded::run_tenant_migrations(db).await.ok();
                    // Run counter migration
                    if let Err(e) = db.execute(
                        counter_service::application::COUNTER_MIGRATION,
                        vec![],
                    ).await {
                        tracing::warn!("Failed to run counter migration: {}", e);
                    }
                }
                db
            }
            None => None,
        };

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self {
            config: Arc::new(config.clone()),
            http_client,
            embedded_db,
        })
    }
}
