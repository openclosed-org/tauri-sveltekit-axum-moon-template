use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ChatError {
    #[error("Conversation not found: {0}")]
    ConversationNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Unauthorized: user {user_sub} cannot access conversation {conversation_id}")]
    Unauthorized {
        user_sub: String,
        conversation_id: String,
    },
}
