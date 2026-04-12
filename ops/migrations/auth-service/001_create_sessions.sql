-- Auth service migrations
-- Session management tables

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    user_sub TEXT NOT NULL,
    tenant_id TEXT,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_address TEXT,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_sub ON sessions(user_sub);
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_id ON sessions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
