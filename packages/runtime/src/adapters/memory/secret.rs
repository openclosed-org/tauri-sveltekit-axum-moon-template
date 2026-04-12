//! In-memory secret adapter.
//!
//! Stores secrets in memory for testing.
//! NOT SECURE — never use in production.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::ports::{Secret, SecretEntry, SecretError};

/// In-memory secret adapter for testing.
///
/// Stores secrets in plain text in memory.
/// NEVER use this in production.
pub struct MemorySecret {
    secrets: RwLock<HashMap<String, SecretEntry>>,
}

impl MemorySecret {
    pub fn new() -> Self {
        Self {
            secrets: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-populate with test secrets.
    pub fn with_secrets(mut self, secrets: Vec<SecretEntry>) -> Self {
        for secret in secrets {
            self.secrets
                .get_mut()
                .insert(secret.name.clone(), secret);
        }
        self
    }
}

impl Default for MemorySecret {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Secret for MemorySecret {
    async fn get(&self, name: &str) -> Result<SecretEntry, SecretError> {
        let secrets = self.secrets.read().await;
        secrets
            .get(name)
            .cloned()
            .ok_or_else(|| SecretError::NotFound(name.to_string()))
    }

    async fn get_versioned(&self, name: &str, version: Option<&str>) -> Result<SecretEntry, SecretError> {
        let secrets = self.secrets.read().await;
        let entry = secrets
            .get(name)
            .ok_or_else(|| SecretError::NotFound(name.to_string()))?;

        match version {
            Some(v) => {
                if entry.version.as_deref() == Some(v) {
                    Ok(entry.clone())
                } else {
                    Err(SecretError::NotFound(format!(
                        "version {} of secret {} not found",
                        v, name
                    )))
                }
            }
            None => Ok(entry.clone()),
        }
    }

    async fn list_names(&self) -> Result<Vec<String>, SecretError> {
        let secrets = self.secrets.read().await;
        Ok(secrets.keys().cloned().collect())
    }

    async fn set(&self, entry: SecretEntry) -> Result<(), SecretError> {
        let mut secrets = self.secrets.write().await;
        secrets.insert(entry.name.clone(), entry);
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<(), SecretError> {
        let mut secrets = self.secrets.write().await;
        secrets.remove(name);
        Ok(())
    }
}
