//! In-memory lock adapter.
//!
//! Uses a HashMap to track lock ownership with TTL-based expiry.
//! Not suitable for production — use Redis or etcd for real distributed locking.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::debug;

use crate::ports::{Lock, LockError, LockGuard, LockManager};

/// In-memory lock adapter for testing.
///
/// Provides basic distributed locking semantics using in-memory state.
/// Locks expire after a timeout to prevent deadlocks.
pub struct MemoryLock {
    locks: Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>,
}

impl MemoryLock {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryLock {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Lock for MemoryLock {
    async fn acquire(&self, lock_name: &str, timeout: Duration) -> Result<LockGuard, LockError> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(LockError::Timeout(timeout));
            }

            match self.try_acquire_lock(lock_name).await {
                Ok(token) => {
                    debug!(lock_name = %lock_name, "lock acquired");
                    let manager = MemoryLockManager {
                        locks: self.locks.clone(),
                    };
                    return Ok(LockGuard::new(
                        Box::new(manager),
                        lock_name.to_string(),
                        token,
                    ));
                }
                Err(LockError::AlreadyLocked) => {
                    sleep(Duration::from_millis(50)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_acquire(&self, lock_name: &str) -> Result<LockGuard, LockError> {
        let token = self.try_acquire_lock(lock_name).await?;
        debug!(lock_name = %lock_name, "lock acquired (try)");
        let manager = MemoryLockManager {
            locks: self.locks.clone(),
        };
        Ok(LockGuard::new(
            Box::new(manager),
            lock_name.to_string(),
            token,
        ))
    }
}

impl MemoryLock {
    async fn try_acquire_lock(&self, lock_name: &str) -> Result<String, LockError> {
        let mut locks = self.locks.write().await;

        // Check if lock exists and hasn't expired
        if let Some((token, acquired_at)) = locks.get(lock_name) {
            // Simple TTL check (5 second default TTL)
            if acquired_at.elapsed() < Duration::from_secs(5) {
                return Err(LockError::AlreadyLocked);
            }
            // Lock expired, remove it
            locks.remove(lock_name);
        }

        // Acquire the lock
        let token = uuid::Uuid::now_v7().to_string();
        locks.insert(
            lock_name.to_string(),
            (token.clone(), std::time::Instant::now()),
        );

        Ok(token)
    }
}

/// Internal lock manager for LockGuard cleanup.
struct MemoryLockManager {
    locks: Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>,
}

#[async_trait]
impl LockManager for MemoryLockManager {
    async fn release_internal(&self, lock_name: &str, token: &str) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;
        if let Some((stored_token, _)) = locks.get(lock_name)
            && stored_token == token
        {
            locks.remove(lock_name);
            debug!(lock_name = %lock_name, "lock released");
            return Ok(());
        }
        // Lock doesn't exist or token mismatch — may have expired
        Ok(())
    }
}
