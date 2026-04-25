//! libSQL/Turso-backed implementations for shared worker state.

use async_trait::async_trait;
use data::ports::lib_sql::LibSqlPort;
use serde::Deserialize;

use crate::checkpoint::{CheckpointStore, CheckpointStoreError};
use crate::dedupe::{DedupeStore, DedupeStoreError};
use crate::idempotency::{IdempotencyStatus, IdempotencyStore, IdempotencyStoreError};

#[derive(Debug, Deserialize)]
struct CheckpointRow {
    last_processed: i64,
}

#[derive(Debug, Deserialize)]
struct IdempotencyRow {
    status: String,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExistsRow {
    count: i64,
}

/// Shared checkpoint store backed by libSQL/Turso.
pub struct LibSqlCheckpointStore<P: LibSqlPort> {
    port: P,
    worker_name: String,
    initial: u64,
}

impl<P: LibSqlPort> LibSqlCheckpointStore<P> {
    pub fn new(port: P, worker_name: &str, initial: u64) -> Self {
        Self {
            port,
            worker_name: worker_name.to_string(),
            initial,
        }
    }
}

#[async_trait]
impl<P: LibSqlPort> CheckpointStore for LibSqlCheckpointStore<P> {
    async fn get(&self) -> Result<u64, CheckpointStoreError> {
        let rows: Vec<CheckpointRow> = self
            .port
            .query(
                "SELECT last_processed FROM worker_runtime_checkpoints WHERE worker_name = ?",
                vec![self.worker_name.clone()],
            )
            .await
            .map_err(|error| CheckpointStoreError::LoadFailed(error.to_string()))?;

        Ok(rows
            .first()
            .map(|row| row.last_processed as u64)
            .unwrap_or(self.initial))
    }

    async fn advance(&self, new_value: u64) -> Result<(), CheckpointStoreError> {
        self.port
            .execute(
                "INSERT INTO worker_runtime_checkpoints (worker_name, last_processed, updated_at) \
                 VALUES (?, ?, datetime('now')) \
                 ON CONFLICT(worker_name) DO UPDATE SET \
                     last_processed = excluded.last_processed, \
                     updated_at = datetime('now')",
                vec![self.worker_name.clone(), new_value.to_string()],
            )
            .await
            .map_err(|error| CheckpointStoreError::PersistFailed(error.to_string()))?;

        Ok(())
    }
}

/// Shared idempotency store backed by libSQL/Turso.
pub struct LibSqlIdempotencyStore<P: LibSqlPort> {
    port: P,
    worker_name: String,
}

impl<P: LibSqlPort> LibSqlIdempotencyStore<P> {
    pub fn new(port: P, worker_name: &str) -> Self {
        Self {
            port,
            worker_name: worker_name.to_string(),
        }
    }
}

#[async_trait]
impl<P: LibSqlPort> IdempotencyStore for LibSqlIdempotencyStore<P> {
    async fn check(&self, key: &str) -> Result<IdempotencyStatus, IdempotencyStoreError> {
        let rows: Vec<IdempotencyRow> = self
            .port
            .query(
                "SELECT status, error_message \
                 FROM worker_runtime_idempotency \
                 WHERE worker_name = ? AND idempotency_key = ?",
                vec![self.worker_name.clone(), key.to_string()],
            )
            .await
            .map_err(|error| IdempotencyStoreError::CheckFailed(error.to_string()))?;

        Ok(match rows.first() {
            Some(row) if row.status == "in_progress" => IdempotencyStatus::InProgress,
            Some(row) if row.status == "completed" => IdempotencyStatus::Completed,
            Some(row) if row.status == "failed" => {
                IdempotencyStatus::Failed(row.error_message.clone().unwrap_or_default())
            }
            Some(row) => IdempotencyStatus::Failed(format!("unknown status: {}", row.status)),
            None => IdempotencyStatus::Unknown,
        })
    }

    async fn start(&self, key: &str) -> Result<bool, IdempotencyStoreError> {
        match self.check(key).await? {
            IdempotencyStatus::InProgress | IdempotencyStatus::Completed => Ok(false),
            IdempotencyStatus::Unknown | IdempotencyStatus::Failed(_) => {
                self.port
                    .execute(
                        "INSERT INTO worker_runtime_idempotency \
                         (worker_name, idempotency_key, status, error_message, updated_at) \
                         VALUES (?, ?, 'in_progress', NULL, datetime('now')) \
                         ON CONFLICT(worker_name, idempotency_key) DO UPDATE SET \
                             status = 'in_progress', \
                             error_message = NULL, \
                             updated_at = datetime('now')",
                        vec![self.worker_name.clone(), key.to_string()],
                    )
                    .await
                    .map_err(|error| IdempotencyStoreError::UpdateFailed(error.to_string()))?;

                Ok(true)
            }
        }
    }

    async fn complete(&self, key: &str) -> Result<(), IdempotencyStoreError> {
        self.port
            .execute(
                "INSERT INTO worker_runtime_idempotency \
                 (worker_name, idempotency_key, status, error_message, updated_at) \
                 VALUES (?, ?, 'completed', NULL, datetime('now')) \
                 ON CONFLICT(worker_name, idempotency_key) DO UPDATE SET \
                     status = 'completed', \
                     error_message = NULL, \
                     updated_at = datetime('now')",
                vec![self.worker_name.clone(), key.to_string()],
            )
            .await
            .map_err(|error| IdempotencyStoreError::UpdateFailed(error.to_string()))?;

        Ok(())
    }

    async fn fail(&self, key: &str, error: String) -> Result<(), IdempotencyStoreError> {
        self.port
            .execute(
                "INSERT INTO worker_runtime_idempotency \
                 (worker_name, idempotency_key, status, error_message, updated_at) \
                 VALUES (?, ?, 'failed', ?, datetime('now')) \
                 ON CONFLICT(worker_name, idempotency_key) DO UPDATE SET \
                     status = 'failed', \
                     error_message = excluded.error_message, \
                     updated_at = datetime('now')",
                vec![self.worker_name.clone(), key.to_string(), error],
            )
            .await
            .map_err(|error| IdempotencyStoreError::UpdateFailed(error.to_string()))?;

        Ok(())
    }

    async fn is_already_processed(&self, key: &str) -> Result<bool, IdempotencyStoreError> {
        Ok(matches!(
            self.check(key).await?,
            IdempotencyStatus::Completed
        ))
    }
}

/// Shared dedupe store backed by libSQL/Turso.
pub struct LibSqlDedupeStore<P: LibSqlPort> {
    port: P,
    worker_name: String,
}

impl<P: LibSqlPort> LibSqlDedupeStore<P> {
    pub fn new(port: P, worker_name: &str) -> Self {
        Self {
            port,
            worker_name: worker_name.to_string(),
        }
    }
}

#[async_trait]
impl<P: LibSqlPort> DedupeStore for LibSqlDedupeStore<P> {
    async fn is_duplicate(&self, id: &str) -> Result<bool, DedupeStoreError> {
        let rows: Vec<ExistsRow> = self
            .port
            .query(
                "SELECT COUNT(*) as count \
                 FROM worker_runtime_dedupe \
                 WHERE worker_name = ? AND message_id = ?",
                vec![self.worker_name.clone(), id.to_string()],
            )
            .await
            .map_err(|error| DedupeStoreError::CheckFailed(error.to_string()))?;

        Ok(rows.first().map(|row| row.count > 0).unwrap_or(false))
    }

    async fn mark_processed(&self, id: &str) -> Result<bool, DedupeStoreError> {
        let affected = self
            .port
            .execute(
                "INSERT OR IGNORE INTO worker_runtime_dedupe \
                 (worker_name, message_id, processed_at) \
                 VALUES (?, ?, datetime('now'))",
                vec![self.worker_name.clone(), id.to_string()],
            )
            .await
            .map_err(|error| DedupeStoreError::MarkFailed(error.to_string()))?;

        Ok(affected > 0)
    }
}
