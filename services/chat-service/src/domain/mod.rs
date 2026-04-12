pub mod error;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub tenant_id: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A message within a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: MessageSender,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Who sent the message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSender {
    /// A human user
    User { user_sub: String },
    /// An AI agent
    Agent { agent_id: String },
    /// A system message
    System,
}

/// A participant in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub user_sub: String,
    pub joined_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(tenant_id: String, title: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            tenant_id,
            title,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Message {
    pub fn new(conversation_id: Uuid, sender: MessageSender, content: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            conversation_id,
            sender,
            content,
            created_at: Utc::now(),
        }
    }
}

impl Participant {
    pub fn new(conversation_id: Uuid, user_sub: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            conversation_id,
            user_sub,
            joined_at: Utc::now(),
        }
    }
}
