//! Application layer — agent conversation management and tool execution.
//!
//! SQL migrations for agent tables.
pub const AGENT_MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS conversations (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
    "CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES conversations(id), role TEXT NOT NULL, content TEXT NOT NULL, tool_calls TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
];
