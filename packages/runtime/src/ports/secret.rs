//! Secret port — secure credential management abstraction.
//!
//! This port defines the interface for reading and managing secrets
//! (API keys, tokens, certificates), whether from environment variables,
//! SOPS, Vault, AWS Secrets Manager, or in-memory (for testing).
//!
//! ## Design principles
//! - Secrets are never logged or exposed in error messages
//! - Access to secrets is auditable
//! - Secrets can be rotated without restarting services

use async_trait::async_trait;
use serde::Serialize;

/// Error types for secret operations.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    NotFound(String),

    #[error("Access denied: insufficient permissions for {0}")]
    AccessDenied(String),

    #[error("Secret expired: {0}")]
    Expired(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Invalid secret name: {0}")]
    InvalidName(String),
}

/// A secret value with metadata.
#[derive(Debug, Clone, Serialize)]
pub struct SecretEntry {
    /// Secret identifier (e.g., "database-url", "api-key").
    pub name: String,
    /// The secret value (should be handled carefully to avoid logging).
    pub value: String,
    /// Optional version for rotation tracking.
    pub version: Option<String>,
    /// Timestamp when the secret expires (RFC3339, None if no expiry).
    pub expires_at: Option<String>,
    /// Timestamp when the secret was last updated (RFC3339).
    pub updated_at: String,
}

impl SecretEntry {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            version: None,
            expires_at: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_expiry(mut self, expires_at: impl Into<String>) -> Self {
        self.expires_at = Some(expires_at.into());
        self
    }

    /// Check if the secret has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.expires_at
            && let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at)
        {
            return chrono::Utc::now() > expiry;
        }
        false
    }
}

/// The Secret port — secure credential management abstraction.
///
/// ## Usage
/// ```ignore
/// // Get a secret value
/// let db_url = secrets.get("database-url").await?;
///
/// // Get with optional version
/// let api_key = secrets.get_versioned("api-key", Some("v2")).await?;
///
/// // List available secrets (names only, not values)
/// let names = secrets.list_names().await?;
/// ```
#[async_trait]
pub trait Secret: Send + Sync {
    /// Get a secret value by name.
    ///
    /// Returns `SecretError::NotFound` if the secret doesn't exist.
    /// The value should NEVER be logged or exposed in error messages.
    async fn get(&self, name: &str) -> Result<SecretEntry, SecretError>;

    /// Get a specific version of a secret.
    ///
    /// If version is None, returns the latest version.
    async fn get_versioned(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<SecretEntry, SecretError>;

    /// List all available secret names (without values).
    async fn list_names(&self) -> Result<Vec<String>, SecretError>;

    /// Set a secret value (for testing or local development).
    ///
    /// In production, this may be read-only depending on the backend.
    async fn set(&self, entry: SecretEntry) -> Result<(), SecretError>;

    /// Delete a secret (for testing or local development).
    ///
    /// In production, this may be disabled depending on the backend.
    async fn delete(&self, name: &str) -> Result<(), SecretError>;
}
