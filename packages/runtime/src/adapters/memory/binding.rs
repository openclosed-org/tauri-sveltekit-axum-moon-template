//! In-memory binding adapter.
//!
//! Tracks runtime mode and capability bindings in memory.
//! Used for testing service wiring without actual infrastructure.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::ports::{Binding, BindingError, CapabilityBinding, RuntimeMode};

/// In-memory binding adapter for testing.
///
/// Stores capability bindings in memory and reports runtime mode.
pub struct MemoryBinding {
    mode: RuntimeMode,
    capabilities: HashMap<String, HashMap<String, String>>,
}

impl MemoryBinding {
    pub fn new(mode: RuntimeMode) -> Self {
        Self {
            mode,
            capabilities: HashMap::new(),
        }
    }

    /// Register a capability with its configuration.
    pub fn register_capability(
        &mut self,
        capability: &str,
        config: HashMap<String, String>,
    ) {
        self.capabilities.insert(capability.to_string(), config);
    }

    /// Register a capability binding.
    pub fn register_binding(&mut self, binding: CapabilityBinding) {
        self.capabilities
            .insert(binding.capability, binding.config);
    }
}

#[async_trait]
impl Binding for MemoryBinding {
    fn runtime_mode(&self) -> Result<RuntimeMode, BindingError> {
        Ok(self.mode.clone())
    }

    fn is_available(&self, capability: &str) -> Result<bool, BindingError> {
        Ok(self.capabilities.contains_key(capability))
    }

    fn list_capabilities(&self) -> Result<Vec<String>, BindingError> {
        Ok(self.capabilities.keys().cloned().collect())
    }

    fn get_capability_config(&self, capability: &str) -> Result<HashMap<String, String>, BindingError> {
        self.capabilities
            .get(capability)
            .cloned()
            .ok_or_else(|| BindingError::CapabilityNotAvailable(capability.to_string()))
    }

    async fn validate_dependencies(&self, required_capabilities: &[&str]) -> Result<(), BindingError> {
        let mut unavailable = Vec::new();
        for &capability in required_capabilities {
            if !self.capabilities.contains_key(capability) {
                unavailable.push(capability.to_string());
            }
        }

        if unavailable.is_empty() {
            Ok(())
        } else {
            Err(BindingError::DependencyResolutionFailed(format!(
                "unavailable capabilities: {}",
                unavailable.join(", ")
            )))
        }
    }
}
