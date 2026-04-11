//! Stable contract definitions — DTOs and events for user service.

use serde::{Deserialize, Serialize};

/// Request to initialize a tenant for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitTenantRequest {
    pub user_sub: String,
    pub user_name: String,
}

/// Response from tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitTenantResponse {
    pub tenant_id: String,
    pub role: String,
    pub created: bool,
}
