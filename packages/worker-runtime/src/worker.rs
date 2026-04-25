//! Minimal shared worker bootstrap, health, and shutdown helpers.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Shared worker health state exposed via `/healthz` and `/readyz`.
pub struct WorkerHealthState {
    healthy: RwLock<bool>,
    counters: RwLock<BTreeMap<String, u64>>,
}

impl WorkerHealthState {
    pub fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            counters: RwLock::new(BTreeMap::new()),
        }
    }

    pub async fn set_healthy(&self, value: bool) {
        let mut healthy = self.healthy.write().await;
        *healthy = value;
    }

    pub async fn record_count(&self, counter: &str, delta: usize) {
        let mut counters = self.counters.write().await;
        let entry = counters.entry(counter.to_string()).or_insert(0);
        *entry += delta as u64;
    }

    async fn snapshot(&self) -> (bool, BTreeMap<String, u64>) {
        let healthy = *self.healthy.read().await;
        let counters = self.counters.read().await.clone();
        (healthy, counters)
    }
}

impl Default for WorkerHealthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared bootstrap result used by worker binaries.
pub struct WorkerBootstrap {
    pub observability: observability::ObservabilityGuard,
    pub state: Arc<WorkerHealthState>,
}

/// Initialize observability and common health state for a worker process.
pub fn bootstrap_worker(
    service_name: &str,
    default_level: &str,
) -> anyhow::Result<WorkerBootstrap> {
    let observability = observability::init_observability(service_name, default_level)
        .map_err(anyhow::Error::msg)?;

    Ok(WorkerBootstrap {
        observability,
        state: Arc::new(WorkerHealthState::new()),
    })
}

async fn healthz(State(state): State<Arc<WorkerHealthState>>) -> Json<serde_json::Value> {
    let (healthy, counters) = state.snapshot().await;
    let mut payload = serde_json::Map::new();
    payload.insert(
        "status".to_string(),
        serde_json::Value::String(if healthy { "ok" } else { "unhealthy" }.to_string()),
    );
    for (key, value) in counters {
        payload.insert(
            key,
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }

    Json(serde_json::Value::Object(payload))
}

async fn readyz(State(state): State<Arc<WorkerHealthState>>) -> Json<serde_json::Value> {
    let (healthy, _) = state.snapshot().await;
    Json(serde_json::json!({
        "status": if healthy { "ready" } else { "not ready" }
    }))
}

async fn start_health_server(
    state: Arc<WorkerHealthState>,
    addr: SocketAddr,
    service_name: &str,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(service = service_name, %addr, "worker health server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Spawn the shared health server task for a worker.
pub fn spawn_health_server(
    state: Arc<WorkerHealthState>,
    addr: SocketAddr,
    service_name: impl Into<String>,
) -> tokio::task::JoinHandle<()> {
    let service_name = service_name.into();
    tokio::spawn(async move {
        if let Err(error) = start_health_server(state, addr, &service_name).await {
            warn!(service = service_name, error = %error, "worker health server failed");
        }
    })
}

/// Wait for a process shutdown signal.
pub async fn shutdown_signal() -> anyhow::Result<()> {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .map_err(|error| anyhow::anyhow!("failed to listen for ctrl-c: {error}"))
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|error| anyhow::anyhow!("failed to listen for SIGTERM: {error}"))?;
        signal.recv().await;
        Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<anyhow::Result<()>>();

    tokio::select! {
        result = ctrl_c => result,
        result = terminate => result,
    }
}
