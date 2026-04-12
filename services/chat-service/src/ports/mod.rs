use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Conversation, Message, Participant};
use crate::domain::error::ChatError;

/// Repository for conversation CRUD
#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn create(&self, conversation: &Conversation) -> Result<(), ChatError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Conversation>, ChatError>;
    async fn list_by_tenant(&self, tenant_id: &str) -> Result<Vec<Conversation>, ChatError>;
    async fn update_title(&self, id: &Uuid, title: &str) -> Result<(), ChatError>;
    async fn delete(&self, id: &Uuid) -> Result<(), ChatError>;
}

/// Repository for message storage
#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn save(&self, message: &Message) -> Result<(), ChatError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Message>, ChatError>;
    async fn list_by_conversation(
        &self,
        conversation_id: &Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, ChatError>;
}

/// Repository for participant management
#[async_trait]
pub trait ParticipantRepository: Send + Sync {
    async fn add(&self, participant: &Participant) -> Result<(), ChatError>;
    async fn list_by_conversation(&self, conversation_id: &Uuid) -> Result<Vec<Participant>, ChatError>;
    async fn remove(&self, conversation_id: &Uuid, user_sub: &str) -> Result<(), ChatError>;
}

/// Event publisher for chat events
#[async_trait]
pub trait ChatEventPublisher: Send + Sync {
    async fn publish_message_sent(&self, message: &Message) -> Result<(), ChatError>;
    async fn publish_conversation_created(&self, conversation: &Conversation) -> Result<(), ChatError>;
}
