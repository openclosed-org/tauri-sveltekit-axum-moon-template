//! Outbox relay worker configuration.
//!
//! Configuration is loaded via figment from environment variables with the `OUTBOX_` prefix.
//! All configuration comes from SOPS-encrypted secrets via Kubernetes, never from `.env` files.

use std::time::Duration;

use contracts_events::NATS_EVENT_SUBJECT_PREFIX;
use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};

/// Outbox relay worker configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Database URL for reading outbox events (e.g., "file:/data/web-bff.db" or "libsql://...")
    pub database_url: String,

    /// Remote Turso auth token when using a cloud libSQL database.
    pub turso_auth_token: Option<String>,

    /// NATS URL for publishing events (e.g., "nats://localhost:4222")
    pub nats_url: String,

    /// NATS subject prefix for published events (e.g., "events")
    pub nats_subject_prefix: String,

    /// Poll interval in milliseconds for checking new outbox entries
    pub poll_interval_ms: u64,

    /// Batch size for processing outbox entries per poll cycle
    pub batch_size: usize,

    /// Path to persist checkpoint state
    pub checkpoint_path: String,

    /// Health check server host
    pub health_host: String,

    /// Health check server port
    pub health_port: u16,

    /// Logging configuration (RUST_LOG)
    pub rust_log: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Environment variables are injected via SOPS/Kustomize/Flux.
    /// For local development without cluster, use:
    ///   `just sops-run outbox-relay-worker`
    pub fn from_env() -> anyhow::Result<Self> {
        let config: Config = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Env::prefixed("OUTBOX_").global())
            .extract()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "file:/data/web-bff.db".to_string(),
            turso_auth_token: None,
            nats_url: "nats://localhost:4222".to_string(),
            nats_subject_prefix: NATS_EVENT_SUBJECT_PREFIX.to_string(),
            poll_interval_ms: 500,
            batch_size: 100,
            checkpoint_path: "/data/outbox-relay-checkpoint.json".to_string(),
            health_host: "0.0.0.0".to_string(),
            health_port: 3030,
            rust_log: "info,outbox_relay_worker=debug".to_string(),
        }
    }
}

impl Config {
    /// Get the poll interval as a Duration.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    /// Get the health check address as a SocketAddr.
    pub fn health_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.health_host, self.health_port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:3030".parse().unwrap())
    }
}
