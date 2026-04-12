//! Workflow port — long-running orchestration abstraction.
//!
//! This port defines the interface for starting, managing, and querying
//! long-running workflows (e.g., tenant onboarding, user registration).
//!
//! ## Design principles
//! - Workflows are identified by unique instance IDs
//! - Each workflow has a type, status, and optional input/output
//! - Workflows can be paused, resumed, and queried for current state

use async_trait::async_trait;
use serde::{Deserialize, de::DeserializeOwned, Serialize};

/// Error types for workflow operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(String),

    #[error("Workflow already exists: {0}")]
    AlreadyExists(String),

    #[error("Workflow failed: {reason}")]
    Failed { reason: String },

    #[error("Workflow timeout: {0}")]
    Timeout(String),

    #[error("Invalid workflow state transition: {0}")]
    InvalidTransition(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Workflow status enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is queued and waiting to start.
    Pending,
    /// Workflow is currently executing.
    Running,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed with an error.
    Failed,
    /// Workflow is paused and can be resumed.
    Paused,
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowStatus::Pending => write!(f, "pending"),
            WorkflowStatus::Running => write!(f, "running"),
            WorkflowStatus::Completed => write!(f, "completed"),
            WorkflowStatus::Failed => write!(f, "failed"),
            WorkflowStatus::Paused => write!(f, "paused"),
        }
    }
}

/// A workflow instance representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance<Input = serde_json::Value, Output = serde_json::Value> {
    /// Unique workflow instance identifier.
    pub instance_id: String,
    /// Workflow type (e.g., "tenant-onboarding").
    pub workflow_type: String,
    /// Input data for the workflow.
    pub input: Option<Input>,
    /// Output data after completion (None if not completed).
    pub output: Option<Output>,
    /// Current workflow status.
    pub status: WorkflowStatus,
    /// Timestamp when the workflow started (RFC3339).
    pub started_at: String,
    /// Timestamp when the workflow completed (RFC3339, None if still running).
    pub completed_at: Option<String>,
    /// Optional tenant scope.
    pub tenant_id: Option<String>,
}

impl<Input, Output> WorkflowInstance<Input, Output> {
    pub fn new(instance_id: impl Into<String>, workflow_type: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            workflow_type: workflow_type.into(),
            input: None,
            output: None,
            status: WorkflowStatus::Pending,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            tenant_id: None,
        }
    }
}

/// The Workflow port — long-running orchestration abstraction.
///
/// ## Usage
/// ```ignore
/// // Start a new workflow
/// let instance = workflow.start(
///     "tenant-onboarding",
///     TenantOnboardingInput { tenant_id: "123", owner: "user-456" },
///     Some("tenant-123"),
/// ).await?;
///
/// // Query workflow status
/// let status = workflow.get_status(&instance.instance_id).await?;
///
/// // Wait for completion
/// let result = workflow.wait_for_completion(&instance.instance_id).await?;
/// ```
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Start a new workflow instance.
    ///
    /// Returns the created workflow instance with a unique ID.
    async fn start<Input: Serialize + Send, Output: DeserializeOwned + Send>(
        &self,
        workflow_type: &str,
        input: Input,
        tenant_id: Option<&str>,
    ) -> Result<WorkflowInstance<Input, Output>, WorkflowError>;

    /// Get the current status of a workflow instance.
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowStatus, WorkflowError>;

    /// Get the full workflow instance details.
    async fn get_instance<Input: DeserializeOwned + Send, Output: DeserializeOwned + Send>(
        &self,
        instance_id: &str,
    ) -> Result<WorkflowInstance<Input, Output>, WorkflowError>;

    /// Pause a running workflow.
    async fn pause(&self, instance_id: &str) -> Result<(), WorkflowError>;

    /// Resume a paused workflow.
    async fn resume(&self, instance_id: &str) -> Result<(), WorkflowError>;

    /// Wait for a workflow to complete (with timeout).
    async fn wait_for_completion<Output: DeserializeOwned + Send>(
        &self,
        instance_id: &str,
        timeout_ms: u64,
    ) -> Result<Output, WorkflowError>;

    /// List workflow instances for a given type and optional tenant.
    async fn list_instances(
        &self,
        workflow_type: &str,
        tenant_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<WorkflowInstance>, WorkflowError>;
}
