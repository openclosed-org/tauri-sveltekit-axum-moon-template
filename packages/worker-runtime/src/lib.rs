//! Worker runtime — shared reliability primitives for background workers.
//!
//! Provides checkpoint, idempotency, and deduplication abstractions that
//! workers can use instead of copying local-file or in-memory implementations.
//!
//! ## Design
//!
//! - Trait-based: workers depend on `CheckpointStore` / `IdempotencyStore` / `DedupeStore`
//! - Multiple backends: file (local dev), libSQL/Turso (shared production)
//! - Minimal: only what outbox-relay and projector actually need
//!
//! ## Usage
//!
//! ```ignore
//! use worker_runtime::{CheckpointStore, FileCheckpointStore};
//!
//! let checkpoint = FileCheckpointStore::new("/tmp/checkpoint.json", 0);
//! checkpoint.advance(42).await?;
//! assert_eq!(checkpoint.get().await?, 42);
//! ```

pub mod adapters;
pub mod checkpoint;
pub mod dedupe;
pub mod idempotency;
pub mod store_factory;
pub mod worker;

pub use adapters::{
    FileCheckpointStore, FileDedupeStore, FileIdempotencyStore, LibSqlCheckpointStore,
    LibSqlDedupeStore, LibSqlIdempotencyStore, MemoryDedupeStore, MemoryIdempotencyStore,
};
pub use checkpoint::{CheckpointStore, CheckpointStoreError};
pub use dedupe::{DedupeStore, DedupeStoreError};
pub use idempotency::{IdempotencyStatus, IdempotencyStore, IdempotencyStoreError};
pub use store_factory::{
    WorkerStoreBackend, WorkerStoreSet, build_checkpoint_store, build_worker_store_set,
    ensure_shared_store_schema,
};
pub use worker::{
    WorkerBootstrap, WorkerHealthState, bootstrap_worker, shutdown_signal, spawn_health_server,
};
