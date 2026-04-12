//! Agent domain errors.

/// Domain-level error for agent operations.
#[derive(Debug, thiserror::Error)]
pub enum AgentDomainError {
    #[error("Invalid conversation: {0}")]
    InvalidConversation(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}
