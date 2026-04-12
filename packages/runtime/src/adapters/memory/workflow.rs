//! In-memory workflow adapter.
//!
//! Stores workflow instances in memory with state machine transitions.
//! No persistence — workflows are lost on restart.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::debug;

use crate::ports::{Workflow, WorkflowError, WorkflowInstance, WorkflowStatus};

/// In-memory workflow adapter for testing.
///
/// Manages workflow instances with basic state machine transitions.
/// Does NOT actually execute workflow logic — just tracks state.
pub struct MemoryWorkflow {
    instances: RwLock<HashMap<String, WorkflowInstance>>,
}

impl MemoryWorkflow {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Workflow for MemoryWorkflow {
    async fn start<Input: Serialize + Send, Output: DeserializeOwned + Send>(
        &self,
        workflow_type: &str,
        input: Input,
        tenant_id: Option<&str>,
    ) -> Result<WorkflowInstance<Input, Output>, WorkflowError> {
        let instance_id = uuid::Uuid::now_v7().to_string();
        let mut instance = WorkflowInstance::new(&instance_id, workflow_type);
        instance.input = Some(input);
        instance.tenant_id = tenant_id.map(String::from);
        instance.status = WorkflowStatus::Running;

        // Serialize to generic instance for storage
        let generic_instance = WorkflowInstance {
            instance_id: instance.instance_id.clone(),
            workflow_type: instance.workflow_type.clone(),
            input: Some(serde_json::to_value(&instance.input)
                .map_err(|e| WorkflowError::SerializationError(e.to_string()))?),
            output: None,
            status: instance.status.clone(),
            started_at: instance.started_at.clone(),
            completed_at: instance.completed_at.clone(),
            tenant_id: instance.tenant_id.clone(),
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), generic_instance);

        debug!(instance_id = %instance_id, workflow_type = %workflow_type, "workflow started");
        Ok(instance)
    }

    async fn get_status(&self, instance_id: &str) -> Result<WorkflowStatus, WorkflowError> {
        let instances = self.instances.read().await;
        let instance = instances
            .get(instance_id)
            .ok_or_else(|| WorkflowError::NotFound(instance_id.to_string()))?;
        Ok(instance.status.clone())
    }

    async fn get_instance<Input: DeserializeOwned + Send, Output: DeserializeOwned + Send>(
        &self,
        instance_id: &str,
    ) -> Result<WorkflowInstance<Input, Output>, WorkflowError> {
        let instances = self.instances.read().await;
        let generic = instances
            .get(instance_id)
            .ok_or_else(|| WorkflowError::NotFound(instance_id.to_string()))?;

        let input: Option<Input> = generic
            .input
            .as_ref()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))
            })
            .transpose()?;

        let output: Option<Output> = generic
            .output
            .as_ref()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))
            })
            .transpose()?;

        Ok(WorkflowInstance {
            instance_id: generic.instance_id.clone(),
            workflow_type: generic.workflow_type.clone(),
            input,
            output,
            status: generic.status.clone(),
            started_at: generic.started_at.clone(),
            completed_at: generic.completed_at.clone(),
            tenant_id: generic.tenant_id.clone(),
        })
    }

    async fn pause(&self, instance_id: &str) -> Result<(), WorkflowError> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| WorkflowError::NotFound(instance_id.to_string()))?;

        if instance.status != WorkflowStatus::Running {
            return Err(WorkflowError::InvalidTransition(
                "can only pause running workflows".to_string(),
            ));
        }

        instance.status = WorkflowStatus::Paused;
        debug!(instance_id = %instance_id, "workflow paused");
        Ok(())
    }

    async fn resume(&self, instance_id: &str) -> Result<(), WorkflowError> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| WorkflowError::NotFound(instance_id.to_string()))?;

        if instance.status != WorkflowStatus::Paused {
            return Err(WorkflowError::InvalidTransition(
                "can only resume paused workflows".to_string(),
            ));
        }

        instance.status = WorkflowStatus::Running;
        debug!(instance_id = %instance_id, "workflow resumed");
        Ok(())
    }

    async fn wait_for_completion<Output: DeserializeOwned + Send>(
        &self,
        instance_id: &str,
        timeout_ms: u64,
    ) -> Result<Output, WorkflowError> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout {
                return Err(WorkflowError::Timeout(format!(
                    "workflow {} did not complete within {}ms",
                    instance_id, timeout_ms
                )));
            }

            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(instance_id) {
                match &instance.status {
                    WorkflowStatus::Completed => {
                        let output: Output = instance
                            .output
                            .as_ref()
                            .map(|v| {
                                serde_json::from_value(v.clone())
                                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))
                            })
                            .ok_or_else(|| WorkflowError::NotFound("output not found".to_string()))??;
                        return Ok(output);
                    }
                    WorkflowStatus::Failed => {
                        return Err(WorkflowError::Failed {
                            reason: "workflow failed".to_string(),
                        });
                    }
                    _ => {
                        drop(instances);
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            } else {
                return Err(WorkflowError::NotFound(instance_id.to_string()));
            }
        }
    }

    async fn list_instances(
        &self,
        workflow_type: &str,
        tenant_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<WorkflowInstance>, WorkflowError> {
        let instances = self.instances.read().await;
        let mut result: Vec<WorkflowInstance> = instances
            .values()
            .filter(|i| {
                i.workflow_type == workflow_type
                    && (tenant_id.is_none() || i.tenant_id.as_deref() == tenant_id)
            })
            .take(limit)
            .cloned()
            .collect();

        // Sort by started_at descending
        result.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(result)
    }
}
