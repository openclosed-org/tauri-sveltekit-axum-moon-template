//! Lock port — distributed locking abstraction.
//!
//! This port defines the interface for acquiring and releasing distributed locks,
//! whether via Redis, etcd, Consul, Dapr, or in-memory (for testing).
//!
//! ## Design principles
//! - Locks prevent concurrent operations on shared resources
//! - All locks have a TTL to prevent deadlocks
//! - Lock acquisition is cancellable with timeout

use async_trait::async_trait;
use std::time::Duration;

/// Error types for lock operations.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Failed to acquire lock: {0}")]
    AcquireFailed(String),

    #[error("Failed to release lock: {0}")]
    ReleaseFailed(String),

    #[error("Lock expired before operation completed")]
    Expired,

    #[error("Lock acquisition timed out after {0:?}")]
    Timeout(Duration),

    #[error("Lock already held by another owner")]
    AlreadyLocked,

    #[error("Invalid lock name: {0}")]
    InvalidName(String),
}

/// A distributed lock guard — automatically releases on drop.
pub struct LockGuard {
    lock_manager: Box<dyn LockManager>,
    lock_name: String,
    lock_token: String,
}

impl LockGuard {
    pub fn new(lock_manager: Box<dyn LockManager>, lock_name: String, lock_token: String) -> Self {
        Self {
            lock_manager,
            lock_name,
            lock_token,
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Synchronous release — in async contexts, use explicit release
        let name = self.lock_name.clone();
        let token = self.lock_token.clone();
        let lock_manager = std::mem::replace(&mut self.lock_manager, Box::new(StubLockManager));

        // Use tokio runtime if available, otherwise log warning
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = lock_manager.release_internal(&name, &token).await {
                        tracing::error!(error = %e, lock_name = %name, "failed to release lock on drop");
                    }
                });
            }
            Err(_) => {
                tracing::warn!(lock_name = %name, "could not release lock on drop (no async runtime)");
            }
        }
    }
}

/// Internal trait for lock management (public for adapter implementations).
#[async_trait]
pub trait LockManager: Send + Sync {
    async fn release_internal(&self, lock_name: &str, token: &str) -> Result<(), LockError>;
}

/// Stub implementation for Drop fallback.
struct StubLockManager;

#[async_trait]
impl LockManager for StubLockManager {
    async fn release_internal(&self, _lock_name: &str, _token: &str) -> Result<(), LockError> {
        Ok(())
    }
}

/// The Lock port — distributed locking abstraction.
///
/// ## Usage
/// ```ignore
/// // Acquire a lock with timeout
/// let guard = lock.acquire("tenant-onboarding:123", Duration::from_secs(30)).await?;
///
/// // Perform critical section...
/// // Lock is automatically released when guard goes out of scope
///
/// // Or explicitly release
/// drop(guard);
/// ```
#[async_trait]
pub trait Lock: Send + Sync {
    /// Acquire a distributed lock with a timeout.
    ///
    /// Returns a `LockGuard` that automatically releases the lock on drop.
    /// If the lock cannot be acquired within the timeout, returns `LockError::Timeout`.
    async fn acquire(&self, lock_name: &str, timeout: Duration) -> Result<LockGuard, LockError>;

    /// Try to acquire a lock without waiting.
    ///
    /// Returns `Err(LockError::AlreadyLocked)` if the lock is already held.
    async fn try_acquire(&self, lock_name: &str) -> Result<LockGuard, LockError>;
}
