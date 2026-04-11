//! Application layer — use cases for user and tenant management.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::domain::error::UserError;
use crate::ports::{TenantRepository, UserRepository, UserTenantRepository};

/// Input for tenant initialization.
#[derive(Debug, Clone)]
pub struct InitTenantInput {
    pub user_sub: String,
    pub user_name: String,
    pub email: Option<String>,
}

/// Result of tenant initialization.
#[derive(Debug, Clone)]
pub struct InitTenantResult {
    pub tenant_id: String,
    pub role: String,
    pub created: bool,
}

/// User service — handles user lifecycle and tenant initialization.
pub struct UserService<U, T, B>
where
    U: UserRepository,
    T: TenantRepository,
    B: UserTenantRepository,
{
    user_repo: Arc<U>,
    tenant_repo: Arc<T>,
    binding_repo: Arc<B>,
}

impl<U, T, B> UserService<U, T, B>
where
    U: UserRepository,
    T: TenantRepository,
    B: UserTenantRepository,
{
    pub fn new(user_repo: U, tenant_repo: T, binding_repo: B) -> Self {
        Self {
            user_repo: Arc::new(user_repo),
            tenant_repo: Arc::new(tenant_repo),
            binding_repo: Arc::new(binding_repo),
        }
    }

    /// Initialize tenant for a user.
    ///
    /// - First login: creates user + tenant + binding (role: 'owner')
    /// - Subsequent login: returns existing tenant_id, updates last_login
    pub async fn init_tenant(&self, input: InitTenantInput) -> Result<InitTenantResult, UserError> {
        if input.user_sub.is_empty() {
            return Err(UserError::InvalidInput("user_sub cannot be empty".to_string()));
        }
        if input.user_name.is_empty() {
            return Err(UserError::InvalidInput("user_name cannot be empty".to_string()));
        }

        // 1. Check existing binding
        if let Some(existing) = self.binding_repo.find_user_tenant(&input.user_sub).await? {
            // Update last login
            let _ = self.user_repo.update_last_login(&input.user_sub).await;

            return Ok(InitTenantResult {
                tenant_id: existing.tenant_id,
                role: existing.role,
                created: false,
            });
        }

        // 2. Create user if not exists
        if self.user_repo.find_by_sub(&input.user_sub).await?.is_none() {
            let user = crate::domain::User {
                id: generate_id(),
                user_sub: input.user_sub.clone(),
                display_name: input.user_name.clone(),
                email: input.email.clone(),
                created_at: Utc::now(),
                last_login_at: Some(Utc::now()),
            };
            self.user_repo.create_user(&user).await?;
        } else {
            let _ = self.user_repo.update_last_login(&input.user_sub).await;
        }

        // 3. Create tenant
        let tenant_id = self.tenant_repo.create_tenant(&input.user_name).await?;

        // 4. Create binding (owner role)
        let binding = self
            .binding_repo
            .create_binding(&input.user_sub, &tenant_id, "owner")
            .await?;

        Ok(InitTenantResult {
            tenant_id: binding.tenant_id,
            role: binding.role,
            created: true,
        })
    }
}

impl<U, T, B> Clone for UserService<U, T, B>
where
    U: UserRepository,
    T: TenantRepository,
    B: UserTenantRepository,
{
    fn clone(&self) -> Self {
        Self {
            user_repo: Arc::clone(&self.user_repo),
            tenant_repo: Arc::clone(&self.tenant_repo),
            binding_repo: Arc::clone(&self.binding_repo),
        }
    }
}

fn generate_id() -> String {
    // Simple hex ID generation — matches the SQL randomblob(16) pattern
    let bytes: [u8; 16] = std::array::from_fn(|_| rand::random::<u8>());
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
