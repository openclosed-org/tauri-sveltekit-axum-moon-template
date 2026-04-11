-- User service migrations — 001_create_user_tables

CREATE TABLE IF NOT EXISTS "user" (
    id TEXT PRIMARY KEY,
    user_sub TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    email TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_user_sub ON "user"(user_sub);
