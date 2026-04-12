//! In-memory invocation adapter.
//!
//! Provides a registry of service handlers that can be invoked synchronously.
//! Useful for testing service-to-service calls without HTTP or RPC.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;

use crate::ports::{
    Invocation, InvocationContext, InvocationError, InvocationRequest, InvocationResponse,
};

/// Type alias for service handler functions.
type ServiceHandler = Arc<
    dyn Fn(serde_json::Value) -> Result<serde_json::Value, InvocationError> + Send + Sync,
>;

/// In-memory invocation adapter for testing.
///
/// Register service handlers, then invoke them by service ID and method name.
pub struct MemoryInvocation {
    handlers: RwLock<HashMap<String, HashMap<String, ServiceHandler>>>,
}

impl MemoryInvocation {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a handler for a specific service and method.
    pub async fn register_handler<Req, Resp, F>(
        &self,
        service_id: &str,
        method: &str,
        handler: F,
    ) where
        Req: DeserializeOwned,
        Resp: Serialize,
        F: Fn(Req) -> Result<Resp, InvocationError> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        let service_handlers = handlers
            .entry(service_id.to_string())
            .or_insert_with(HashMap::new);

        let handler = Arc::new(move |payload: serde_json::Value| {
            let req: Req = serde_json::from_value(payload)
                .map_err(|e| InvocationError::SerializationError(e.to_string()))?;
            let resp = handler(req)?;
            serde_json::to_value(&resp)
                .map_err(|e| InvocationError::SerializationError(e.to_string()))
        });

        service_handlers.insert(method.to_string(), handler);
    }
}

impl Default for MemoryInvocation {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Invocation for MemoryInvocation {
    async fn invoke<Req: Serialize + Send, Resp: DeserializeOwned + Send>(
        &self,
        request: InvocationRequest<Req>,
    ) -> Result<InvocationResponse<Resp>, InvocationError> {
        let start = std::time::Instant::now();

        let handlers = self.handlers.read().await;
        let service_handlers = handlers
            .get(&request.service_id)
            .ok_or_else(|| InvocationError::ServiceNotFound(request.service_id.clone()))?;

        let handler = service_handlers
            .get(&request.method)
            .ok_or_else(|| InvocationError::MethodNotFound(request.method.clone()))?;

        let payload = serde_json::to_value(&request.payload)
            .map_err(|e| InvocationError::SerializationError(e.to_string()))?;

        let result_payload = handler(payload)?;

        let resp: Resp = serde_json::from_value(result_payload)
            .map_err(|e| InvocationError::SerializationError(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(InvocationResponse {
            payload: resp,
            correlation_id: Some(request.context.correlation_id),
            duration_ms,
        })
    }
}
