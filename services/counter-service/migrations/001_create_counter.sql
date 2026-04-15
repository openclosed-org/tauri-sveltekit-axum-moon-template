-- Counter service migrations
-- Tables: counter (with CAS version), counter_outbox, counter_idempotency
-- All migrations are idempotent (CREATE TABLE IF NOT EXISTS).

CREATE TABLE IF NOT EXISTS counter (
    tenant_id TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS counter_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    source_service TEXT NOT NULL DEFAULT 'counter-service',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    published INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_counter_outbox_pending
    ON counter_outbox(published, id);

CREATE TABLE IF NOT EXISTS counter_idempotency (
    idempotency_key TEXT PRIMARY KEY,
    result_value INTEGER NOT NULL,
    result_version INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
