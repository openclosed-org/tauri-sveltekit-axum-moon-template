//! Tenant service domain events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tenant-related domain events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TenantEvent {
    /// Tenant was created.
    TenantCreated(TenantCreatedEvent),
    /// Tenant was updated.
    TenantUpdated(TenantUpdatedEvent),
    /// Tenant was deleted.
    TenantDeleted(TenantDeletedEvent),
    /// Member was added to tenant.
    MemberAdded(MemberAddedEvent),
    /// Member was removed from tenant.
    MemberRemoved(MemberRemovedEvent),
    /// Member role was changed.
    MemberRoleChanged(MemberRoleChangedEvent),
}

/// Event: Tenant created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCreatedEvent {
    pub tenant_id: String,
    pub name: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

/// Event: Tenant updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUpdatedEvent {
    pub tenant_id: String,
    pub name: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Event: Tenant deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDeletedEvent {
    pub tenant_id: String,
    pub deleted_at: DateTime<Utc>,
}

/// Event: Member added to tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddedEvent {
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub added_at: DateTime<Utc>,
}

/// Event: Member removed from tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemovedEvent {
    pub tenant_id: String,
    pub user_id: String,
    pub removed_at: DateTime<Utc>,
}

/// Event: Member role changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRoleChangedEvent {
    pub tenant_id: String,
    pub user_id: String,
    pub old_role: String,
    pub new_role: String,
    pub changed_at: DateTime<Utc>,
}
