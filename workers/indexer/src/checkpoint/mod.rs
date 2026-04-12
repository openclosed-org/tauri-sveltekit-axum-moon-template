//! Source checkpoint — tracks progress per event source.
//!
//! Each source (e.g., "nostr-relay-1", "farcaster-feed") has its own
//! cursor so the indexer can resume independently per source.

use std::collections::HashMap;
use std::sync::RwLock;

/// Tracks the last processed cursor per source.
pub struct SourceCheckpoint {
    cursors: RwLock<HashMap<String, String>>,
}

impl SourceCheckpoint {
    pub fn new() -> Self {
        Self {
            cursors: RwLock::new(HashMap::new()),
        }
    }

    /// Get the cursor for a source.
    pub fn get(&self, source: &str) -> Option<String> {
        self.cursors.read().unwrap().get(source).cloned()
    }

    /// Update the cursor for a source.
    pub fn update(&self, source: &str, cursor: String) {
        self.cursors.write().unwrap().insert(source.to_string(), cursor);
    }

    /// List all checkpoints.
    pub fn list(&self) -> std::collections::HashMap<String, String> {
        self.cursors.read().unwrap().clone()
    }
}

impl Default for SourceCheckpoint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_starts_empty() {
        let cp = SourceCheckpoint::new();
        assert!(cp.get("source-1").is_none());
    }

    #[test]
    fn checkpoint_updates_per_source() {
        let cp = SourceCheckpoint::new();
        cp.update("source-1", "cursor-a".to_string());
        cp.update("source-2", "cursor-b".to_string());

        assert_eq!(cp.get("source-1"), Some("cursor-a".to_string()));
        assert_eq!(cp.get("source-2"), Some("cursor-b".to_string()));
    }
}
