//! Sync-reconciler worker — sync conflict resolution and reconciliation.
//!
//! Periodically compares data sources and reconciles differences
//! based on configured strategies.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use runtime::adapters::memory::MemoryLock;
use runtime::ports::Lock;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod conflict;
mod executors;
mod plans;

use executors::{ReconcileResult, ReconcileExecutor, StubReconcileExecutor};
use plans::{ReconciliationPlan, SyncStrategy};

/// Reconciler error types.
#[derive(Debug, thiserror::Error)]
pub enum ReconcilerError {
    #[error("Execution failed: {0}")]
    Execution(String),

    #[error("Conflict resolution failed: {0}")]
    ConflictResolution(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Worker state.
struct WorkerState {
    healthy: RwLock<bool>,
    reconcile_count: RwLock<u64>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            healthy: RwLock::new(true),
            reconcile_count: RwLock::new(0),
        }
    }

    async fn record_reconcile(&self) {
        let mut guard = self.reconcile_count.write().await;
        *guard += 1;
    }
}

async fn healthz(state: axum::extract::State<Arc<WorkerState>>) -> axum::Json<serde_json::Value> {
    let count = state.reconcile_count.read().await;
    axum::Json(serde_json::json!({
        "status": "ok",
        "reconcile_count": *count,
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
    info!("Sync-reconciler health server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sync_reconciler_worker=info".into()),
        )
        .init();

    info!("Sync-reconciler worker starting");

    let state = Arc::new(WorkerState::new());

    let health_addr: SocketAddr = "0.0.0.0:3034".parse()?;
    let health_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state, health_addr).await {
            warn!(error = %e, "health server failed");
        }
    });

    // Build reconciliation with stub plans
    let executor = StubReconcileExecutor;

    // Initialize runtime lock for conflict resolution
    let lock = MemoryLock::new();

    let plans = vec![
        ReconciliationPlan::new(
            "plan-settings-sync",
            "Sync settings across nodes",
            "primary-settings",
            "replica-settings",
            SyncStrategy::SourceWins,
        ),
    ];

    info!("Sync-reconciler running with runtime lock ({} plans)", plans.len());

    // Main reconciliation loop
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));
    loop {
        interval.tick().await;

        for plan in &plans {
            match executor.execute(plan).await {
                Ok(result) => {
                    info!(
                        plan_id = %result.plan_id,
                        success = result.success,
                        conflicts = result.conflicts_found,
                        "reconciliation complete"
                    );
                    state.record_reconcile().await;
                }
                Err(e) => {
                    warn!(plan_id = %plan.id, error = %e, "reconciliation failed");
                }
            }
        }
    }
}
