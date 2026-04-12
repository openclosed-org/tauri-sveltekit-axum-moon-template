//! User-scoped settings entity.
//!
//! Represents a user's personal Agent connection configuration.
//! Stored per-user (user_sub), not per-tenant.

use serde::{Deserialize, Serialize};

/// Complete user settings record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// OAuth subject identifier — the primary key.
    pub user_sub: String,
    /// LLM API connection settings.
    pub agent_connection: AgentConnectionSettings,
    /// RFC3339 timestamp of last update.
    pub updated_at: String,
}

/// Agent API connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConnectionSettings {
    /// API key (stored as-is; masking is a presentation concern).
    pub api_key: String,
    /// Base URL for the LLM API.
    pub base_url: String,
    /// Model name (e.g., "gpt-4o-mini").
    pub model: String,
}

impl Default for AgentConnectionSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
        }
    }
}

impl UserSettings {
    /// Create new settings with defaults for a given user.
    pub fn new(user_sub: &str) -> Self {
        Self {
            user_sub: user_sub.to_string(),
            agent_connection: AgentConnectionSettings::default(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
