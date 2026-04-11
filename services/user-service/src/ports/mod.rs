//! Port interfaces — external dependency abstractions.

use async_trait::async_trait;
use crate::domain::UserTenantBinding;
use crate::domain::error::UserError;

/// Repository port for user data access.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their OAuth subject identifier.
    async fn find_by_sub(&self, user_sub: &str) -> Result<Option<crate::domain::User>, UserError>;

    /// Create a new user record.
    async fn create_user(&self, user: &crate::domain::User) -> Result<(), UserError>;

    /// Update user's last login timestamp.
    async fn update_last_login(&self, user_sub: &str) -> Result<(), UserError>;
}

/// Repository port for tenant data access.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Create a new tenant.
    async fn create_tenant(&self, name: &str) -> Result<String, UserError>;

    /// Find tenant by ID.
    async fn find_by_id(&self, tenant_id: &str) -> Result<Option<crate::domain::Tenant>, UserError>;
}

/// Repository port for user-tenant bindings.
#[async_trait]
pub trait UserTenantRepository: Send + Sync {
    /// Find user's tenant binding.
    async fn find_user_tenant(&self, user_sub: &str) -> Result<Option<UserTenantBinding>, UserError>;

    /// Create a user-tenant binding.
    async fn create_binding(
        &self,
        user_sub: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<UserTenantBinding, UserError>;
}
