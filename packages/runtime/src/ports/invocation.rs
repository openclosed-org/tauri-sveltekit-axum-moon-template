//! Invocation port — synchronous request/response abstraction.
//!
//! This port defines the interface for invoking services synchronously,
//! whether via direct in-process calls, HTTP, RPC, or Dapr.
//!
//! ## Design principles
//! - Services depend on this port, NOT on concrete invocation mechanisms
//! - Adapters implement this trait for direct calls, HTTP clients, gRPC, etc.
//! - All calls carry correlation IDs for distributed tracing

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Error types for invocation.
#[derive(Debug, thiserror::Error)]
pub enum InvocationError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invocation timeout: {0}")]
    Timeout(String),

    #[error("Service returned error: {code} - {message}")]
    ServiceError { code: String, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Transport error: {0}")]
    TransportError(String),
}

/// Request context for distributed tracing and correlation.
#[derive(Debug, Clone, Serialize)]
pub struct InvocationContext {
    /// Correlation ID for tracing across service boundaries.
    pub correlation_id: String,
    /// The calling service identifier.
    pub caller_id: String,
    /// Optional tenant identifier for multi-tenant isolation.
    pub tenant_id: Option<String>,
    /// Optional timeout override for this specific invocation.
    pub timeout_ms: Option<u64>,
}

impl InvocationContext {
    pub fn new(correlation_id: impl Into<String>, caller_id: impl Into<String>) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            caller_id: caller_id.into(),
            tenant_id: None,
            timeout_ms: None,
        }
    }

    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Invocation request — a typed synchronous call to a service method.
#[derive(Debug, Clone, Serialize)]
pub struct InvocationRequest<Req = serde_json::Value> {
    /// Target service identifier (e.g., "user-service").
    pub service_id: String,
    /// Method or operation name (e.g., "GetUser").
    pub method: String,
    /// Request payload.
    pub payload: Req,
    /// Invocation context for tracing.
    pub context: InvocationContext,
}

impl<Req> InvocationRequest<Req> {
    pub fn new(
        service_id: impl Into<String>,
        method: impl Into<String>,
        payload: Req,
        context: InvocationContext,
    ) -> Self {
        Self {
            service_id: service_id.into(),
            method: method.into(),
            payload,
            context,
        }
    }
}

/// Invocation response — a typed result from a service call.
#[derive(Debug, Clone)]
pub struct InvocationResponse<Resp = serde_json::Value> {
    /// Response payload.
    pub payload: Resp,
    /// Optional correlation ID echoed back.
    pub correlation_id: Option<String>,
    /// Duration of the invocation in milliseconds.
    pub duration_ms: u64,
}

/// The Invocation port — synchronous request/response abstraction.
///
/// ## Usage
/// ```ignore
/// let response = invoker
///     .invoke(InvocationRequest::new(
///         "user-service",
///         "GetUser",
///         GetUserRequest { user_id: "123" },
///         InvocationContext::new("corr-123", "web-bff"),
///     ))
///     .await?;
/// ```
#[async_trait]
pub trait Invocation: Send + Sync {
    /// Invoke a service method synchronously.
    ///
    /// The request and response types must be serializable.
    /// The adapter handles serialization, transport, and deserialization.
    async fn invoke<Req: Serialize + Send, Resp: DeserializeOwned + Send>(
        &self,
        request: InvocationRequest<Req>,
    ) -> Result<InvocationResponse<Resp>, InvocationError>;
}
