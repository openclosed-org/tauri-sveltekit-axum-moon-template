//! Settings feature — user-level Agent connection preferences.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Agent connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConnectionSettings {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

/// Settings service trait.
#[async_trait]
pub trait SettingsService: Send + Sync {
    /// Get settings for a user (creates defaults if missing).
    async fn get_settings(
        &self,
        user_sub: &str,
    ) -> Result<crate::AgentConnectionSettings, SettingsError>;

    /// Update agent connection settings.
    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: AgentConnectionSettings,
    ) -> Result<crate::AgentConnectionSettings, SettingsError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Settings not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
}
