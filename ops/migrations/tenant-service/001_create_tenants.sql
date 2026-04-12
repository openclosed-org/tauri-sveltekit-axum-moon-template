-- Tenant service migrations
-- Tenant and user-tenant binding tables

CREATE TABLE IF NOT EXISTS tenant (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS user_tenant (
    id TEXT PRIMARY KEY,
    user_sub TEXT NOT NULL UNIQUE,
    tenant_id TEXT NOT NULL REFERENCES tenant(id),
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_tenant_tenant_id ON user_tenant(tenant_id);
CREATE INDEX IF NOT EXISTS idx_user_tenant_user_sub ON user_tenant(user_sub);
