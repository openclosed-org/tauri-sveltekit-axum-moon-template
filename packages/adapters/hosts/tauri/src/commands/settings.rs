//! Settings Tauri commands — bridge to SettingsService.

use feature_settings::{AgentConnectionSettings, SettingsService};
use settings_service::application::ApplicationSettingsService;
use settings_service::infrastructure::LibSqlSettingsRepository;
use storage_turso::EmbeddedTurso;
use tauri::Manager;

fn build_settings_service(
    db: EmbeddedTurso,
) -> ApplicationSettingsService<LibSqlSettingsRepository<EmbeddedTurso>> {
    let repo = LibSqlSettingsRepository::new(db);
    ApplicationSettingsService::new(repo)
}

/// Get user settings.
#[tauri::command]
pub async fn settings_get(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_settings_service(db);
    
    // TODO: Extract user_sub from auth context
    // For now, use a default user_sub (single-user mode)
    let user_sub = "default_user";
    
    match service.get_settings(user_sub).await {
        Ok(settings) => Ok(serde_json::json!({
            "api_key_masked": mask_api_key(&settings.api_key),
            "base_url": settings.base_url,
            "model": settings.model,
        })),
        Err(e) => Err(e.to_string()),
    }
}

/// Update agent connection settings.
#[tauri::command]
pub async fn settings_update(
    app: tauri::AppHandle,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<serde_json::Value, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_settings_service(db);
    
    // TODO: Extract user_sub from auth context
    let user_sub = "default_user";
    
    let new_settings = AgentConnectionSettings {
        api_key,
        base_url,
        model,
    };
    
    match service.update_agent_connection(user_sub, new_settings).await {
        Ok(settings) => Ok(serde_json::json!({
            "api_key_masked": mask_api_key(&settings.api_key),
            "base_url": settings.base_url,
            "model": settings.model,
        })),
        Err(e) => Err(e.to_string()),
    }
}

fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}
