//! Outbox entry — the database record of an event to be published.
//!
//! This is written in the same transaction as the business data,
//! then processed asynchronously by the OutboxPublisher.

use chrono::{DateTime, Utc};
use contracts_events::AppEvent;
use serde::{Deserialize, Serialize};

/// Outbox entry status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    /// Not yet published — ready for processing.
    Pending,
    /// Successfully published to the event bus.
    Published,
    /// Failed to publish — will be retried.
    Failed,
}

/// A single outbox row.
///
/// ## Schema
/// ```sql
/// CREATE TABLE event_outbox (
///     id TEXT PRIMARY KEY,           -- UUID v7
///     event_type TEXT NOT NULL,      -- discriminant (e.g. "counter_changed")
///     event_payload TEXT NOT NULL,   -- JSON-serialized AppEvent
///     source_service TEXT NOT NULL,  -- which service emitted this
///     correlation_id TEXT,           -- tracing ID
///     status TEXT NOT NULL DEFAULT 'pending',
///     created_at TEXT NOT NULL DEFAULT (datetime('now')),
///     published_at TEXT              -- NULL until successfully published
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub id: String,
    pub event_type: String,
    pub event: AppEvent,
    pub source_service: String,
    pub correlation_id: Option<String>,
    pub status: OutboxStatus,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
}

impl OutboxEntry {
    /// Create a new pending outbox entry.
    pub fn pending(event: AppEvent, source_service: impl Into<String>) -> Self {
        let id = uuid::Uuid::now_v7().to_string();
        let event_type = event_type_name(&event);
        Self {
            id,
            event_type,
            event,
            source_service: source_service.into(),
            correlation_id: None,
            status: OutboxStatus::Pending,
            created_at: Utc::now(),
            published_at: None,
            retry_count: 0,
        }
    }

    /// Mark this entry as published.
    pub fn mark_published(mut self) -> Self {
        self.status = OutboxStatus::Published;
        self.published_at = Some(Utc::now());
        self
    }

    /// Mark this entry as failed and increment retry count.
    pub fn mark_failed(mut self) -> Self {
        self.status = OutboxStatus::Failed;
        self.retry_count += 1;
        self
    }
}

/// Extract a human-readable type name from an AppEvent.
fn event_type_name(event: &AppEvent) -> String {
    match event {
        AppEvent::TenantCreated(_) => "tenant.created".to_string(),
        AppEvent::TenantMemberAdded(_) => "tenant.member_added".to_string(),
        AppEvent::CounterChanged(_) => "counter.changed".to_string(),
        AppEvent::ChatMessageSent(_) => "chat.message_sent".to_string(),
    }
}

/// SQL for creating the outbox table (idempotent).
pub const OUTBOX_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS event_outbox (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    event_payload TEXT NOT NULL,
    source_service TEXT NOT NULL,
    correlation_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    published_at TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0
)";

/// SQL for inserting a pending outbox entry.
pub const INSERT_OUTBOX_SQL: &str =
    "INSERT INTO event_outbox (id, event_type, event_payload, source_service, correlation_id, status)
     VALUES (?, ?, ?, ?, ?, 'pending')";

/// SQL for selecting pending entries to process (ordered by creation time).
pub const SELECT_PENDING_SQL: &str =
    "SELECT id, event_type, event_payload, source_service, correlation_id, status, created_at, published_at, retry_count
     FROM event_outbox
     WHERE status IN ('pending', 'failed')
     AND retry_count < 5
     ORDER BY created_at ASC
     LIMIT 100";

/// SQL for marking an entry as published.
pub const MARK_PUBLISHED_SQL: &str =
    "UPDATE event_outbox SET status = 'published', published_at = datetime('now') WHERE id = ?";

/// SQL for marking an entry as failed.
pub const MARK_FAILED_SQL: &str =
    "UPDATE event_outbox SET status = 'failed', retry_count = retry_count + 1 WHERE id = ?";
