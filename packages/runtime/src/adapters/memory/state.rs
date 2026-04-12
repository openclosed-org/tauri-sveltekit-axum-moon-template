//! In-memory state adapter.
//!
//! Uses a HashMap to store key-value entries with version tracking.
//! Supports optimistic concurrency control.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;
use tracing::debug;

use crate::ports::{State, StateEntry, StateError};

/// Internal state entry for storage.
struct InternalEntry {
    value: serde_json::Value,
    version: u64,
    tenant_id: Option<String>,
    updated_at: String,
}

/// In-memory state adapter for testing.
///
/// Stores state in a HashMap with version tracking for optimistic concurrency.
pub struct MemoryState {
    store: RwLock<HashMap<String, InternalEntry>>,
}

impl MemoryState {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl State for MemoryState {
    async fn get<Value: DeserializeOwned + Send>(&self, key: &str) -> Result<StateEntry<Value>, StateError> {
        let store = self.store.read().await;
        let entry = store
            .get(key)
            .ok_or_else(|| StateError::NotFound(key.to_string()))?;

        let value: Value = serde_json::from_value(entry.value.clone())
            .map_err(|e| StateError::SerializationError(e.to_string()))?;

        Ok(StateEntry {
            value,
            version: entry.version,
            key: key.to_string(),
            tenant_id: entry.tenant_id.clone(),
            updated_at: entry.updated_at.clone(),
        })
    }

    async fn set<Value: Serialize + Send>(&self, entry: StateEntry<Value>) -> Result<(), StateError> {
        let mut store = self.store.write().await;

        let serialized_value = serde_json::to_value(&entry.value)
            .map_err(|e| StateError::SerializationError(e.to_string()))?;

        if let Some(existing) = store.get(&entry.key) {
            // Check version for optimistic concurrency
            if existing.version != entry.version {
                return Err(StateError::VersionConflict {
                    expected: existing.version,
                    actual: entry.version,
                });
            }

            // Update with incremented version
            store.insert(
                entry.key.clone(),
                InternalEntry {
                    value: serialized_value,
                    version: entry.version + 1,
                    tenant_id: entry.tenant_id,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                },
            );
        } else {
            // New entry
            store.insert(
                entry.key.clone(),
                InternalEntry {
                    value: serialized_value,
                    version: entry.version,
                    tenant_id: entry.tenant_id,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                },
            );
        }

        debug!(key = %entry.key, version = %entry.version, "state entry set");
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), StateError> {
        let mut store = self.store.write().await;
        store.remove(key);
        debug!(key = %key, "state entry deleted");
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, StateError> {
        let store = self.store.read().await;
        Ok(store.contains_key(key))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StateError> {
        let store = self.store.read().await;
        Ok(store
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}
