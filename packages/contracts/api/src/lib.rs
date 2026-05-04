//! contracts/api — Route-level shared DTOs.
//! These Rust DTOs are the schema source for generated OpenAPI artifacts.

#![deny(unused_imports, unused_variables)]

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Server status: "ok" or "degraded"
    pub status: String,
}

impl HealthResponse {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}

/// Request body for tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct InitTenantRequest {
    /// OAuth provider's subject identifier.
    #[validate(length(min = 1, message = "user_sub is required"))]
    pub user_sub: String,
    /// Display name for the user.
    #[validate(length(min = 1, max = 100))]
    pub user_name: String,
}

/// Response from tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitTenantResponse {
    /// The tenant ID in "table:key" format.
    pub tenant_id: String,
    /// User's role within the tenant.
    pub role: String,
    /// Whether a new tenant was created.
    pub created: bool,
}

impl InitTenantResponse {
    pub fn new(tenant_id: impl Into<String>, role: impl Into<String>, created: bool) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            role: role.into(),
            created,
        }
    }
}

/// Generic counter operation response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CounterResponse {
    /// The current counter value after the operation.
    pub value: i64,
}

/// Current authenticated user profile.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: String,
    pub user_sub: String,
    pub display_name: String,
    pub email: Option<String>,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// Tenant binding visible to the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserTenantResponse {
    pub tenant_id: String,
    pub tenant_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_tenant_response_serializes_nullable_name() {
        let response = UserTenantResponse {
            tenant_id: "tenant-1".to_string(),
            tenant_name: None,
            role: "owner".to_string(),
            joined_at: "2026-05-02T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("tenant_name"));
        assert!(json.contains("null"));
    }
}
