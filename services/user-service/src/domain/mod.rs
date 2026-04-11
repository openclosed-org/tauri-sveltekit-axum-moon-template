//! User domain entities and value objects.

pub mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity — represents an authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub user_sub: String,       // OAuth subject identifier
    pub display_name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// User-tenant binding — maps a user to a tenant with a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTenantBinding {
    pub id: String,
    pub user_sub: String,
    pub tenant_id: String,
    pub role: String,           // "owner", "member", "admin"
    pub joined_at: DateTime<Utc>,
}

/// Tenant entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
