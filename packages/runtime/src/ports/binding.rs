//! Binding port — service wiring and dependency injection abstraction.
//!
//! This port defines the interface for composing services, workers, and adapters
//! at runtime. It handles dependency injection, feature flag evaluation,
//! and runtime capability discovery.
//!
//! ## Design principles
//! - Services declare what capabilities they need (not how they're implemented)
//! - Binding resolves implementations based on configuration and environment
//! - Enables seamless switching between direct, Dapr, and test adapters

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error types for binding operations.
#[derive(Debug, thiserror::Error)]
pub enum BindingError {
    #[error("Capability not available: {0}")]
    CapabilityNotAvailable(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionFailed(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

/// Runtime mode indicating which adapter type is active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeMode {
    /// Direct in-process calls (single-node deployment).
    Direct,
    /// Dapr sidecar (distributed deployment with Dapr).
    Dapr,
    /// Custom adapters (user-provided implementations).
    Custom,
    /// In-memory (testing only).
    Memory,
}

impl std::fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeMode::Direct => write!(f, "direct"),
            RuntimeMode::Dapr => write!(f, "dapr"),
            RuntimeMode::Custom => write!(f, "custom"),
            RuntimeMode::Memory => write!(f, "memory"),
        }
    }
}

/// Capability registration for dependency injection.
#[derive(Debug, Clone)]
pub struct CapabilityBinding {
    /// Capability name (e.g., "pubsub", "state", "invocation").
    pub capability: String,
    /// Adapter identifier (e.g., "nats", "turso", "memory").
    pub adapter: String,
    /// Configuration for this adapter (adapter-specific).
    pub config: HashMap<String, String>,
}

impl CapabilityBinding {
    pub fn new(capability: impl Into<String>, adapter: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            adapter: adapter.into(),
            config: HashMap::new(),
        }
    }

    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

/// The Binding port — service wiring and dependency injection.
///
/// ## Usage
/// ```ignore
/// // Check if a capability is available
/// if binding.is_available("pubsub")? {
///     let pubsub = binding.resolve_pubsub()?;
///     // use pubsub...
/// }
///
/// // Get runtime mode for adapter selection
/// let mode = binding.runtime_mode()?;
/// ```
#[async_trait]
pub trait Binding: Send + Sync {
    /// Get the current runtime mode.
    fn runtime_mode(&self) -> Result<RuntimeMode, BindingError>;

    /// Check if a capability is available in the current configuration.
    fn is_available(&self, capability: &str) -> Result<bool, BindingError>;

    /// List all available capabilities.
    fn list_capabilities(&self) -> Result<Vec<String>, BindingError>;

    /// Get configuration for a specific capability.
    fn get_capability_config(&self, capability: &str) -> Result<HashMap<String, String>, BindingError>;

    /// Validate that all required dependencies are resolvable.
    ///
    /// Returns a list of unresolved dependencies if validation fails.
    async fn validate_dependencies(&self, required_capabilities: &[&str]) -> Result<(), BindingError>;
}
