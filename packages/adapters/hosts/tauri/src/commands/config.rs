use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub api_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            google_client_id: std::env::var("GOOGLE_CLIENT_ID")
                .unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_ID".to_string()),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_SECRET".to_string()),
            api_url: std::env::var("API_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
        }
    }
}

pub fn load_config() -> AppConfig {
    let cwd = std::env::current_dir().unwrap_or_default();
    let env_path = cwd
        .ancestors()
        .map(|dir| dir.join(".env"))
        .find(|path| path.is_file());

    if let Some(path) = env_path {
        if let Err(error) = dotenvy::from_path_override(&path) {
            tracing::warn!(?path, %error, "failed to load .env");
        } else {
            tracing::info!(?path, "loaded .env");
        }
    }

    AppConfig::default()
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    load_config()
}
