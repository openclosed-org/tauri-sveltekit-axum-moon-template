use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

/// DTO for a conversation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub participant_count: usize,
    pub last_message_at: Option<String>,
}

/// DTO for a message in API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDto {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: String, // "user:{sub}", "agent:{id}", or "system"
    pub content: String,
    pub created_at: String,
}

/// DTO for sending a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// DTO for a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantDto {
    pub user_sub: String,
    pub joined_at: String,
}
