//! Unified event outbox — the single source of truth for event persistence.
//!
//! Written in the same transaction as business data, then processed
//! asynchronously by the outbox-relay worker.
//!
//! ## Schema
//! The `event_outbox` table is the **default outbox truth source** for all
//! services. Every service writes here — no per-service private outbox tables.
//! - `sequence` (AUTOINCREMENT) ensures monotonic ordering for replay/checkpoint.
//! - `event_id` (UUID v7) provides global stable event identification.
//! - `status` / `retry_count` / `published_at` track delivery state.

use chrono::{DateTime, Utc};
use contracts_events::AppEvent;
use serde::{Deserialize, Serialize};

/// Outbox entry status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    /// Ready for processing.
    Pending,
    /// Successfully published.
    Published,
    /// Failed — eligible for retry (up to max_retries).
    Failed,
}

/// A single event_outbox row.
///
/// ## Schema
/// ```sql
/// CREATE TABLE event_outbox (
///     sequence INTEGER PRIMARY KEY AUTOINCREMENT,
///     event_id TEXT NOT NULL UNIQUE,
///     event_type TEXT NOT NULL,
///     event_payload TEXT NOT NULL,
///     source_service TEXT NOT NULL,
///     correlation_id TEXT,
///     status TEXT NOT NULL DEFAULT 'pending',
///     retry_count INTEGER NOT NULL DEFAULT 0,
///     created_at TEXT NOT NULL DEFAULT (datetime('now')),
///     published_at TEXT
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub sequence: i64,
    pub event_id: String,
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
        let event_id = uuid::Uuid::now_v7().to_string();
        let event_type = event_type_name(&event);
        Self {
            sequence: 0,
            event_id,
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

/// SQL for creating the unified event_outbox table (idempotent).
pub const OUTBOX_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS event_outbox (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    event_payload TEXT NOT NULL,
    source_service TEXT NOT NULL,
    correlation_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    published_at TEXT
)";

/// SQL for creating the pending-index (for relay polling).
pub const OUTBOX_PENDING_INDEX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_event_outbox_pending ON event_outbox(status, sequence)";

/// SQL for inserting a pending outbox entry.
pub const INSERT_OUTBOX_SQL: &str =
    "INSERT INTO event_outbox (event_id, event_type, event_payload, source_service, correlation_id, status)
     VALUES (?, ?, ?, ?, ?, 'pending')";

/// SQL for selecting pending entries to process (ordered by sequence).
pub const SELECT_PENDING_SQL: &str =
    "SELECT sequence, event_id, event_type, event_payload, source_service, correlation_id, status, created_at, published_at, retry_count
     FROM event_outbox
     WHERE status IN ('pending', 'failed')
     AND retry_count < 5
     ORDER BY sequence ASC
     LIMIT 100";

/// SQL for marking an entry as published.
pub const MARK_PUBLISHED_SQL: &str = "UPDATE event_outbox SET status = 'published', published_at = datetime('now') WHERE event_id = ?";

/// SQL for marking an entry as failed (increment retry_count).
pub const MARK_FAILED_SQL: &str =
    "UPDATE event_outbox SET status = 'failed', retry_count = retry_count + 1 WHERE event_id = ?";
