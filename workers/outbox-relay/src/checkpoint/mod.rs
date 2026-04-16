//! Checkpoint store — tracks the last processed outbox ID.
//!
//! This allows the worker to resume from where it left off
//! after a restart or crash. Checkpoint state is persisted
//! to a JSON file for crash recovery.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Persistent checkpoint state.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointState {
    last_processed: u64,
}

/// Stores and retrieves the last processed outbox sequence number.
/// Persists checkpoint to disk for crash recovery.
pub struct CheckpointStore {
    last_processed: AtomicU64,
    checkpoint_path: String,
}

impl CheckpointStore {
    /// Create a new checkpoint store, loading from disk if exists.
    pub fn new(initial: u64, checkpoint_path: &str) -> Self {
        let loaded = Self::load_from_disk(checkpoint_path).unwrap_or(initial);
        Self {
            last_processed: AtomicU64::new(loaded),
            checkpoint_path: checkpoint_path.to_string(),
        }
    }

    /// Load checkpoint from disk if the file exists.
    fn load_from_disk(path: &str) -> Option<u64> {
        let path_ref = Path::new(path);
        if path_ref.exists() {
            match fs::read_to_string(path_ref) {
                Ok(content) => match serde_json::from_str::<CheckpointState>(&content) {
                    Ok(state) => Some(state.last_processed),
                    Err(e) => {
                        tracing::warn!(error = %e, "failed to parse checkpoint file, using initial value");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "failed to read checkpoint file, using initial value");
                    None
                }
            }
        } else {
            None
        }
    }

    /// Persist checkpoint to disk.
    fn persist(&self) {
        let state = CheckpointState {
            last_processed: self.get(),
        };
        if let Err(e) =
            serde_json::to_string_pretty(&state).map(|json| fs::write(&self.checkpoint_path, json))
        {
            tracing::warn!(error = %e, "failed to persist checkpoint to disk");
        }
    }

    /// Get the last processed checkpoint.
    pub fn get(&self) -> u64 {
        self.last_processed.load(Ordering::Acquire)
    }

    /// Advance the checkpoint to a new value and persist to disk.
    pub fn advance(&self, new_value: u64) {
        self.last_processed.store(new_value, Ordering::Release);
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_checkpoint_path() -> String {
        let uuid = uuid::Uuid::new_v4();
        format!("/tmp/outbox-checkpoint-test-{}.json", uuid)
    }

    #[test]
    fn checkpoint_starts_at_initial() {
        let path = temp_checkpoint_path();
        let store = CheckpointStore::new(42, &path);
        assert_eq!(store.get(), 42);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn checkpoint_advances_monotonically() {
        let path = temp_checkpoint_path();
        let store = CheckpointStore::new(0, &path);
        store.advance(10);
        assert_eq!(store.get(), 10);
        store.advance(25);
        assert_eq!(store.get(), 25);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn checkpoint_persists_to_disk() {
        let path = temp_checkpoint_path();
        {
            let store = CheckpointStore::new(0, &path);
            store.advance(100);
            assert_eq!(store.get(), 100);
        }

        // Reload from disk
        let store2 = CheckpointStore::new(0, &path);
        assert_eq!(store2.get(), 100);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn checkpoint_handles_corrupt_file() {
        let path = temp_checkpoint_path();
        fs::write(&path, "not valid json").unwrap();

        let store = CheckpointStore::new(42, &path);
        assert_eq!(store.get(), 42);
        let _ = fs::remove_file(&path);
    }
}
