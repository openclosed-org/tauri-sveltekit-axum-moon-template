//! User service domain events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User-related domain events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserEvent {
    /// User was created.
    UserCreated(UserCreatedEvent),
    /// User logged in.
    UserLoggedIn(UserLoggedInEvent),
    /// User was updated.
    UserUpdated(UserUpdatedEvent),
    /// User was deleted.
    UserDeleted(UserDeletedEvent),
    /// Tenant was initialized for user.
    TenantInitialized(TenantInitializedEvent),
}

/// Event: User created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: String,
    pub user_sub: String,
    pub display_name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Event: User logged in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedInEvent {
    pub user_id: String,
    pub user_sub: String,
    pub login_at: DateTime<Utc>,
}

/// Event: User updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdatedEvent {
    pub user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Event: User deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    pub user_id: String,
    pub user_sub: String,
    pub deleted_at: DateTime<Utc>,
}

/// Event: Tenant initialized for user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInitializedEvent {
    pub user_id: String,
    pub user_sub: String,
    pub tenant_id: String,
    pub role: String,
    pub created: bool,
    pub initialized_at: DateTime<Utc>,
}
