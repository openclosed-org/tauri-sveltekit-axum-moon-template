//! Projector worker configuration.

use contracts_events::{NATS_EVENT_SUBJECT_PREFIX, PROJECTOR_QUEUE_GROUP};
use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub turso_auth_token: Option<String>,
    pub nats_url: Option<String>,
    pub nats_subject: String,
    pub nats_queue_group: String,
    pub poll_interval_ms: u64,
    pub batch_size: usize,
    pub checkpoint_path: String,
    pub health_host: String,
    pub health_port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let config: Config = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Env::prefixed("PROJECTOR_").global())
            .extract()?;
        Ok(config)
    }

    pub fn health_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.health_host, self.health_port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:3050".parse().unwrap())
    }

    pub fn poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.poll_interval_ms)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "file:/data/web-bff.db".to_string(),
            turso_auth_token: None,
            nats_url: None,
            nats_subject: format!("{}.counter.changed", NATS_EVENT_SUBJECT_PREFIX),
            nats_queue_group: PROJECTOR_QUEUE_GROUP.to_string(),
            poll_interval_ms: 500,
            batch_size: 100,
            checkpoint_path: "/data/projector-checkpoint.json".to_string(),
            health_host: "0.0.0.0".to_string(),
            health_port: 3050,
            rust_log: "info,projector_worker=debug".to_string(),
        }
    }
}
