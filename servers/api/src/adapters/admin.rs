//! Admin service adapters — implement admin ports using concrete services.
//!
//! These adapters live in the server composition layer (not in services).
//! They bridge admin-service ports to actual tenant-service and counter-service implementations.

use async_trait::async_trait;
use kernel::TenantId;
use tenant_service::application::TenantServiceTrait;
use feature_counter::CounterService;

use admin_service::ports::{CounterRepository, TenantRepository, TenantSummary};

/// Tenant repository adapter that wraps tenant-service.
///
/// This adapter implements the admin port's TenantRepository trait by
/// delegating to the actual tenant-service implementation.
pub struct TenantServiceAdapter {
    tenant_service: tenant_service::application::TenantService<
        tenant_service::infrastructure::LibSqlTenantRepository<storage_turso::EmbeddedTurso>,
    >,
}

impl TenantServiceAdapter {
    pub fn new(
        tenant_service: tenant_service::application::TenantService<
            tenant_service::infrastructure::LibSqlTenantRepository<storage_turso::EmbeddedTurso>,
        >,
    ) -> Self {
        Self { tenant_service }
    }
}

#[async_trait]
impl TenantRepository for TenantServiceAdapter {
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let tenants = self
            .tenant_service
            .list_tenants()
            .await
            .map_err(|e: tenant_service::application::TenantServiceError| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
            })?;

        Ok(tenants
            .into_iter()
            .map(|t| TenantSummary {
                id: t.id,
                name: t.name,
                created_at: t.created_at,
            })
            .collect())
    }
}

/// Counter repository adapter that wraps counter-service.
///
/// This adapter implements the admin port's CounterRepository trait by
/// delegating to the actual counter-service implementation.
pub struct CounterServiceAdapter {
    counter_service:
        counter_service::application::RepositoryBackedCounterService<
            counter_service::infrastructure::LibSqlCounterRepository<storage_turso::EmbeddedTurso>,
        >,
}

impl CounterServiceAdapter {
    pub fn new(
        counter_service: counter_service::application::RepositoryBackedCounterService<
            counter_service::infrastructure::LibSqlCounterRepository<storage_turso::EmbeddedTurso>,
        >,
    ) -> Self {
        Self { counter_service }
    }
}

#[async_trait]
impl CounterRepository for CounterServiceAdapter {
    async fn get_value(&self, _tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        // For now, use the global counter
        // TODO: Implement tenant-specific counter logic
        self.counter_service
            .get_value()
            .await
            .map_err(|e: feature_counter::CounterError| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        self.counter_service
            .get_value()
            .await
            .map_err(|e: feature_counter::CounterError| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>
            })
    }
}
