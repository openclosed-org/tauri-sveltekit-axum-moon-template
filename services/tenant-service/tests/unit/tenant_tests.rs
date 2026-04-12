//! Tenant service unit tests.

#[cfg(test)]
mod tenant_tests {
    use crate::application::{TenantService, TenantServiceTrait};
    use crate::domain::{CreateTenantInput, Tenant};
    use crate::application::TenantServiceError;
    use crate::ports::{TenantRepository, RepositoryError};

    /// Mock tenant repository for testing.
    struct MockTenantRepository {
        tenants: std::sync::Mutex<Vec<Tenant>>,
    }

    impl MockTenantRepository {
        fn new() -> Self {
            Self {
                tenants: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
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
            let mut tenants = self.tenants.lock().unwrap();
            tenants.push(tenant.clone());
            Ok(tenant)
        }

        async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, RepositoryError> {
            let tenants = self.tenants.lock().unwrap();
            Ok(tenants.iter().find(|t| t.id == id).cloned())
        }

        async fn list_tenants(&self) -> Result<Vec<Tenant>, RepositoryError> {
            let tenants = self.tenants.lock().unwrap();
            Ok(tenants.clone())
        }

        async fn delete_tenant(&self, id: &str) -> Result<(), RepositoryError> {
            let mut tenants = self.tenants.lock().unwrap();
            tenants.retain(|t| t.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_tenant_success() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo);

        let input = CreateTenantInput {
            id: "tenant-123".to_string(),
            name: "Test Tenant".to_string(),
        };

        let result = service.create_tenant(input).await.unwrap();
        assert_eq!(result.id, "tenant-123");
        assert_eq!(result.name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_create_tenant_empty_id() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo);

        let input = CreateTenantInput {
            id: "".to_string(),
            name: "Test Tenant".to_string(),
        };

        let result = service.create_tenant(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TenantServiceError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_create_tenant_empty_name() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo);

        let input = CreateTenantInput {
            id: "tenant-456".to_string(),
            name: "".to_string(),
        };

        let result = service.create_tenant(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TenantServiceError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_get_tenant_existing() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo.clone());

        // Create a tenant first
        let input = CreateTenantInput {
            id: "tenant-789".to_string(),
            name: "Get Test Tenant".to_string(),
        };
        service.create_tenant(input).await.unwrap();

        // Then retrieve it
        let result = service.get_tenant("tenant-789").await.unwrap();
        assert!(result.is_some());
        let tenant = result.unwrap();
        assert_eq!(tenant.name, "Get Test Tenant");
    }

    #[tokio::test]
    async fn test_get_tenant_nonexistent() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo);

        let result = service.get_tenant("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_tenants() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo.clone());

        // Create multiple tenants
        service.create_tenant(CreateTenantInput {
            id: "tenant-1".to_string(),
            name: "Tenant One".to_string(),
        }).await.unwrap();

        service.create_tenant(CreateTenantInput {
            id: "tenant-2".to_string(),
            name: "Tenant Two".to_string(),
        }).await.unwrap();

        // List all tenants
        let result = service.list_tenants().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_tenant() {
        let repo = MockTenantRepository::new();
        let service = TenantService::new(repo.clone());

        // Create a tenant
        service.create_tenant(CreateTenantInput {
            id: "tenant-delete".to_string(),
            name: "Delete Me".to_string(),
        }).await.unwrap();

        // Delete it
        service.delete_tenant("tenant-delete").await.unwrap();

        // Verify it's gone
        let result = service.get_tenant("tenant-delete").await.unwrap();
        assert!(result.is_none());
    }
}
