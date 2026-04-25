//! Bootstrap and composition root for the Web BFF.

use crate::config::Config;
use crate::state::{BffCompositionRoot, BffState, DatabaseBackend};
use authz::{AuthzPort, MockAuthzAdapter, OpenFgaAdapter, OpenFgaConfig};
use counter_service::{
    application::RepositoryBackedCounterService, infrastructure::LibSqlCounterRepository,
};
use data::ports::lib_sql::LibSqlPort;
use moka::future::Cache;
use std::sync::Arc;
use storage_turso::{EmbeddedTurso, TursoBackend, TursoCloud};
use tenant_service::{
    application::TenantService,
    infrastructure::libsql_adapter::LibSqlTenantRepository as TenantServiceRepository,
};
use user_service::infrastructure::{
    LibSqlTenantRepository as UserTenantInfoRepository, LibSqlUserRepository,
    LibSqlUserTenantRepository,
};

pub async fn bootstrap_bff_state(config: Config) -> anyhow::Result<BffState> {
    let db = initialize_database(&config).await?;
    let authz = build_authz_adapter(&config)?;
    let composition = build_composition_root(db.clone());

    Ok(BffState {
        config,
        db,
        composition,
        counter_cache: build_counter_cache(),
        http_client: build_http_client(),
        authz,
    })
}

pub async fn bootstrap_test_state(db: EmbeddedTurso) -> anyhow::Result<BffState> {
    run_user_migrations(&db).await?;
    let db = Some(DatabaseBackend::Embedded(db));

    Ok(BffState {
        config: Config::default(),
        db: db.clone(),
        composition: build_composition_root(db),
        counter_cache: build_counter_cache(),
        http_client: build_http_client(),
        authz: Arc::new(MockAuthzAdapter::new()),
    })
}

fn build_counter_cache() -> Cache<String, i64> {
    Cache::builder()
        .max_capacity(10_000)
        .time_to_live(std::time::Duration::from_secs(300))
        .build()
}

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap_or_default()
}

fn build_composition_root(db: Option<DatabaseBackend>) -> Option<BffCompositionRoot> {
    let backend = match db {
        Some(DatabaseBackend::Embedded(db)) => TursoBackend::Embedded(db),
        Some(DatabaseBackend::Remote(db)) => TursoBackend::Remote(db),
        None => return None,
    };

    Some(BffCompositionRoot {
        counter_service: Arc::new(RepositoryBackedCounterService::new(
            LibSqlCounterRepository::new(backend.clone()),
        )),
        tenant_service: Arc::new(TenantService::new(TenantServiceRepository::new(
            backend.clone(),
        ))),
        user_profile_repository: Arc::new(LibSqlUserRepository::new(backend.clone())),
        user_tenant_repository: Arc::new(LibSqlUserTenantRepository::new(backend.clone())),
        user_tenant_info_repository: Arc::new(UserTenantInfoRepository::new(backend)),
    })
}

async fn initialize_database(config: &Config) -> anyhow::Result<Option<DatabaseBackend>> {
    if let (Some(url), Some(token)) = (&config.turso_url, &config.turso_auth_token) {
        let db = TursoCloud::new(url, token)
            .await
            .map_err(anyhow::Error::msg)?;
        storage_turso::remote::run_tenant_migrations(&db)
            .await
            .map_err(anyhow::Error::msg)?;
        run_user_migrations(&db).await?;
        LibSqlCounterRepository::new(db.clone())
            .migrate()
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(Some(DatabaseBackend::Remote(db)))
    } else if let Some(url) = &config.database_url {
        let db = EmbeddedTurso::new(url).await.map_err(anyhow::Error::msg)?;
        storage_turso::embedded::run_tenant_migrations(&db)
            .await
            .map_err(anyhow::Error::msg)?;
        run_user_migrations(&db).await?;
        LibSqlCounterRepository::new(db.clone())
            .migrate()
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(Some(DatabaseBackend::Embedded(db)))
    } else {
        Ok(None)
    }
}

async fn run_user_migrations<P: LibSqlPort>(db: &P) -> anyhow::Result<()> {
    db.execute_batch(include_str!(
        "../../../../services/user-service/migrations/001_create_user_tables.sql"
    ))
    .await
    .map_err(anyhow::Error::msg)
}

fn build_authz_adapter(config: &Config) -> anyhow::Result<Arc<dyn AuthzPort>> {
    if config.openfga_endpoint.trim().is_empty() {
        return Ok(Arc::new(MockAuthzAdapter::new()));
    }

    let adapter = OpenFgaAdapter::new(OpenFgaConfig {
        endpoint: config.openfga_endpoint.clone(),
        store_id: config.openfga_store_id.clone(),
        authorization_model_id: (!config.openfga_authorization_model_id.trim().is_empty())
            .then(|| config.openfga_authorization_model_id.clone()),
    })
    .map_err(anyhow::Error::msg)?;

    Ok(Arc::new(adapter))
}
