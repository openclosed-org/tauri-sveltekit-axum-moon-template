use crate::sync::{SyncConfig, SyncEngine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Shared sync engine state managed by Tauri.
pub struct SyncState {
    pub engine: Arc<Mutex<SyncEngine>>,
}

/// Response for sync stats command.
#[derive(Debug, Serialize)]
pub struct SyncStatsResponse {
    pub last_sync: Option<String>,
    pub total_pushes: u64,
    pub total_pulls: u64,
    pub total_conflicts: u64,
    pub total_errors: u64,
    pub is_syncing: bool,
}

/// Request to configure sync settings.
#[derive(Debug, Deserialize)]
pub struct SyncConfigureRequest {
    pub sync_interval_secs: Option<u64>,
    pub push_batch_size: Option<u32>,
    pub pull_batch_size: Option<u32>,
    pub max_retries: Option<u32>,
    pub conflict_strategy: Option<String>,
    pub wal_checkpoint_threshold: Option<u64>,
}

/// Start background sync.
#[tauri::command]
pub async fn sync_start(state: State<'_, SyncState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    engine.start_background_sync(app);
    tracing::info!("background sync started via command");
    Ok(())
}

/// Stop background sync.
#[tauri::command]
pub async fn sync_stop(state: State<'_, SyncState>) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    engine.stop_background_sync();
    tracing::info!("background sync stopped via command");
    Ok(())
}

/// Trigger a single sync cycle.
#[tauri::command]
pub async fn sync_once(state: State<'_, SyncState>) -> Result<(), String> {
    let engine = state.engine.lock().await;
    engine
        .sync_once()
        .await
        .map_err(|e| format!("sync failed: {e}"))?;
    Ok(())
}

/// Get current sync statistics.
#[tauri::command]
pub async fn sync_get_stats(state: State<'_, SyncState>) -> Result<SyncStatsResponse, String> {
    let engine = state.engine.lock().await;
    let stats = engine.stats_handle().lock().await.clone();
    drop(engine);

    Ok(SyncStatsResponse {
        last_sync: stats.last_sync.map(|dt| dt.to_rfc3339()),
        total_pushes: stats.total_pushes,
        total_pulls: stats.total_pulls,
        total_conflicts: stats.total_conflicts,
        total_errors: stats.total_errors,
        is_syncing: stats.is_syncing,
    })
}

/// Update sync configuration.
#[tauri::command]
pub async fn sync_configure(
    state: State<'_, SyncState>,
    req: SyncConfigureRequest,
) -> Result<(), String> {
    let mut engine = state.engine.lock().await;

    let current_config = engine.config().clone();

    let strategy = match req.conflict_strategy.as_deref() {
        Some("last_push_wins") => crate::sync::ConflictStrategy::LastPushWins,
        Some("last_write_wins") => crate::sync::ConflictStrategy::LastWriteWins,
        Some("manual") => crate::sync::ConflictStrategy::Manual,
        Some(other) => return Err(format!("Unknown conflict strategy: {other}")),
        None => current_config.conflict_strategy,
    };

    let new_config = SyncConfig {
        sync_interval_secs: req
            .sync_interval_secs
            .unwrap_or(current_config.sync_interval_secs),
        push_batch_size: req
            .push_batch_size
            .unwrap_or(current_config.push_batch_size),
        pull_batch_size: req
            .pull_batch_size
            .unwrap_or(current_config.pull_batch_size),
        max_retries: req.max_retries.unwrap_or(current_config.max_retries),
        conflict_strategy: strategy,
        wal_checkpoint_threshold: req
            .wal_checkpoint_threshold
            .unwrap_or(current_config.wal_checkpoint_threshold),
    };

    engine.configure(new_config);
    tracing::info!("sync configuration updated");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats_response_serialization() {
        let response = SyncStatsResponse {
            last_sync: Some("2026-01-01T00:00:00Z".to_string()),
            total_pushes: 10,
            total_pulls: 5,
            total_conflicts: 0,
            total_errors: 1,
            is_syncing: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("total_pushes"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_sync_configure_request_partial() {
        let req = SyncConfigureRequest {
            sync_interval_secs: Some(60),
            push_batch_size: None,
            pull_batch_size: None,
            max_retries: None,
            conflict_strategy: None,
            wal_checkpoint_threshold: None,
        };

        assert_eq!(req.sync_interval_secs, Some(60));
        assert!(req.push_batch_size.is_none());
    }
}
