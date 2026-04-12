//! Application layer — settings use case orchestration.

use async_trait::async_trait;
use feature_settings::{AgentConnectionSettings as FeatureSettings, SettingsError, SettingsService as SettingsFeature};
use tracing::debug;

use crate::domain::entity::AgentConnectionSettings as EntitySettings;
use crate::ports::SettingsRepository;

/// Application-level settings service.
pub struct ApplicationSettingsService<R: SettingsRepository> {
    repo: R,
}

impl<R: SettingsRepository> ApplicationSettingsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

fn to_feature(s: EntitySettings) -> FeatureSettings {
    FeatureSettings {
        api_key: s.api_key,
        base_url: s.base_url,
        model: s.model,
    }
}

fn to_entity(s: FeatureSettings) -> EntitySettings {
    EntitySettings {
        api_key: s.api_key,
        base_url: s.base_url,
        model: s.model,
    }
}

#[async_trait]
impl<R: SettingsRepository> SettingsFeature for ApplicationSettingsService<R> {
    async fn get_settings(&self, user_sub: &str) -> Result<FeatureSettings, SettingsError> {
        let settings = self
            .repo
            .get_or_create(user_sub)
            .await
            .map_err(|e| SettingsError::Database(e.to_string()))?;
        Ok(to_feature(settings.agent_connection))
    }

    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: FeatureSettings,
    ) -> Result<FeatureSettings, SettingsError> {
        let entity_settings = to_entity(settings);
        let result = self
            .repo
            .update_agent_connection(user_sub, entity_settings)
            .await
            .map_err(|e| SettingsError::Database(e.to_string()))?;
        debug!(user_sub, "settings.agent_connection_updated");
        Ok(to_feature(result.agent_connection))
    }
}

/// SQL migration for the settings table (idempotent).
pub const SETTINGS_MIGRATION: &str =
    "CREATE TABLE IF NOT EXISTS settings (\
        user_sub TEXT PRIMARY KEY,\
        api_key TEXT NOT NULL DEFAULT '',\
        base_url TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',\
        model TEXT NOT NULL DEFAULT 'gpt-4o-mini',\
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))\
    )";
