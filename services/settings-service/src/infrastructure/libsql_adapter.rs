//! LibSQL implementation of SettingsRepository.
//!
//! Stores user-scoped settings (keyed by user_sub, NOT tenant_id).

use async_trait::async_trait;
use domain::ports::lib_sql::LibSqlPort;
use serde::Deserialize;

use crate::domain::{AgentConnectionSettings, UserSettings};
use crate::ports::{RepositoryError, SettingsRepository};

#[derive(Debug, Deserialize, Clone)]
struct SettingsRow {
    user_sub: String,
    api_key: String,
    base_url: String,
    model: String,
    updated_at: String,
}

/// LibSQL-backed SettingsRepository.
pub struct LibSqlSettingsRepository<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> LibSqlSettingsRepository<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    /// Run the settings table migration (idempotent).
    pub async fn migrate(&self) -> Result<(), RepositoryError> {
        self.port
            .execute(super::super::application::service::SETTINGS_MIGRATION, vec![])
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<P: LibSqlPort> SettingsRepository for LibSqlSettingsRepository<P> {
    async fn get_or_create(&self, user_sub: &str) -> Result<UserSettings, RepositoryError> {
        let rows: Vec<SettingsRow> = self
            .port
            .query(
                "SELECT user_sub, api_key, base_url, model, updated_at \
                 FROM settings WHERE user_sub = ? LIMIT 1",
                vec![user_sub.to_string()],
            )
            .await?;

        if let Some(row) = rows.first() {
            return Ok(row_to_settings(row.clone()));
        }

        // Create default settings for new user
        let defaults = UserSettings::new(user_sub);
        self.port
            .execute(
                "INSERT INTO settings (user_sub, api_key, base_url, model, updated_at) \
                 VALUES (?, ?, ?, ?, ?)",
                vec![
                    user_sub.to_string(),
                    defaults.agent_connection.api_key.clone(),
                    defaults.agent_connection.base_url.clone(),
                    defaults.agent_connection.model.clone(),
                    defaults.updated_at.clone(),
                ],
            )
            .await?;

        Ok(defaults)
    }

    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: AgentConnectionSettings,
    ) -> Result<UserSettings, RepositoryError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.port
            .execute(
                "UPDATE settings SET api_key = ?, base_url = ?, model = ?, updated_at = ? \
                 WHERE user_sub = ?",
                vec![
                    settings.api_key.clone(),
                    settings.base_url.clone(),
                    settings.model.clone(),
                    now.clone(),
                    user_sub.to_string(),
                ],
            )
            .await?;

        Ok(UserSettings {
            user_sub: user_sub.to_string(),
            agent_connection: settings,
            updated_at: now,
        })
    }
}

fn row_to_settings(row: SettingsRow) -> UserSettings {
    UserSettings {
        user_sub: row.user_sub,
        agent_connection: AgentConnectionSettings {
            api_key: row.api_key,
            base_url: row.base_url,
            model: row.model,
        },
        updated_at: row.updated_at,
    }
}
