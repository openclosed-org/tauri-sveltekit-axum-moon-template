use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    // 尝试从项目根目录加载.env文件
    let config_paths = [
        PathBuf::from(".env"),
        PathBuf::from("../.env"),
        PathBuf::from("../../.env"),
    ];

    for path in &config_paths {
        if path.exists() {
            if let Err(e) = dotenvy::dotenv_override() {
                tracing::warn!(%e, "failed to load .env");
            } else {
                tracing::info!(?path, "loaded .env");
                break;
            }
        }
    }

    AppConfig::default()
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    load_config()
}
