use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub jwt_secret: String,
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<figment::Error>> {
        Figment::new()
            .merge(Env::prefixed("ADMIN_BFF_"))
            .extract()
            .map_err(Box::new)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 3020,
            cors_allowed_origins: vec!["http://localhost:5173".to_string()],
            jwt_secret: "dev-secret-change-in-production".to_string(),
            database_url: Some("file:data/admin-bff.db".to_string()),
        }
    }
}
