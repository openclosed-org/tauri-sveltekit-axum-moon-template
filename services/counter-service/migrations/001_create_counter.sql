-- Counter service migrations
-- Tables: counter (with CAS version), event_outbox, counter_idempotency
-- All migrations are idempotent (CREATE TABLE IF NOT EXISTS).

CREATE TABLE IF NOT EXISTS counter (
    tenant_id TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Unified event outbox — shared by all services.
CREATE TABLE IF NOT EXISTS event_outbox (
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
);

CREATE INDEX IF NOT EXISTS idx_event_outbox_pending
    ON event_outbox(status, sequence);

CREATE TABLE IF NOT EXISTS counter_idempotency (
    idempotency_key TEXT PRIMARY KEY,
    result_value INTEGER NOT NULL,
    result_version INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
