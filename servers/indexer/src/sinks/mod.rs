//! Event sinks — write indexed events to storage.

use async_trait::async_trait;
use contracts_events::AppEvent;

use crate::IndexerError;
use crate::IndexedEvent;

/// Event sink trait — writes events to a storage layer.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Name of this sink (e.g., "turso-events").
    fn name(&self) -> &str;

    /// Write an event to the sink.
    async fn write(&self, event: &AppEvent) -> Result<(), IndexerError>;
}

/// In-memory stub sink for testing — collects events.
pub struct MemoryEventSink {
    pub events: tokio::sync::Mutex<Vec<IndexedEvent>>,
}

impl MemoryEventSink {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MemoryEventSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventSink for MemoryEventSink {
    fn name(&self) -> &str {
        "memory-sink"
    }

    async fn write(&self, event: &AppEvent) -> Result<(), IndexerError> {
        let event_type = match event {
            AppEvent::TenantCreated(_) => "tenant.created",
            AppEvent::TenantMemberAdded(_) => "tenant.member_added",
            AppEvent::CounterChanged(_) => "counter.changed",
            AppEvent::ChatMessageSent(_) => "chat.message_sent",
        };

        let indexed = IndexedEvent {
            id: uuid::Uuid::now_v7().to_string(),
            event_type: event_type.to_string(),
            tenant_id: extract_tenant_id(event),
            user_sub: extract_user_sub(event),
            payload: serde_json::to_string(event)
                .map_err(|e| IndexerError::Internal(format!("serialize failed: {}", e)))?,
            indexed_at: chrono::Utc::now().to_rfc3339(),
        };

        self.events.lock().await.push(indexed);
        Ok(())
    }
}

fn extract_tenant_id(event: &AppEvent) -> Option<String> {
    match event {
        AppEvent::TenantCreated(e) => Some(e.tenant_id.clone()),
        AppEvent::TenantMemberAdded(e) => Some(e.tenant_id.clone()),
        AppEvent::CounterChanged(e) => Some(e.tenant_id.clone()),
        AppEvent::ChatMessageSent(_) => None,
    }
}

fn extract_user_sub(event: &AppEvent) -> Option<String> {
    match event {
        AppEvent::TenantCreated(e) => Some(e.owner_sub.clone()),
        AppEvent::TenantMemberAdded(e) => Some(e.user_sub.clone()),
        AppEvent::CounterChanged(_) => None,
        AppEvent::ChatMessageSent(e) => Some(e.sender_id.clone()),
    }
}

/// Turso-based event sink — writes to the embedded database.
pub struct TursoEventSink<P: domain::ports::lib_sql::LibSqlPort> {
    db: P,
}

impl<P: domain::ports::lib_sql::LibSqlPort> TursoEventSink<P> {
    pub fn new(db: P) -> Self {
        Self { db }
    }
}
