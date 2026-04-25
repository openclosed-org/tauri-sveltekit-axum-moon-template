//! Tenant service — orchestrates tenant lifecycle use cases via repository ports.

use async_trait::async_trait;

use crate::domain::{CreateTenantInput, Tenant};
use crate::ports::{RepositoryError, TenantRepository};

/// Application-level error type.
#[derive(Debug, thiserror::Error)]
pub enum TenantServiceError {
    #[error("Database error: {0}")]
    Database(#[from] RepositoryError),
    #[error("Tenant not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result of initializing a tenant for a user.
#[derive(Debug, Clone)]
pub struct InitTenantResult {
    pub tenant_id: String,
    pub role: String,
    pub created: bool,
}

/// Trait for tenant service operations — the application-layer contract.
#[async_trait]
pub trait TenantServiceTrait: Send + Sync {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError>;
    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError>;
    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError>;

    /// Initialize a tenant for a user. Creates tenant + user_tenant binding if not exists.
    /// Returns existing or newly created tenant_id.
    async fn init_tenant_for_user(
        &self,
        user_sub: &str,
        tenant_name: &str,
    ) -> Result<InitTenantResult, TenantServiceError>;
}

/// Concrete service implementation backed by any TenantRepository.
pub struct TenantService<R: TenantRepository> {
    repo: R,
}

impl<R: TenantRepository> TenantService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: TenantRepository> TenantServiceTrait for TenantService<R> {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError> {
        if input.id.is_empty() {
            return Err(TenantServiceError::InvalidInput(
                "tenant id cannot be empty".into(),
            ));
        }
        if input.name.is_empty() {
            return Err(TenantServiceError::InvalidInput(
                "tenant name cannot be empty".into(),
            ));
        }

        self.repo
            .create_tenant(input)
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError> {
        self.repo
            .get_tenant(id)
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError> {
        self.repo
            .list_tenants()
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError> {
        self.repo
            .delete_tenant(id)
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn init_tenant_for_user(
        &self,
        user_sub: &str,
        tenant_name: &str,
    ) -> Result<InitTenantResult, TenantServiceError> {
        // 1. Check if user already has a tenant binding
        if let Some(binding) = self
            .repo
            .find_user_tenant(user_sub)
            .await
            .map_err(TenantServiceError::Database)?
        {
            return Ok(InitTenantResult {
                tenant_id: binding.tenant_id,
                role: binding.role,
                created: false,
            });
        }

        // 2. Create tenant with generated UUID
        let tenant_id = uuid::Uuid::new_v4().to_string();
        let input = CreateTenantInput {
            id: tenant_id,
            name: tenant_name.to_string(),
        };
        let tenant = self.create_tenant(input).await?;

        // 3. Create user_tenant binding (owner role)
        self.repo
            .create_user_tenant_binding(user_sub, &tenant.id, "owner")
            .await
            .map_err(TenantServiceError::Database)?;

        Ok(InitTenantResult {
            tenant_id: tenant.id,
            role: "owner".to_string(),
            created: true,
        })
    }
}
