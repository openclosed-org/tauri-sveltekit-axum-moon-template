//! contracts/events — shared domain event payload types.
//!
//! Any event persisted to `event_outbox` or sent across process/deployable
//! boundaries must be represented here as `AppEvent` inside `EventEnvelope`.
//! Service-local orchestration events may remain in a service crate only when
//! they never cross HTTP/message/outbox boundaries.

#![deny(unused_imports, unused_variables)]

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const NATS_EVENT_SUBJECT_PREFIX: &str = "events";
pub const NATS_OUTBOX_TOPIC_PREFIX: &str = "outbox";
pub const PROJECTOR_QUEUE_GROUP: &str = "projector-worker";

/// Tenant created event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantCreated {
    pub tenant_id: String,
    pub owner_sub: String,
}

/// Tenant member added event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantMemberAdded {
    pub tenant_id: String,
    pub user_sub: String,
    pub role: String,
}

/// Counter value changed event — emitted after a successful counter mutation.
///
/// ## Dedupe rule (per model.yaml)
/// `tenant_id + counter_key + version`
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CounterChanged {
    pub tenant_id: String,
    pub counter_key: String,
    pub operation: CounterOperation,
    pub new_value: i64,
    pub delta: i64,
    pub version: i64,
}

/// Stable counter mutation operations used by `counter.changed` events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CounterOperation {
    Increment,
    Decrement,
    Reset,
}

/// Chat message sent event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

    /// Decode a canonical envelope or a legacy bare event payload.
    pub fn decode(
        payload: &str,
        source_service: impl Into<String>,
    ) -> Result<Self, EventEnvelopeError> {
        let source_service = source_service.into();

        serde_json::from_str::<EventEnvelope>(payload).or_else(|envelope_error| {
            serde_json::from_str::<AppEvent>(payload)
                .map(|event| EventEnvelope::new(event, source_service.clone()))
                .map_err(|app_event_error| EventEnvelopeError::Deserialize {
                    envelope_error: envelope_error.to_string(),
                    app_event_error: app_event_error.to_string(),
                })
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventEnvelopeError {
    #[error(
        "failed to deserialize event envelope: envelope={envelope_error}; app_event={app_event_error}"
    )]
    Deserialize {
        envelope_error: String,
        app_event_error: String,
    },
}

/// Unified application event envelope.
///
/// This is the single shared event type for cross-process/domain-event traffic.
/// All events written to `event_outbox` must be variants of this enum.
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
