//! Projection checkpoint — tracks the last processed event sequence.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointState {
    last_processed: u64,
}

/// Tracks the last processed event sequence for resumption.
pub struct ProjectionCheckpoint {
    last_processed: AtomicU64,
    checkpoint_path: String,
}

impl ProjectionCheckpoint {
    pub fn new(initial: u64, checkpoint_path: &str) -> Self {
        let loaded = Self::load_from_disk(checkpoint_path).unwrap_or(initial);
        Self {
            last_processed: AtomicU64::new(loaded),
            checkpoint_path: checkpoint_path.to_string(),
        }
    }

    fn load_from_disk(path: &str) -> Option<u64> {
        let path_ref = Path::new(path);
        if !path_ref.exists() {
            return None;
        }

        match fs::read_to_string(path_ref) {
            Ok(content) => match serde_json::from_str::<CheckpointState>(&content) {
                Ok(state) => Some(state.last_processed),
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse projector checkpoint file");
                    None
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "failed to read projector checkpoint file");
                None
            }
        }
    }

    fn persist(&self) {
        let state = CheckpointState {
            last_processed: self.get(),
        };
        if let Err(e) =
            serde_json::to_string_pretty(&state).map(|json| fs::write(&self.checkpoint_path, json))
        {
            tracing::warn!(error = %e, "failed to persist projector checkpoint");
        }
    }

    pub fn get(&self) -> u64 {
        self.last_processed.load(Ordering::Acquire)
    }

    pub fn advance(&self, sequence: u64) {
        if sequence > self.get() {
            self.last_processed.store(sequence, Ordering::Release);
            self.persist();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_starts_at_initial() {
        let path = "/tmp/projector-checkpoint-starts-at-initial.json";
        let _ = std::fs::remove_file(path);
        let cp = ProjectionCheckpoint::new(100, path);
        assert_eq!(cp.get(), 100);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn only_advances_forward() {
        let path = "/tmp/projector-checkpoint-only-advances-forward.json";
        let _ = std::fs::remove_file(path);
        let cp = ProjectionCheckpoint::new(100, path);
        cp.advance(200);
        assert_eq!(cp.get(), 200);
        cp.advance(50); // Should not go backward
        assert_eq!(cp.get(), 200);
        let _ = std::fs::remove_file(path);
    }
}
