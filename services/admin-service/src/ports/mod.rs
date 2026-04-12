//! Admin service ports — abstract interfaces for external dependencies.
//!
//! These traits define the capabilities admin-service needs from external systems
//! without coupling to specific implementations.
//!
//! ## Design
//! - `TenantRepository`: Access to tenant data
//! - `CounterRepository`: Access to counter data
//!
//! Implementations of these ports live in `infrastructure/` or in server-level
//! adapters that bridge to tenant-service and counter-service.

use async_trait::async_trait;
use kernel::TenantId;

/// Summary information about a tenant.
#[derive(Debug, Clone)]
pub struct TenantSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Port for accessing tenant data.
///
/// This is an abstract interface — concrete implementations (e.g., wrapping
/// tenant-service or direct DB access) are provided in infrastructure adapters.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// List all tenants.
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Port for accessing counter data.
///
/// This is an abstract interface — concrete implementations (e.g., wrapping
/// counter-service or direct DB access) are provided in infrastructure adapters.
#[async_trait]
pub trait CounterRepository: Send + Sync {
    /// Get counter value for a specific tenant.
    async fn get_value(&self, tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;

    /// Get global/default counter value.
    async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
}
