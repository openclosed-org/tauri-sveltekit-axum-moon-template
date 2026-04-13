//! Repository port for tenant data access.

use async_trait::async_trait;

/// Error type for repository operations.
pub type RepositoryError = Box<dyn std::error::Error + Send + Sync>;

/// Represents a user-tenant binding.
#[derive(Debug, Clone)]
pub struct UserTenantBinding {
    pub tenant_id: String,
    pub role: String,
}

/// Abstract repository interface for tenant operations.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Create a new tenant.
    async fn create_tenant(
        &self,
        input: crate::domain::CreateTenantInput,
    ) -> Result<crate::domain::Tenant, RepositoryError>;

    /// Get a tenant by ID.
    async fn get_tenant(&self, id: &str) -> Result<Option<crate::domain::Tenant>, RepositoryError>;

    /// List all tenants.
    async fn list_tenants(&self) -> Result<Vec<crate::domain::Tenant>, RepositoryError>;

    /// Delete a tenant by ID.
    async fn delete_tenant(&self, id: &str) -> Result<(), RepositoryError>;

    /// Check if a user already has a tenant binding.
    async fn find_user_tenant(&self, user_sub: &str) -> Result<Option<UserTenantBinding>, RepositoryError>;

    /// Create a user-tenant binding.
    async fn create_user_tenant_binding(
        &self,
        user_sub: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<(), RepositoryError>;
}
