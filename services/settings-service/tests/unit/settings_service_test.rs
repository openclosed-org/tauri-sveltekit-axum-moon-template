//! Unit tests for ApplicationSettingsService using mock repository.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use feature_settings::SettingsService;
use settings_service::application::ApplicationSettingsService;
use settings_service::domain::{AgentConnectionSettings, UserSettings};
use settings_service::ports::{RepositoryError, SettingsRepository};

/// In-memory mock repository.
struct MockSettingsRepository {
    store: Arc<Mutex<Vec<UserSettings>>>,
}

impl MockSettingsRepository {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[async_trait]
impl SettingsRepository for MockSettingsRepository {
    async fn get_or_create(&self, user_sub: &str) -> Result<UserSettings, RepositoryError> {
        let store = self.store.lock().await;
        if let Some(existing) = store.iter().find(|s| s.user_sub == user_sub) {
            return Ok(existing.clone());
        }
        drop(store);

        let defaults = UserSettings::new(user_sub);
        self.store.lock().await.push(defaults.clone());
        Ok(defaults)
    }

    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: AgentConnectionSettings,
    ) -> Result<UserSettings, RepositoryError> {
        let mut store = self.store.lock().await;
        if let Some(existing) = store.iter_mut().find(|s| s.user_sub == user_sub) {
            existing.agent_connection = settings.clone();
            existing.updated_at = chrono::Utc::now().to_rfc3339();
            return Ok(existing.clone());
        }

        let new_settings = UserSettings {
            user_sub: user_sub.to_string(),
            agent_connection: settings,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        store.push(new_settings.clone());
        Ok(new_settings)
    }
}

#[tokio::test]
async fn get_or_create_returns_defaults_for_new_user() {
    let repo = MockSettingsRepository::new();
    let service = ApplicationSettingsService::new(repo);

    let settings = service.get_settings("user-new").await.unwrap();
    assert_eq!(settings.base_url, "https://api.openai.com/v1");
    assert_eq!(settings.model, "gpt-4o-mini");
    assert!(settings.api_key.is_empty());
}

#[tokio::test]
async fn update_and_retrieve_settings() {
    let repo = MockSettingsRepository::new();
    let service = ApplicationSettingsService::new(repo);

    let updated = service
        .update_agent_connection(
            "user-1",
            AgentConnectionSettings {
                api_key: "sk-test-key".to_string(),
                base_url: "https://api.example.com/v1".to_string(),
                model: "claude-3-opus".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.api_key, "sk-test-key");
    assert_eq!(updated.base_url, "https://api.example.com/v1");
    assert_eq!(updated.model, "claude-3-opus");

    let retrieved = service.get_settings("user-1").await.unwrap();
    assert_eq!(retrieved.model, "claude-3-opus");
}

#[tokio::test]
async fn user_isolation_settings_dont_leak() {
    let repo = MockSettingsRepository::new();
    let service = ApplicationSettingsService::new(repo);

    // User A updates settings
    service
        .update_agent_connection(
            "user-a",
            AgentConnectionSettings {
                api_key: "sk-a".to_string(),
                base_url: "https://a.example.com".to_string(),
                model: "model-a".to_string(),
            },
        )
        .await
        .unwrap();

    // User B should still have defaults
    let b_settings = service.get_settings("user-b").await.unwrap();
    assert!(b_settings.api_key.is_empty());
    assert_eq!(b_settings.base_url, "https://api.openai.com/v1");

    // User A should have their custom settings
    let a_settings = service.get_settings("user-a").await.unwrap();
    assert_eq!(a_settings.api_key, "sk-a");
    assert_eq!(a_settings.base_url, "https://a.example.com");
}
