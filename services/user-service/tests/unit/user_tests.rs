//! User service unit tests.

#[cfg(test)]
mod user_tests {
    use chrono::Utc;

    use crate::application::{InitTenantInput, UserService};
    use crate::domain::User;
    use crate::domain::error::UserError;
    use crate::ports::{TenantRepository, UserRepository, UserTenantRepository};

    /// Mock user repository for testing.
    struct MockUserRepository {
        users: std::sync::Mutex<Vec<User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_sub(&self, user_sub: &str) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.user_sub == user_sub).cloned())
        }

        async fn create_user(&self, user: &User) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            users.push(user.clone());
            Ok(())
        }

        async fn update_last_login(&self, _user_sub: &str) -> Result<(), UserError> {
            Ok(())
        }
    }

    /// Mock tenant repository for testing.
    struct MockTenantRepository {
        tenants: std::sync::Mutex<Vec<crate::domain::Tenant>>,
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
        async fn create_tenant(&self, name: &str) -> Result<String, UserError> {
            let id = format!("tenant-{}", uuid::Uuid::new_v4());
            let tenant = crate::domain::Tenant {
                id: id.clone(),
                name: name.to_string(),
                created_at: Utc::now(),
            };
            let mut tenants = self.tenants.lock().unwrap();
            tenants.push(tenant);
            Ok(id)
        }

        async fn find_by_id(&self, tenant_id: &str) -> Result<Option<crate::domain::Tenant>, UserError> {
            let tenants = self.tenants.lock().unwrap();
            Ok(tenants.iter().find(|t| t.id == tenant_id).cloned())
        }
    }

    /// Mock user-tenant repository for testing.
    struct MockUserTenantRepository {
        bindings: std::sync::Mutex<Vec<crate::domain::UserTenantBinding>>,
    }

    impl MockUserTenantRepository {
        fn new() -> Self {
            Self {
                bindings: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserTenantRepository for MockUserTenantRepository {
        async fn find_user_tenant(
            &self,
            user_sub: &str,
        ) -> Result<Option<crate::domain::UserTenantBinding>, UserError> {
            let bindings = self.bindings.lock().unwrap();
            Ok(bindings.iter().find(|b| b.user_sub == user_sub).cloned())
        }

        async fn create_binding(
            &self,
            user_sub: &str,
            tenant_id: &str,
            role: &str,
        ) -> Result<crate::domain::UserTenantBinding, UserError> {
            let binding = crate::domain::UserTenantBinding {
                id: format!("binding-{}", uuid::Uuid::new_v4()),
                user_sub: user_sub.to_string(),
                tenant_id: tenant_id.to_string(),
                role: role.to_string(),
                joined_at: Utc::now(),
            };
            let mut bindings = self.bindings.lock().unwrap();
            bindings.push(binding.clone());
            Ok(binding)
        }
    }

    #[tokio::test]
    async fn test_init_tenant_first_login() {
        let user_repo = MockUserRepository::new();
        let tenant_repo = MockTenantRepository::new();
        let binding_repo = MockUserTenantRepository::new();

        let service = UserService::new(user_repo, tenant_repo, binding_repo);

        let input = InitTenantInput {
            user_sub: "test-user-123".to_string(),
            user_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
        };

        let result = service.init_tenant(input).await.unwrap();

        assert!(result.created);
        assert_eq!(result.role, "owner");
        assert!(!result.tenant_id.is_empty());
    }

    #[tokio::test]
    async fn test_init_tenant_subsequent_login() {
        let user_repo = MockUserRepository::new();
        let tenant_repo = MockTenantRepository::new();
        let binding_repo = MockUserTenantRepository::new();

        // First login - creates tenant
        let service = UserService::new(
            user_repo.clone(),
            tenant_repo.clone(),
            binding_repo.clone(),
        );

        let input = InitTenantInput {
            user_sub: "test-user-456".to_string(),
            user_name: "Test User 2".to_string(),
            email: None,
        };

        let first_result = service.init_tenant(input).await.unwrap();
        assert!(first_result.created);
        let tenant_id = first_result.tenant_id.clone();

        // Second login - should return existing tenant
        let input2 = InitTenantInput {
            user_sub: "test-user-456".to_string(),
            user_name: "Test User 2".to_string(),
            email: None,
        };

        let second_result = service.init_tenant(input2).await.unwrap();
        assert!(!second_result.created);
        assert_eq!(second_result.tenant_id, tenant_id);
        assert_eq!(second_result.role, "owner");
    }

    #[tokio::test]
    async fn test_init_tenant_empty_user_sub() {
        let user_repo = MockUserRepository::new();
        let tenant_repo = MockTenantRepository::new();
        let binding_repo = MockUserTenantRepository::new();

        let service = UserService::new(user_repo, tenant_repo, binding_repo);

        let input = InitTenantInput {
            user_sub: "".to_string(),
            user_name: "Test User".to_string(),
            email: None,
        };

        let result = service.init_tenant(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UserError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_init_tenant_empty_user_name() {
        let user_repo = MockUserRepository::new();
        let tenant_repo = MockTenantRepository::new();
        let binding_repo = MockUserTenantRepository::new();

        let service = UserService::new(user_repo, tenant_repo, binding_repo);

        let input = InitTenantInput {
            user_sub: "test-user-789".to_string(),
            user_name: "".to_string(),
            email: None,
        };

        let result = service.init_tenant(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UserError::InvalidInput(_)));
    }
}
