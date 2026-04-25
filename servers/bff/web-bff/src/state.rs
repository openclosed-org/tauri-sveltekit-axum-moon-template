//! BFF 状态 — 组合所有服务依赖并注入 Axum State。
//!
//! Supports both embedded Turso (local SQLite) and remote Turso cloud.
//! When `turso_url` is configured, connects to Turso cloud.
//! Otherwise falls back to embedded database.

use crate::bootstrap::{bootstrap_bff_state, bootstrap_test_state};
use crate::composition::{
    CounterServiceHandle, TenantServiceHandle, UserProfileRepositoryHandle,
    UserTenantInfoRepositoryHandle, UserTenantRepositoryHandle,
};
use crate::config::Config;
use authz::{AuthzPort, AuthzTupleKey};
use moka::future::Cache;
use std::sync::Arc;
use storage_turso::{EmbeddedTurso, TursoCloud};

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

    /// Prebuilt application wiring owned by the Web BFF composition root.
    pub composition: Option<BffCompositionRoot>,

    /// In-process cache for counter values (tenant_id → value).
    pub counter_cache: Cache<String, i64>,

    /// Shared HTTP client for external service calls.
    pub http_client: reqwest::Client,

    /// Authorization adapter — mock for dev, OpenFGA when explicitly configured.
    pub authz: Arc<dyn AuthzPort>,
}

#[derive(Clone)]
pub struct BffCompositionRoot {
    pub counter_service: CounterServiceHandle,
    pub tenant_service: TenantServiceHandle,
    pub user_profile_repository: UserProfileRepositoryHandle,
    pub user_tenant_repository: UserTenantRepositoryHandle,
    pub user_tenant_info_repository: UserTenantInfoRepositoryHandle,
}

impl BffState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        bootstrap_bff_state(config).await
    }

    /// Create BffState with a pre-initialized database instance.
    /// Used for testing with in-memory databases.
    pub async fn new_with_db(db: EmbeddedTurso) -> Self {
        bootstrap_test_state(db)
            .await
            .expect("failed to bootstrap test BFF state")
    }

    pub async fn seed_dev_counter_authz(
        &self,
        user_sub: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<(), authz::AuthzError> {
        let user = format!("user:{user_sub}");
        let tenant_object = format!("tenant:{tenant_id}");
        let counter_object = format!("counter:{tenant_id}");

        for tuple in [
            AuthzTupleKey::new(&user, role, &tenant_object),
            AuthzTupleKey::new(&user, "member", &tenant_object),
            AuthzTupleKey::new(&user, "can_read", &counter_object),
            AuthzTupleKey::new(&user, "can_write", &counter_object),
        ] {
            let exists = self
                .authz
                .check(&tuple.user, &tuple.relation, &tuple.object)
                .await?;
            if !exists {
                self.authz.write_tuple(&tuple).await?;
            }
        }

        Ok(())
    }

    pub fn counter_service(&self) -> Option<CounterServiceHandle> {
        self.composition
            .as_ref()
            .map(|composition| composition.counter_service.clone())
    }

    pub fn user_profile_repository(&self) -> Option<UserProfileRepositoryHandle> {
        self.composition
            .as_ref()
            .map(|composition| composition.user_profile_repository.clone())
    }

    pub fn user_tenant_repository(&self) -> Option<UserTenantRepositoryHandle> {
        self.composition
            .as_ref()
            .map(|composition| composition.user_tenant_repository.clone())
    }

    pub fn user_read_repositories(
        &self,
    ) -> Option<(UserTenantRepositoryHandle, UserTenantInfoRepositoryHandle)> {
        let composition = self.composition.as_ref()?;
        Some((
            composition.user_tenant_repository.clone(),
            composition.user_tenant_info_repository.clone(),
        ))
    }

    pub fn tenant_service(&self) -> Option<TenantServiceHandle> {
        self.composition
            .as_ref()
            .map(|composition| composition.tenant_service.clone())
    }
}
