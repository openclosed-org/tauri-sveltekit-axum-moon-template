use figment::{Figment, providers::Env};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Env::prefixed("ADMIN_BFF_"))
            .extract()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 3020,
            cors_allowed_origins: vec!["http://localhost:5173".to_string()],
            jwt_secret: "dev-secret-change-in-production".to_string(),
        }
    }
}
