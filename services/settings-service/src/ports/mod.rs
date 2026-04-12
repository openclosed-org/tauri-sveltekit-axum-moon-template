//! Settings repository port — abstract storage interface.

use async_trait::async_trait;

/// Error type for repository operations.
pub type RepositoryError = Box<dyn std::error::Error + Send + Sync>;

/// Abstract repository interface for settings.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Get settings for a user, creating defaults if missing.
    async fn get_or_create(
        &self,
        user_sub: &str,
    ) -> Result<crate::domain::UserSettings, RepositoryError>;

    /// Update agent connection settings for a user.
    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: crate::domain::AgentConnectionSettings,
    ) -> Result<crate::domain::UserSettings, RepositoryError>;
}
