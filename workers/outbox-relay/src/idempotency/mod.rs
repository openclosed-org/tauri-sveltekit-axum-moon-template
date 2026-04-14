//! Idempotency keys — ensures outbox entries are published exactly once.
//!
/// Tracks idempotency keys for each outbox entry to prevent
/// duplicate event publishing when the relay restarts or retries.
use std::collections::HashMap;
use std::sync::RwLock;

/// Represents the status of an idempotency key.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyStatus {
    /// The key has not been seen before.
    Unknown,
    /// The event is currently being processed.
    InProgress,
    /// The event has been successfully published.
    Completed,
    /// The event publishing failed.
    Failed(String),
}

/// Manages idempotency keys for outbox entries.
pub struct IdempotencyStore {
    /// Maps idempotency key to its current status.
    store: RwLock<HashMap<String, IdempotencyStatus>>,
    max_size: usize,
}

impl IdempotencyStore {
    /// Create a new idempotency store with the given capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            store: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
        }
    }

    /// Check the status of an idempotency key.
    pub fn check(&self, key: &str) -> IdempotencyStatus {
        let store = self.store.read().unwrap();
        store
            .get(key)
            .cloned()
            .unwrap_or(IdempotencyStatus::Unknown)
    }

    /// Mark an idempotency key as in progress.
    /// Returns false if the key is already in progress or completed.
    pub fn start(&self, key: &str) -> bool {
        let mut store = self.store.write().unwrap();

        // Evict old entries if at capacity (remove completed/failed entries first)
        if store.len() >= self.max_size {
            let to_remove: Vec<String> = store
                .iter()
                .filter(|(_, v)| {
                    matches!(
                        v,
                        IdempotencyStatus::Completed | IdempotencyStatus::Failed(_)
                    )
                })
                .take(store.len() / 4)
                .map(|(k, _)| k.clone())
                .collect();
            for k in to_remove {
                store.remove(&k);
            }
        }

        match store.get(key) {
            Some(IdempotencyStatus::InProgress) | Some(IdempotencyStatus::Completed) => false,
            _ => {
                store.insert(key.to_string(), IdempotencyStatus::InProgress);
                true
            }
        }
    }

    /// Mark an idempotency key as successfully completed.
    pub fn complete(&self, key: &str) {
        let mut store = self.store.write().unwrap();
        store.insert(key.to_string(), IdempotencyStatus::Completed);
    }

    /// Mark an idempotency key as failed with an error message.
    pub fn fail(&self, key: &str, error: String) {
        let mut store = self.store.write().unwrap();
        store.insert(key.to_string(), IdempotencyStatus::Failed(error));
    }

    /// Check if a key has already been processed (completed).
    pub fn is_already_processed(&self, key: &str) -> bool {
        let store = self.store.read().unwrap();
        matches!(store.get(key), Some(IdempotencyStatus::Completed))
    }

    /// Get the number of tracked keys.
    pub fn len(&self) -> usize {
        self.store.read().unwrap().len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.read().unwrap().is_empty()
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new(50_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_key_returns_unknown_status() {
        let store = IdempotencyStore::new(100);
        assert_eq!(store.check("key-1"), IdempotencyStatus::Unknown);
    }

    #[test]
    fn start_marks_key_as_in_progress() {
        let store = IdempotencyStore::new(100);
        assert!(store.start("key-1"));
        assert_eq!(store.check("key-1"), IdempotencyStatus::InProgress);
    }

    #[test]
    fn cannot_start_already_in_progress() {
        let store = IdempotencyStore::new(100);
        assert!(store.start("key-1"));
        assert!(!store.start("key-1"));
    }

    #[test]
    fn complete_marks_key_as_completed() {
        let store = IdempotencyStore::new(100);
        store.start("key-1");
        store.complete("key-1");
        assert_eq!(store.check("key-1"), IdempotencyStatus::Completed);
    }

    #[test]
    fn is_already_processed_returns_true_for_completed() {
        let store = IdempotencyStore::new(100);
        store.start("key-1");
        store.complete("key-1");
        assert!(store.is_already_processed("key-1"));
    }

    #[test]
    fn fail_records_error() {
        let store = IdempotencyStore::new(100);
        store.start("key-1");
        store.fail("key-1", "connection timeout".to_string());
        assert_eq!(
            store.check("key-1"),
            IdempotencyStatus::Failed("connection timeout".to_string())
        );
    }

    #[test]
    fn completed_keys_cannot_be_restarted() {
        let store = IdempotencyStore::new(100);
        store.start("key-1");
        store.complete("key-1");
        assert!(!store.start("key-1"));
        assert!(store.is_already_processed("key-1"));
    }
}
