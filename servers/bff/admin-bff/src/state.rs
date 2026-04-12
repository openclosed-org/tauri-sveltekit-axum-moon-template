use reqwest::Client;
use std::sync::Arc;

use crate::config::Config;

/// Admin BFF application state
#[derive(Clone)]
pub struct AdminBffState {
    pub config: Arc<Config>,
    pub http_client: Client,
}

impl AdminBffState {
    pub async fn new_with_config(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config: Arc::new(config.clone()),
            http_client,
        })
    }
}
