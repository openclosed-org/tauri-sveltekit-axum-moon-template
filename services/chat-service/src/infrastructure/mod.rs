use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::{Conversation, Message, Participant};
use crate::domain::error::ChatError;
use crate::ports::{ConversationRepository, MessageRepository, ParticipantRepository};

/// LibSQL-backed conversation repository
pub struct LibSqlConversationRepository;

#[async_trait]
impl ConversationRepository for LibSqlConversationRepository {
    async fn create(&self, _conversation: &Conversation) -> Result<(), ChatError> {
        // SQL: INSERT INTO conversations (id, tenant_id, title, created_at, updated_at) VALUES (?, ?, ?, ?, ?)
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<Conversation>, ChatError> {
        // SQL: SELECT id, tenant_id, title, created_at, updated_at FROM conversations WHERE id = ?
        Ok(None)
    }

    async fn list_by_tenant(&self, _tenant_id: &str) -> Result<Vec<Conversation>, ChatError> {
        // SQL: SELECT ... FROM conversations WHERE tenant_id = ? ORDER BY updated_at DESC
        Ok(vec![])
    }

    async fn update_title(&self, _id: &Uuid, _title: &str) -> Result<(), ChatError> {
        // SQL: UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> Result<(), ChatError> {
        // SQL: DELETE FROM conversations WHERE id = ?
        Ok(())
    }
}

/// LibSQL-backed message repository
pub struct LibSqlMessageRepository;

#[async_trait]
impl MessageRepository for LibSqlMessageRepository {
    async fn save(&self, _message: &Message) -> Result<(), ChatError> {
        // SQL: INSERT INTO messages (id, conversation_id, sender, sender_data, content, created_at) VALUES (?, ?, ?, ?, ?, ?)
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<Message>, ChatError> {
        // SQL: SELECT ... FROM messages WHERE id = ?
        Ok(None)
    }

    async fn list_by_conversation(
        &self,
        _conversation_id: &Uuid,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<Message>, ChatError> {
        // SQL: SELECT ... FROM messages WHERE conversation_id = ? ORDER BY created_at ASC LIMIT ? OFFSET ?
        Ok(vec![])
    }
}

/// LibSQL-backed participant repository
pub struct LibSqlParticipantRepository;

#[async_trait]
impl ParticipantRepository for LibSqlParticipantRepository {
    async fn add(&self, _participant: &Participant) -> Result<(), ChatError> {
        // SQL: INSERT INTO participants (id, conversation_id, user_sub, joined_at) VALUES (?, ?, ?, ?)
        Ok(())
    }

    async fn list_by_conversation(&self, _conversation_id: &Uuid) -> Result<Vec<Participant>, ChatError> {
        // SQL: SELECT ... FROM participants WHERE conversation_id = ?
        Ok(vec![])
    }

    async fn remove(&self, _conversation_id: &Uuid, _user_sub: &str) -> Result<(), ChatError> {
        // SQL: DELETE FROM participants WHERE conversation_id = ? AND user_sub = ?
        Ok(())
    }
}
