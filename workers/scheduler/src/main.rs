//! Scheduler worker — time-based job dispatch.
//!
//! Evaluates cron expressions and dispatches registered jobs
//! at their scheduled intervals.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use runtime::adapters::memory::MemoryQueue;
use runtime::ports::Queue;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod dedupe;
mod dispatch;
mod jobs;

use dispatch::{JobDispatcher, LoggingExecutor};
use jobs::{JobRegistry, ScheduledJob};

/// Worker state.
struct WorkerState {
    healthy: RwLock<bool>,
    dispatch_count: RwLock<u64>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            dispatch_count: RwLock::new(0),
        }
    }

    async fn record_dispatch(&self) {
        let mut guard = self.dispatch_count.write().await;
        *guard += 1;
    }
}

async fn healthz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let dispatched = state.dispatch_count.read().await;
    axum::Json(serde_json::json!({
        "status": "ok",
        "dispatch_count": *dispatched,
    }))
}

async fn readyz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let healthy = state.healthy.read().await;
    if *healthy {
        axum::Json(serde_json::json!({ "status": "ready" }))
    } else {
        axum::Json(serde_json::json!({ "status": "not ready" }))
    }
}

async fn start_health_server(state: Arc<WorkerState>, addr: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Scheduler health server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scheduler_worker=info".into()),
        )
        .init();

    info!("Scheduler worker starting");

    let state = Arc::new(WorkerState::new());

    let health_addr: SocketAddr = "0.0.0.0:3033".parse()?;
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    // Build job registry with stub jobs
    let registry = JobRegistry::new();
    registry
        .register(ScheduledJob {
            id: "cleanup-tmp".to_string(),
            name: "Cleanup temporary data".to_string(),
            cron_expression: "0 */6 * * *".to_string(), // Every 6 hours
            enabled: true,
        })
        .await;

    let executor = LoggingExecutor;
    let dispatcher = JobDispatcher::new(executor);

    // Initialize runtime queue for job scheduling
    let job_queue = MemoryQueue::new();

    info!(
        "Scheduler worker running with runtime queue ({} jobs registered)",
        registry.enabled_jobs().await.len()
    );

    // Main scheduling loop — evaluate every minute
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;

        let jobs = registry.enabled_jobs().await;
        for job in &jobs {
            // In a real implementation, we'd check the cron expression
            // against the current time here. For the stub, we just log.
            info!(job_id = %job.id, "checking job schedule (stub: not actually dispatching)");
        }
    }
}
