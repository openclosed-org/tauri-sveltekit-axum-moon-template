//! In-memory implementations for testing.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use crate::dedupe::{DedupeStore, DedupeStoreError};
use crate::idempotency::{IdempotencyStatus, IdempotencyStore, IdempotencyStoreError};

// ── Memory Idempotency Store ─────────────────────────────────

/// In-memory idempotency store for testing.
pub struct MemoryIdempotencyStore {
    store: RwLock<HashMap<String, IdempotencyStatus>>,
}

impl MemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for MemoryIdempotencyStore {
    async fn check(&self, key: &str) -> Result<IdempotencyStatus, IdempotencyStoreError> {
        let store = self.store.read().unwrap();
        Ok(store
            .get(key)
            .cloned()
            .unwrap_or(IdempotencyStatus::Unknown))
    }

    async fn start(&self, key: &str) -> Result<bool, IdempotencyStoreError> {
        let mut store = self.store.write().unwrap();
        match store.get(key) {
            Some(IdempotencyStatus::InProgress) | Some(IdempotencyStatus::Completed) => Ok(false),
            _ => {
                store.insert(key.to_string(), IdempotencyStatus::InProgress);
                Ok(true)
            }
        }
    }

    async fn complete(&self, key: &str) -> Result<(), IdempotencyStoreError> {
        let mut store = self.store.write().unwrap();
        store.insert(key.to_string(), IdempotencyStatus::Completed);
        Ok(())
    }

    async fn fail(&self, key: &str, error: String) -> Result<(), IdempotencyStoreError> {
        let mut store = self.store.write().unwrap();
        store.insert(key.to_string(), IdempotencyStatus::Failed(error));
        Ok(())
    }

    async fn is_already_processed(&self, key: &str) -> Result<bool, IdempotencyStoreError> {
        let store = self.store.read().unwrap();
        Ok(matches!(store.get(key), Some(IdempotencyStatus::Completed)))
    }
}

// ── Memory Dedupe Store ──────────────────────────────────────

/// In-memory deduplication store for testing.
pub struct MemoryDedupeStore {
    seen: RwLock<HashSet<String>>,
}

impl MemoryDedupeStore {
    pub fn new() -> Self {
        Self {
            seen: RwLock::new(HashSet::new()),
        }
    }
}

impl Default for MemoryDedupeStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DedupeStore for MemoryDedupeStore {
    async fn is_duplicate(&self, id: &str) -> Result<bool, DedupeStoreError> {
        let seen = self.seen.read().unwrap();
        Ok(seen.contains(id))
    }

    async fn mark_processed(&self, id: &str) -> Result<bool, DedupeStoreError> {
        let mut seen = self.seen.write().unwrap();
        Ok(seen.insert(id.to_string()))
    }
}
