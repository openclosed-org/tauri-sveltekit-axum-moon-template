//! contracts/events — Domain event payload types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub const NATS_EVENT_SUBJECT_PREFIX: &str = "events";
pub const NATS_OUTBOX_TOPIC_PREFIX: &str = "outbox";
pub const PROJECTOR_QUEUE_GROUP: &str = "projector-worker";

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

/// Stable actor reference carried in event metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorRef {
    pub actor_id: String,
    pub actor_type: String,
    pub subject: Option<String>,
}

/// Stable resource reference carried in event metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource_type: String,
    pub resource_id: String,
}

/// Shared metadata that follows an event across sync and async paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventMetadata {
    pub event_type: String,
    pub schema_version: String,
    pub tenant_id: Option<String>,
    pub actor: Option<ActorRef>,
    pub resource: Option<ResourceRef>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

impl EventMetadata {
    pub fn for_event(event: &AppEvent) -> Self {
        let event_type = event_type_name(event).to_string();
        let schema_version = event_schema_version(event).to_string();
        match event {
            AppEvent::TenantCreated(event) => Self {
                event_type,
                schema_version,
                tenant_id: Some(event.tenant_id.clone()),
                actor: Some(ActorRef {
                    actor_id: event.owner_sub.clone(),
                    actor_type: "user".to_string(),
                    subject: Some(event.owner_sub.clone()),
                }),
                resource: Some(ResourceRef {
                    resource_type: "tenant".to_string(),
                    resource_id: event.tenant_id.clone(),
                }),
                trace_id: None,
                span_id: None,
                correlation_id: None,
                causation_id: None,
            },
            AppEvent::TenantMemberAdded(event) => Self {
                event_type,
                schema_version,
                tenant_id: Some(event.tenant_id.clone()),
                actor: Some(ActorRef {
                    actor_id: event.user_sub.clone(),
                    actor_type: "user".to_string(),
                    subject: Some(event.user_sub.clone()),
                }),
                resource: Some(ResourceRef {
                    resource_type: "tenant-member".to_string(),
                    resource_id: format!("{}:{}", event.tenant_id, event.user_sub),
                }),
                trace_id: None,
                span_id: None,
                correlation_id: None,
                causation_id: None,
            },
            AppEvent::CounterChanged(event) => Self {
                event_type,
                schema_version,
                tenant_id: Some(event.tenant_id.clone()),
                actor: None,
                resource: Some(ResourceRef {
                    resource_type: "counter".to_string(),
                    resource_id: event.counter_key.clone(),
                }),
                trace_id: None,
                span_id: None,
                correlation_id: None,
                causation_id: None,
            },
            AppEvent::ChatMessageSent(event) => Self {
                event_type,
                schema_version,
                tenant_id: None,
                actor: Some(ActorRef {
                    actor_id: event.sender_id.clone(),
                    actor_type: "user".to_string(),
                    subject: Some(event.sender_id.clone()),
                }),
                resource: Some(ResourceRef {
                    resource_type: "conversation-message".to_string(),
                    resource_id: format!("{}:{}", event.conversation_id, event.message_id),
                }),
                trace_id: None,
                span_id: None,
                correlation_id: None,
                causation_id: None,
            },
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }
}

/// Stable event identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub uuid::Uuid);

impl EventId {
    pub fn new_v7() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Canonical event envelope shared by outbox, relay, and consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub event: AppEvent,
    pub source_service: String,
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    pub fn new(event: AppEvent, source_service: impl Into<String>) -> Self {
        let metadata = EventMetadata::for_event(&event);
        Self {
            id: EventId::new_v7(),
            event,
            source_service: source_service.into(),
            metadata,
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_correlation_id(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_causation_id(causation_id);
        self
    }
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

pub fn event_type_name(event: &AppEvent) -> &'static str {
    match event {
        AppEvent::TenantCreated(_) => "tenant.created",
        AppEvent::TenantMemberAdded(_) => "tenant.member_added",
        AppEvent::CounterChanged(_) => "counter.changed",
        AppEvent::ChatMessageSent(_) => "chat.message_sent",
    }
}

pub fn event_schema_version(event: &AppEvent) -> &'static str {
    match event {
        AppEvent::TenantCreated(_) => "1.0.0",
        AppEvent::TenantMemberAdded(_) => "1.0.0",
        AppEvent::CounterChanged(_) => "1.0.0",
        AppEvent::ChatMessageSent(_) => "1.0.0",
    }
}

pub fn runtime_outbox_topic_for_type(event_type: &str) -> String {
    format!(
        "{}.{}",
        NATS_OUTBOX_TOPIC_PREFIX,
        normalize_event_type(event_type)
    )
}

fn normalize_event_type(event_type: &str) -> &str {
    event_type.trim_matches('.')
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
