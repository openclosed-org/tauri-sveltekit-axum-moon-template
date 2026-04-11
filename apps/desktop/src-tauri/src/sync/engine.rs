use crate::sync::conflict::{ConflictRecord, ConflictStrategy, resolve_conflict};
use chrono::{DateTime, Utc};
use domain::ports::lib_sql::LibSqlPort;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use storage_turso::EmbeddedTurso;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::time;

/// Configuration for the sync engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// How often to run sync in seconds (default: 30).
    pub sync_interval_secs: u64,
    /// Number of frames to push per batch (default: 128).
    pub push_batch_size: u32,
    /// Number of frames to pull per batch (default: 128).
    pub pull_batch_size: u32,
    /// Maximum retry attempts for network errors (default: 5).
    pub max_retries: u32,
    /// Conflict resolution strategy.
    pub conflict_strategy: ConflictStrategy,
    /// WAL size threshold (in frames) before triggering checkpoint.
    pub wal_checkpoint_threshold: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_secs: 30,
            push_batch_size: 128,
            pull_batch_size: 128,
            max_retries: 5,
            conflict_strategy: ConflictStrategy::LastPushWins,
            wal_checkpoint_threshold: 1000,
        }
    }
}

/// Statistics about the sync state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStats {
    pub last_sync: Option<DateTime<Utc>>,
    pub total_pushes: u64,
    pub total_pulls: u64,
    pub total_conflicts: u64,
    pub total_errors: u64,
    pub is_syncing: bool,
}

/// Error type for sync operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("network error: {0}")]
    Network(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("conflict detected: {0}")]
    Conflict(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("sync cancelled")]
    Cancelled,

    #[error("max retries exceeded: {0}")]
    MaxRetriesExceeded(String),
}

/// Core sync engine managing push/pull operations between local libsql and remote.
pub struct SyncEngine {
    local_db: EmbeddedTurso,
    remote_url: String,
    auth_token: String,
    config: SyncConfig,
    stats: Arc<Mutex<SyncStats>>,
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
    http_client: reqwest::Client,
}

impl SyncEngine {
    /// Create a new sync engine.
    pub fn new(
        local_db: EmbeddedTurso,
        remote_url: String,
        auth_token: String,
        config: SyncConfig,
    ) -> Self {
        Self {
            local_db,
            remote_url,
            auth_token,
            config,
            stats: Arc::new(Mutex::new(SyncStats::default())),
            cancel_tx: None,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Get a clone of the stats handle for external access.
    pub fn stats_handle(&self) -> Arc<Mutex<SyncStats>> {
        self.stats.clone()
    }

    /// Push local WAL frames to the remote server.
    ///
    /// Reads WAL frames since the last sync position and sends them in batches
    /// to the remote /sync endpoint. Handles conflicts using the configured strategy.
    pub async fn push(&self) -> Result<u64, SyncError> {
        let mut frames_pushed: u64 = 0;
        let batch_size = self.config.push_batch_size;

        tracing::debug!(batch_size, "starting push");

        loop {
            // Read WAL frames from local database since last sync position
            let frames = self.read_wal_frames(frames_pushed, batch_size).await?;

            if frames.is_empty() {
                break;
            }

            // Send frames to remote server with retry logic
            let pushed = self.send_frames_with_retry(&frames, frames_pushed).await?;

            frames_pushed += pushed as u64;

            if (pushed as u32) < batch_size {
                break;
            }
        }

        if frames_pushed > 0 {
            let mut stats = self.stats.lock().await;
            stats.total_pushes += frames_pushed;
            stats.last_sync = Some(Utc::now());
        }

        tracing::info!(frames_pushed, "push completed");
        Ok(frames_pushed)
    }

    /// Pull remote changes and apply them locally.
    ///
    /// Requests frames from the remote server since the last known position
    /// and applies them to the local database.
    pub async fn pull(&self) -> Result<u64, SyncError> {
        let mut frames_pulled: u64 = 0;
        let batch_size = self.config.pull_batch_size;

        tracing::debug!(batch_size, "starting pull");

        loop {
            // Request frames from remote since last known position
            let frames = self
                .fetch_remote_frames_with_retry(frames_pulled, batch_size)
                .await?;

            if frames.is_empty() {
                break;
            }

            // Apply frames to local database
            let applied = self.apply_remote_frames(&frames).await?;
            frames_pulled += applied as u64;

            if (applied as u32) < batch_size {
                break;
            }
        }

        if frames_pulled > 0 {
            let mut stats = self.stats.lock().await;
            stats.total_pulls += frames_pulled;
            stats.last_sync = Some(Utc::now());
        }

        tracing::info!(frames_pulled, "pull completed");
        Ok(frames_pulled)
    }

    /// Execute a single sync cycle: push local changes, then pull remote changes.
    pub async fn sync_once(&self) -> Result<(), SyncError> {
        let mut stats = self.stats.lock().await;
        if stats.is_syncing {
            tracing::debug!("sync already in progress, skipping");
            return Ok(());
        }
        stats.is_syncing = true;
        drop(stats);

        let result = async {
            // Push local changes first
            let pushed = self.push().await?;

            // Pull remote changes
            let pulled = self.pull().await?;

            // Check if WAL needs checkpointing
            let wal_size = self.get_wal_size().await?;
            if wal_size > self.config.wal_checkpoint_threshold {
                self.checkpoint().await?;
            }

            tracing::info!(pushed, pulled, "sync cycle completed");
            Ok(())
        }
        .await;

        let mut stats = self.stats.lock().await;
        stats.is_syncing = false;

        if let Err(ref e) = result {
            stats.total_errors += 1;
            tracing::error!(%e, "sync cycle failed");
        }

        result
    }

    /// Start background sync loop running at the configured interval.
    ///
    /// Spawns a tokio task that runs sync_once() periodically.
    /// Returns immediately; use stop_background_sync() to halt.
    pub fn start_background_sync(&mut self, app_handle: tauri::AppHandle) {
        // Cancel any existing background sync
        self.stop_background_sync();

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
        self.cancel_tx = Some(cancel_tx);

        let engine = self.clone_for_background();
        let interval = Duration::from_secs(self.config.sync_interval_secs);

        tauri::async_runtime::spawn(async move {
            let mut ticker = time::interval(interval);
            let mut cancel_rx = cancel_rx;

            tracing::info!(
                interval_secs = interval.as_secs(),
                "background sync started"
            );

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if let Err(e) = engine.sync_once().await {
                            tracing::warn!(%e, "background sync cycle failed");
                            let _ = app_handle.emit("sync:error", &format!("{e}"));
                        } else {
                            let stats = engine.stats.lock().await.clone();
                            let _ = app_handle.emit("sync:stats", &stats);
                        }
                    }
                    _ = &mut cancel_rx => {
                        tracing::info!("background sync cancelled");
                        break;
                    }
                }
            }
        });
    }

    /// Stop the background sync loop.
    pub fn stop_background_sync(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
            tracing::info!("background sync stopped");
        }
    }

    /// Checkpoint the WAL to optimize database size.
    ///
    /// Triggers a WAL checkpoint which moves committed frames from the WAL
    /// into the main database file, reducing WAL size.
    pub async fn checkpoint(&self) -> Result<(), SyncError> {
        tracing::info!("running WAL checkpoint");

        // Execute PRAGMA wal_checkpoint(PASSIVE) to checkpoint WAL
        // PASSIVE mode doesn't block other connections
        let result = self
            .local_db
            .execute("PRAGMA wal_checkpoint(PASSIVE)", vec![])
            .await;

        match result {
            Ok(_) => {
                tracing::info!("WAL checkpoint completed");
                Ok(())
            }
            Err(e) => {
                let msg = format!("WAL checkpoint failed: {e}");
                tracing::error!(%msg);
                Err(SyncError::Database(msg))
            }
        }
    }

    /// Get a reference to the current sync configuration.
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Update the sync configuration.
    pub fn configure(&mut self, config: SyncConfig) {
        // If interval changed and sync is running, restart it
        let interval_changed = self.config.sync_interval_secs != config.sync_interval_secs;
        self.config = config;

        if interval_changed && self.cancel_tx.is_some() {
            tracing::info!("sync interval changed, restarting background sync");
            // Note: caller should re-start with app_handle if needed
            self.stop_background_sync();
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────

    /// Clone self for background task (shares stats Arc and db clone).
    fn clone_for_background(&self) -> Self {
        Self {
            local_db: self.local_db.clone(),
            remote_url: self.remote_url.clone(),
            auth_token: self.auth_token.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
            cancel_tx: None,
            http_client: self.http_client.clone(),
        }
    }

    /// Read WAL frames from local database starting at the given offset.
    async fn read_wal_frames(
        &self,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, SyncError> {
        // Read frames from a sync tracking table
        // This assumes a local sync_metadata table exists
        let sql = format!(
            "SELECT frame_data FROM sync_frames WHERE frame_id > {} ORDER BY frame_id LIMIT {}",
            offset, limit
        );

        // Use a raw query that returns JSON values
        let frames: Vec<serde_json::Value> = self
            .local_db
            .query(&sql, vec![])
            .await
            .map_err(|e| SyncError::Database(e.to_string()))?;

        Ok(frames)
    }

    /// Get current WAL size (number of pending frames).
    async fn get_wal_size(&self) -> Result<u64, SyncError> {
        // Use a heuristic based on the sync_frames count
        let count_sql = "SELECT COUNT(*) as cnt FROM sync_frames WHERE synced = 0";
        let count_result: Vec<serde_json::Value> = self
            .local_db
            .query(count_sql, vec![])
            .await
            .map_err(|e| SyncError::Database(e.to_string()))?;

        let count = count_result
            .first()
            .and_then(|v| v.get("cnt"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(count)
    }

    /// Send frames to remote server with exponential backoff retry.
    async fn send_frames_with_retry(
        &self,
        frames: &[serde_json::Value],
        offset: u64,
    ) -> Result<usize, SyncError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = exponential_backoff(attempt);
                tracing::debug!(attempt, delay_secs = delay.as_secs(), "retrying push");
                time::sleep(delay).await;
            }

            match self.send_frames(frames, offset).await {
                Ok(n) => return Ok(n),
                Err(e) => {
                    tracing::warn!(attempt, %e, "push attempt failed");
                    last_error = Some(e);
                }
            }
        }

        Err(SyncError::MaxRetriesExceeded(format!(
            "push failed after {} retries: {:?}",
            self.config.max_retries, last_error
        )))
    }

    /// Send frames to the remote /sync endpoint.
    async fn send_frames(
        &self,
        frames: &[serde_json::Value],
        offset: u64,
    ) -> Result<usize, SyncError> {
        let url = format!("{}/sync", self.remote_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "offset": offset,
            "frames": frames,
            "conflict_strategy": self.config.conflict_strategy.to_string(),
        });

        let resp = self
            .http_client
            .post(&url)
            .bearer_auth(&self.auth_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<empty body>".to_string());

            // Check for conflict response
            if status == 409 {
                let conflicts: Vec<ConflictRecord> =
                    serde_json::from_str(&body).unwrap_or_default();
                let mut stats = self.stats.lock().await;
                stats.total_conflicts += conflicts.len() as u64;

                for conflict in &conflicts {
                    tracing::warn!(
                        table = %conflict.table,
                        row_id = %conflict.row_id,
                        resolution = %conflict.resolution,
                        "conflict resolved"
                    );
                }

                return Ok(frames.len());
            }

            return Err(SyncError::Network(format!(
                "remote returned {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        let applied = body.get("applied").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        Ok(applied)
    }

    /// Fetch remote frames with exponential backoff retry.
    async fn fetch_remote_frames_with_retry(
        &self,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, SyncError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = exponential_backoff(attempt);
                tracing::debug!(attempt, delay_secs = delay.as_secs(), "retrying pull");
                time::sleep(delay).await;
            }

            match self.fetch_remote_frames(offset, limit).await {
                Ok(frames) => return Ok(frames),
                Err(e) => {
                    tracing::warn!(attempt, %e, "pull attempt failed");
                    last_error = Some(e);
                }
            }
        }

        Err(SyncError::MaxRetriesExceeded(format!(
            "pull failed after {} retries: {:?}",
            self.config.max_retries, last_error
        )))
    }

    /// Fetch frames from the remote server.
    async fn fetch_remote_frames(
        &self,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, SyncError> {
        let url = format!(
            "{}/sync?offset={}&limit={}",
            self.remote_url.trim_end_matches('/'),
            offset,
            limit
        );

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SyncError::Network(format!(
                "remote returned {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        let frames = body
            .get("frames")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(frames)
    }

    /// Apply remote frames to the local database.
    async fn apply_remote_frames(&self, frames: &[serde_json::Value]) -> Result<usize, SyncError> {
        if frames.is_empty() {
            return Ok(0);
        }

        let mut applied = 0;

        for frame in frames {
            // Extract SQL from frame and execute locally
            if let Some(sql) = frame.get("sql").and_then(|v| v.as_str()) {
                let params: Vec<String> = frame
                    .get("params")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                if let Err(e) = self.local_db.execute(sql, params).await {
                    // Handle conflict: try to resolve
                    let resolution = resolve_conflict(self.config.conflict_strategy, None, None);

                    if resolution == "remote_won" {
                        // Retry with conflict resolution (e.g., DELETE then INSERT)
                        tracing::warn!(%e, "conflict during pull, applying remote_won strategy");
                        // For LastPushWins, we skip conflicting rows
                        continue;
                    }

                    let mut stats = self.stats.lock().await;
                    stats.total_conflicts += 1;
                    tracing::warn!(%e, "failed to apply frame");
                } else {
                    applied += 1;
                }
            }
        }

        Ok(applied)
    }
}

/// Calculate exponential backoff delay with jitter.
///
/// Uses base delay of 1 second, doubling each attempt, capped at 60 seconds.
fn exponential_backoff(attempt: u32) -> Duration {
    let base_secs = 1u64;
    let max_secs = 60u64;
    let delay = base_secs.saturating_mul(2u64.saturating_pow(attempt));
    let delay = delay.min(max_secs);

    // Add jitter: ±25% random variation
    let jitter = (delay as f64 * 0.25) as u64;
    let jittered = delay.saturating_sub(jitter)
        + (rand::random::<u64>() % jitter.saturating_mul(2).saturating_add(1));

    Duration::from_secs(jittered.min(max_secs))
}

/// Initialize sync metadata tables in the local database.
///
/// Creates the sync_frames table for tracking unsynced changes.
/// Safe to call multiple times (uses IF NOT EXISTS).
pub async fn init_sync_tables(db: &EmbeddedTurso) -> Result<(), SyncError> {
    db.execute(
        "
        CREATE TABLE IF NOT EXISTS sync_frames (
            frame_id INTEGER PRIMARY KEY AUTOINCREMENT,
            frame_data TEXT NOT NULL,
            synced INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sync_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sync_frames_synced ON sync_frames(synced);
        ",
        vec![],
    )
    .await
    .map_err(|e| SyncError::Database(e.to_string()))?;

    tracing::info!("sync metadata tables initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.sync_interval_secs, 30);
        assert_eq!(config.push_batch_size, 128);
        assert_eq!(config.pull_batch_size, 128);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.conflict_strategy, ConflictStrategy::LastPushWins);
        assert_eq!(config.wal_checkpoint_threshold, 1000);
    }

    #[test]
    fn test_sync_stats_default() {
        let stats = SyncStats::default();
        assert!(stats.last_sync.is_none());
        assert_eq!(stats.total_pushes, 0);
        assert_eq!(stats.total_pulls, 0);
        assert_eq!(stats.total_conflicts, 0);
        assert_eq!(stats.total_errors, 0);
        assert!(!stats.is_syncing);
    }

    #[test]
    fn test_exponential_backoff_increases() {
        let delay_0 = exponential_backoff(0);
        let delay_1 = exponential_backoff(1);
        let delay_2 = exponential_backoff(2);

        // With jitter, we can't assert exact values, but the trend should increase
        assert!(delay_0.as_secs() <= delay_1.as_secs() || delay_0.as_secs() <= 3);
        assert!(delay_1.as_secs() <= delay_2.as_secs() || delay_1.as_secs() <= 5);
    }

    #[test]
    fn test_exponential_backoff_capped() {
        let delay = exponential_backoff(10);
        assert!(delay.as_secs() <= 60);
    }

    #[test]
    fn test_sync_error_display() {
        let err = SyncError::Network("timeout".to_string());
        assert_eq!(format!("{err}"), "network error: timeout");

        let err = SyncError::Database("corrupt".to_string());
        assert_eq!(format!("{err}"), "database error: corrupt");

        let err = SyncError::Conflict("row conflict".to_string());
        assert_eq!(format!("{err}"), "conflict detected: row conflict");
    }

    #[test]
    fn test_sync_config_serde() {
        let config = SyncConfig {
            sync_interval_secs: 60,
            conflict_strategy: ConflictStrategy::LastWriteWins,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SyncConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sync_interval_secs, 60);
        assert_eq!(
            deserialized.conflict_strategy,
            ConflictStrategy::LastWriteWins
        );
    }
}
