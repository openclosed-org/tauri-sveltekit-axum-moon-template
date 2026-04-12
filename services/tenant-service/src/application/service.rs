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

/// Trait for tenant service operations — the application-layer contract.
#[async_trait]
pub trait TenantServiceTrait: Send + Sync {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError>;
    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError>;
    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError>;
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
    async fn create_tenant(
        &self,
        input: CreateTenantInput,
    ) -> Result<Tenant, TenantServiceError> {
        if input.id.is_empty() {
            return Err(TenantServiceError::InvalidInput("tenant id cannot be empty".into()));
        }
        if input.name.is_empty() {
            return Err(TenantServiceError::InvalidInput("tenant name cannot be empty".into()));
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
}
