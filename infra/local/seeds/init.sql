-- Seed data for local development
-- This file contains initial data for testing and development

-- ── Users ────────────────────────────────────────────────────
-- Note: In production, users are created via OAuth flow.
-- These are test users for development only.

-- ── Tenants ──────────────────────────────────────────────────
-- Default tenant for testing
INSERT OR IGNORE INTO tenant (id, name, created_at)
VALUES ('dev-tenant', 'Development Tenant', datetime('now'));

-- ── User-Tenant Bindings ────────────────────────────────────
-- Bind test user to dev tenant
INSERT OR IGNORE INTO user_tenant (id, user_sub, tenant_id, role, joined_at)
VALUES ('binding-1', 'dev-user-sub', 'dev-tenant', 'owner', datetime('now'));

-- ── Sessions ─────────────────────────────────────────────────
-- No seed data needed - sessions are created via auth flow

-- ── Settings ─────────────────────────────────────────────────
-- Default settings for dev tenant
INSERT OR IGNORE INTO settings (tenant_id, key, value, created_at, updated_at)
VALUES 
    ('dev-tenant', 'theme', 'dark', datetime('now'), datetime('now')),
    ('dev-tenant', 'language', 'en', datetime('now'), datetime('now')),
    ('dev-tenant', 'timezone', 'UTC', datetime('now'), datetime('now'));
