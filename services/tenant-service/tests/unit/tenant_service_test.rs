//! Unit tests for TenantService using mock repository.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use tenant_service::application::{TenantService, TenantServiceTrait};
use tenant_service::domain::{CreateTenantInput, Tenant};
use tenant_service::ports::{RepositoryError, TenantRepository};

/// In-memory mock repository for testing.
struct MockTenantRepository {
    tenants: Arc<Mutex<Vec<Tenant>>>,
}

impl MockTenantRepository {
    fn new() -> Self {
        Self {
            tenants: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[async_trait]
impl TenantRepository for MockTenantRepository {
    async fn create_tenant(
        &self,
        input: CreateTenantInput,
    ) -> Result<Tenant, RepositoryError> {
        let tenant = Tenant {
            id: input.id.clone(),
            name: input.name.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.tenants.lock().await.push(tenant.clone());
        Ok(tenant)
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, RepositoryError> {
        let tenants = self.tenants.lock().await;
        Ok(tenants.iter().find(|t| t.id == id).cloned())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, RepositoryError> {
        let tenants = self.tenants.lock().await;
        Ok(tenants.clone())
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), RepositoryError> {
        let mut tenants = self.tenants.lock().await;
        tenants.retain(|t| t.id != id);
        Ok(())
    }
}

#[tokio::test]
async fn test_create_and_get_tenant() {
    let repo = MockTenantRepository::new();
    let service = TenantService::new(repo);

    let input = CreateTenantInput {
        id: "tenant-1".into(),
        name: "Test Tenant".into(),
    };

    let created = service.create_tenant(input.clone()).await.unwrap();
    assert_eq!(created.id, "tenant-1");
    assert_eq!(created.name, "Test Tenant");

    let found = service.get_tenant("tenant-1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Tenant");
}

#[tokio::test]
async fn test_list_tenants() {
    let repo = MockTenantRepository::new();
    let service = TenantService::new(repo);

    service
        .create_tenant(CreateTenantInput {
            id: "t1".into(),
            name: "Tenant 1".into(),
        })
        .await
        .unwrap();

    service
        .create_tenant(CreateTenantInput {
            id: "t2".into(),
            name: "Tenant 2".into(),
        })
        .await
        .unwrap();

    let tenants = service.list_tenants().await.unwrap();
    assert_eq!(tenants.len(), 2);
}

#[tokio::test]
async fn test_delete_tenant() {
    let repo = MockTenantRepository::new();
    let service = TenantService::new(repo);

    service
        .create_tenant(CreateTenantInput {
            id: "t1".into(),
            name: "To Delete".into(),
        })
        .await
        .unwrap();

    service.delete_tenant("t1").await.unwrap();

    let found = service.get_tenant("t1").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_tenant() {
    let repo = MockTenantRepository::new();
    let service = TenantService::new(repo);

    let found = service.get_tenant("nonexistent").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_reject_empty_tenant_id() {
    let repo = MockTenantRepository::new();
    let service = TenantService::new(repo);

    let result = service
        .create_tenant(CreateTenantInput {
            id: "".into(),
            name: "Test".into(),
        })
        .await;

    assert!(result.is_err());
}
