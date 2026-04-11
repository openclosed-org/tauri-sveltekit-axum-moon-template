//! Unit tests for user service.

use async_trait::async_trait;
use chrono::Utc;

use user_service::domain;
use user_service::domain::error::UserError;
use user_service::ports::{TenantRepository, UserRepository, UserTenantRepository};

// ── Mock implementations ─────────────────────────────────────

struct MockUserRepository {
    users: std::sync::Mutex<Vec<domain::User>>,
}

impl MockUserRepository {
    fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn find_by_sub(&self, user_sub: &str) -> Result<Option<domain::User>, UserError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.user_sub == user_sub).cloned())
    }

    async fn create_user(&self, user: &domain::User) -> Result<(), UserError> {
        self.users.lock().unwrap().push(user.clone());
        Ok(())
    }

    async fn update_last_login(&self, _user_sub: &str) -> Result<(), UserError> {
        Ok(())
    }
}

struct MockTenantRepository;

#[async_trait]
impl TenantRepository for MockTenantRepository {
    async fn create_tenant(&self, name: &str) -> Result<String, UserError> {
        Ok(format!("tenant-{}", name.to_lowercase()))
    }

    async fn find_by_id(&self, _tenant_id: &str) -> Result<Option<domain::Tenant>, UserError> {
        Ok(None)
    }
}

struct MockUserTenantRepository {
    bindings: std::sync::Mutex<Vec<domain::UserTenantBinding>>,
}

impl MockUserTenantRepository {
    fn new() -> Self {
        Self {
            bindings: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl UserTenantRepository for MockUserTenantRepository {
    async fn find_user_tenant(
        &self,
        user_sub: &str,
    ) -> Result<Option<domain::UserTenantBinding>, UserError> {
        let bindings = self.bindings.lock().unwrap();
        Ok(bindings
            .iter()
            .find(|b| b.user_sub == user_sub)
            .cloned())
    }

    async fn create_binding(
        &self,
        user_sub: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<domain::UserTenantBinding, UserError> {
        let binding = domain::UserTenantBinding {
            id: format!("binding-{}", user_sub),
            user_sub: user_sub.to_string(),
            tenant_id: tenant_id.to_string(),
            role: role.to_string(),
            joined_at: Utc::now(),
        };
        self.bindings.lock().unwrap().push(binding.clone());
        Ok(binding)
    }
}

// ── Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_init_tenant_creates_new() {
    use user_service::application::{InitTenantInput, UserService};

    let user_repo = MockUserRepository::new();
    let tenant_repo = MockTenantRepository;
    let binding_repo = MockUserTenantRepository::new();

    let service = UserService::new(user_repo, tenant_repo, binding_repo);

    let input = InitTenantInput {
        user_sub: "google-123".to_string(),
        user_name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
    };

    let result = service.init_tenant(input).await.unwrap();
    assert!(result.created);
    assert_eq!(result.role, "owner");
    assert_eq!(result.tenant_id, "tenant-alice");
}

#[tokio::test]
async fn test_init_tenant_returns_existing() {
    use user_service::application::{InitTenantInput, UserService};

    let user_repo = MockUserRepository::new();
    let tenant_repo = MockTenantRepository;
    let binding_repo = MockUserTenantRepository::new();

    // Pre-create a binding
    binding_repo
        .create_binding("google-456", "tenant-existing", "owner")
        .await
        .unwrap();

    let service = UserService::new(user_repo, tenant_repo, binding_repo);

    let input = InitTenantInput {
        user_sub: "google-456".to_string(),
        user_name: "Bob".to_string(),
        email: None,
    };

    let result = service.init_tenant(input).await.unwrap();
    assert!(!result.created);
    assert_eq!(result.tenant_id, "tenant-existing");
}

#[tokio::test]
async fn test_init_tenant_rejects_empty_sub() {
    use user_service::application::{InitTenantInput, UserService};

    let user_repo = MockUserRepository::new();
    let tenant_repo = MockTenantRepository;
    let binding_repo = MockUserTenantRepository::new();

    let service = UserService::new(user_repo, tenant_repo, binding_repo);

    let input = InitTenantInput {
        user_sub: "".to_string(),
        user_name: "Charlie".to_string(),
        email: None,
    };

    let result = service.init_tenant(input).await;
    assert!(result.is_err());
}
