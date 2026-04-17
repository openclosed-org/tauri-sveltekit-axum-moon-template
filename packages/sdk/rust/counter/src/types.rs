//! Counter SDK types — shared across all backends.

use thiserror::Error;

/// Opaque identifier for a Counter aggregate.
///
/// App shells use this as an opaque string; the SDK backends
/// map it to whatever internal representation they need.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CounterId(pub String);

impl CounterId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CounterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors returned by counter operations via the SDK.
#[derive(Debug, Error)]
pub enum CounterError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Counter not found: {0}")]
    NotFound(String),

    #[error("CAS conflict: counter was modified by another writer")]
    CasConflict,
}
