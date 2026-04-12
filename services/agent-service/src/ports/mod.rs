//! External dependency abstractions for agent service.

use async_trait::async_trait;
use feature_agent::AgentError;
use std::pin::Pin;

/// Stream of SSE text chunks from an LLM.
pub type LlmStream = Pin<Box<dyn futures_util::Stream<Item = Result<String, AgentError>> + Send>>;

/// Abstract LLM provider interface.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stream a chat completion with tool support.
    async fn chat_stream(
        &self,
        messages: Vec<serde_json::Value>,
        tools: Vec<serde_json::Value>,
        model: &str,
    ) -> Result<LlmStream, AgentError>;
}

/// Abstract tool executor interface.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a named tool with the given arguments.
    async fn execute(
        &self,
        name: &str,
        arguments: serde_json::Value,
        conversation_id: &str,
    ) -> Result<String, AgentError>;
}
