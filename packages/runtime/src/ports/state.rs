//! State port — key-value state persistence abstraction.
//!
//! This port defines the interface for reading and writing service state,
//! whether via SQLite, Turso, SurrealDB, Redis, or in-memory stores.
//!
//! ## Design principles
//! - Services depend on this port, NOT on concrete databases
//! - State is identified by unique keys (optionally scoped by tenant)
//! - Supports optimistic concurrency control via version numbers

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Error types for state operations.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

/// A versioned state entry for optimistic concurrency control.
#[derive(Debug, Clone, Serialize)]
pub struct StateEntry<Value> {
    /// The stored value.
    pub value: Value,
    /// Current version number (increments on each write).
    pub version: u64,
    /// Key identifying this entry.
    pub key: String,
    /// Optional tenant scope for multi-tenant isolation.
    pub tenant_id: Option<String>,
    /// Timestamp when the entry was last updated (RFC3339).
    pub updated_at: String,
}

impl<Value> StateEntry<Value> {
    pub fn new(key: impl Into<String>, value: Value, tenant_id: Option<String>) -> Self {
        Self {
            value,
            version: 1,
            key: key.into(),
            tenant_id,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// The State port — key-value state persistence abstraction.
///
/// ## Usage
/// ```ignore
/// // Writing state
/// let entry = StateEntry::new("counter:123", CounterState { value: 42 }, Some("tenant-1"));
/// state.set(entry).await?;
///
/// // Reading state with optimistic concurrency
/// let mut entry = state.get::<CounterState>("counter:123").await?;
/// entry.value.value = 43;
/// state.set(entry).await?; // Fails if version changed
/// ```
#[async_trait]
pub trait State: Send + Sync {
    /// Get a state entry by key.
    ///
    /// Returns `StateError::NotFound` if the key doesn't exist.
    async fn get<Value: DeserializeOwned + Send>(&self, key: &str) -> Result<StateEntry<Value>, StateError>;

    /// Set a state entry with optimistic concurrency control.
    ///
    /// If the entry's version doesn't match the stored version,
    /// returns `StateError::VersionConflict`.
    async fn set<Value: Serialize + Send>(&self, entry: StateEntry<Value>) -> Result<(), StateError>;

    /// Delete a state entry by key.
    ///
    /// Returns `Ok(())` even if the key doesn't exist.
    async fn delete(&self, key: &str) -> Result<(), StateError>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, StateError>;

    /// List all keys matching a prefix pattern.
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StateError>;
}
