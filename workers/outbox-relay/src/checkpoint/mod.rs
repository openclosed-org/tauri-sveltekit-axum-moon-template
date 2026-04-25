//! Checkpoint store — re-exports from worker-runtime.
//!
//! This module is retained for backward compatibility but delegates
//! to the shared worker-runtime implementation.

pub use worker_runtime::CheckpointStore as CheckpointStorePort;
