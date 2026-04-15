//! contracts/events — Domain event payload types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Tenant created event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "events/")]
pub struct TenantCreated {
    pub tenant_id: String,
    pub owner_sub: String,
}

/// Tenant member added event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "events/")]
pub struct TenantMemberAdded {
    pub tenant_id: String,
    pub user_sub: String,
    pub role: String,
}

/// Counter value changed event — emitted after a successful counter mutation.
///
/// ## Dedupe rule (per model.yaml)
/// `tenant_id + counter_key + version`
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "events/")]
pub struct CounterChanged {
    pub tenant_id: String,
    pub counter_key: String,
    pub operation: String,
    pub new_value: i64,
    pub delta: i64,
    pub version: i64,
}

/// Chat message sent event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "events/")]
pub struct ChatMessageSent {
    pub conversation_id: String,
    pub message_id: String,
    pub sender_id: String,
}

/// Unified application event envelope.
///
/// This is the single type that flows through the EventBus.
/// All services publish and consume via this enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    #[serde(rename = "tenant.created")]
    TenantCreated(TenantCreated),
    #[serde(rename = "tenant.member_added")]
    TenantMemberAdded(TenantMemberAdded),
    #[serde(rename = "counter.changed")]
    CounterChanged(CounterChanged),
    #[serde(rename = "chat.message_sent")]
    ChatMessageSent(ChatMessageSent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_tenant_created() {
        TenantCreated::export().unwrap();
    }

    #[test]
    fn export_tenant_member_added() {
        TenantMemberAdded::export().unwrap();
    }
}
