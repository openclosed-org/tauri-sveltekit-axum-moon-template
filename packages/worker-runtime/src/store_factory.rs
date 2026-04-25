//! Shared store bootstrap and backend selection helpers.

use data::ports::lib_sql::LibSqlPort;
use storage_turso::TursoBackend;

use crate::adapters::{
    FileCheckpointStore, FileDedupeStore, FileIdempotencyStore, LibSqlCheckpointStore,
    LibSqlDedupeStore, LibSqlIdempotencyStore,
};
use crate::{CheckpointStore, DedupeStore, IdempotencyStore};

const SHARED_STORE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS worker_runtime_checkpoints (
    worker_name TEXT PRIMARY KEY,
    last_processed INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS worker_runtime_idempotency (
    worker_name TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('in_progress', 'completed', 'failed')),
    error_message TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(worker_name, idempotency_key)
);

CREATE TABLE IF NOT EXISTS worker_runtime_dedupe (
    worker_name TEXT NOT NULL,
    message_id TEXT NOT NULL,
    processed_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(worker_name, message_id)
);

CREATE INDEX IF NOT EXISTS idx_worker_runtime_idempotency_status
    ON worker_runtime_idempotency(worker_name, status, updated_at);

CREATE INDEX IF NOT EXISTS idx_worker_runtime_dedupe_processed_at
    ON worker_runtime_dedupe(worker_name, processed_at);
"#;

/// Concrete backend selected for worker runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStoreBackend {
    /// Shared state stored in libSQL/Turso for multi-instance workers.
    SharedLibSql,
    /// Local single-instance fallback using file/in-memory state.
    LocalFallback,
}

impl WorkerStoreBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SharedLibSql => "shared-libsql",
            Self::LocalFallback => "local-fallback",
        }
    }
}

/// Bundle of worker stores selected from configuration.
pub struct WorkerStoreSet {
    pub checkpoint: Box<dyn CheckpointStore>,
    pub idempotency: Box<dyn IdempotencyStore>,
    pub dedupe: Box<dyn DedupeStore>,
    pub backend: WorkerStoreBackend,
}

/// Ensure the shared libSQL/Turso schema exists before stores use it.
pub async fn ensure_shared_store_schema<P: LibSqlPort>(port: &P) -> anyhow::Result<()> {
    port.execute_batch(SHARED_STORE_SCHEMA)
        .await
        .map_err(|error| {
            anyhow::anyhow!("failed to initialize worker-runtime shared store schema: {error}")
        })
}

/// Build only the checkpoint store for a worker.
pub async fn build_checkpoint_store(
    worker_name: &str,
    database_url: &str,
    auth_token: Option<&str>,
    checkpoint_path: &str,
    initial: u64,
) -> anyhow::Result<(Box<dyn CheckpointStore>, WorkerStoreBackend)> {
    if is_shared_database_url(database_url) {
        let backend = TursoBackend::connect(database_url, auth_token)
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "failed to connect shared worker store '{}': {}",
                    database_url,
                    error
                )
            })?;
        ensure_shared_store_schema(&backend).await?;

        return Ok((
            Box::new(LibSqlCheckpointStore::new(backend, worker_name, initial)),
            WorkerStoreBackend::SharedLibSql,
        ));
    }

    Ok((
        Box::new(FileCheckpointStore::new(checkpoint_path, initial)),
        WorkerStoreBackend::LocalFallback,
    ))
}

/// Build checkpoint, idempotency, and dedupe stores for a worker.
pub async fn build_worker_store_set(
    worker_name: &str,
    database_url: &str,
    auth_token: Option<&str>,
    checkpoint_path: &str,
) -> anyhow::Result<WorkerStoreSet> {
    if is_shared_database_url(database_url) {
        let backend = TursoBackend::connect(database_url, auth_token)
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "failed to connect shared worker store '{}': {}",
                    database_url,
                    error
                )
            })?;
        ensure_shared_store_schema(&backend).await?;

        return Ok(WorkerStoreSet {
            checkpoint: Box::new(LibSqlCheckpointStore::new(backend.clone(), worker_name, 0)),
            idempotency: Box::new(LibSqlIdempotencyStore::new(backend.clone(), worker_name)),
            dedupe: Box::new(LibSqlDedupeStore::new(backend, worker_name)),
            backend: WorkerStoreBackend::SharedLibSql,
        });
    }

    Ok(WorkerStoreSet {
        checkpoint: Box::new(FileCheckpointStore::new(checkpoint_path, 0)),
        idempotency: Box::new(FileIdempotencyStore::default()),
        dedupe: Box::new(FileDedupeStore::default()),
        backend: WorkerStoreBackend::LocalFallback,
    })
}

fn is_shared_database_url(database_url: &str) -> bool {
    database_url.starts_with("libsql://") && !database_url.starts_with("libsql://file:")
}

#[cfg(test)]
mod tests {
    use super::{WorkerStoreBackend, is_shared_database_url};

    #[test]
    fn remote_libsql_uses_shared_backend() {
        assert!(is_shared_database_url("libsql://counter-db.turso.io"));
        assert!(!is_shared_database_url("libsql://file:/tmp/local.db"));
        assert!(!is_shared_database_url("file:/tmp/local.db"));
        assert_eq!(WorkerStoreBackend::LocalFallback.as_str(), "local-fallback");
    }
}
