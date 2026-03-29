//! Axum shared state — holds database connections, cache, and HTTP client.
//!
//! Injected via Router::with_state() so all route handlers can access
//! DB, cache, and HTTP client through the State extractor.

use moka::future::Cache;
use std::time::Duration;
use surrealdb::{engine::any::Any, Surreal};

/// Shared application state for Axum routes.
///
/// All fields are cheaply cloneable (Arc-wrapped internally).
#[derive(Clone)]
pub struct AppState {
    /// SurrealDB connection pool (server-side database).
    pub db: Surreal<Any>,

    /// Moka in-memory cache (replaces Redis per D-10/D-12).
    /// - 10,000 entries max
    /// - 5-minute TTL per entry
    pub cache: Cache<String, String>,

    /// Shared reqwest HTTP client (connection pool, per D-13/D-14).
    /// - 30s timeout
    /// - 10 max idle connections per host
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Initialize AppState with in-memory SurrealDB for development.
    ///
    /// Production: use `rocksdb://path` or remote SurrealDB endpoint.
    pub async fn new_dev() -> Result<Self, Box<dyn std::error::Error>> {
        // SurrealDB — in-memory for dev (per D-04)
        let db = Surreal::<Any>::init();
        db.connect("memory").await?;
        db.use_ns("app").use_db("main").await?;

        // Moka cache — 10k entries, 5min TTL (per D-10/D-11)
        let cache: Cache<String, String> = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build();

        // reqwest client — 30s timeout, 10 idle per host (per D-13/D-14)
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self {
            db,
            cache,
            http_client,
        })
    }
}
