//! Tenant domain entities and value objects.

use serde::{Deserialize, Serialize};

/// Tenant entity — represents a multi-tenant isolation boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Input for creating a new tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantInput {
    pub id: String,
    pub name: String,
}
