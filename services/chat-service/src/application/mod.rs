use uuid::Uuid;

use crate::domain::{Conversation, Message, MessageSender, Participant};
use crate::domain::error::ChatError;
use crate::ports::{ConversationRepository, MessageRepository, ParticipantRepository, ChatEventPublisher};

/// Chat service — orchestrates conversation, message, and participant operations
pub struct ChatService<C, M, P, E> {
    conversation_repo: C,
    message_repo: M,
    participant_repo: P,
    event_publisher: E,
}

impl<C, M, P, E> ChatService<C, M, P, E>
where
    C: ConversationRepository,
    M: MessageRepository,
    P: ParticipantRepository,
    E: ChatEventPublisher,
{
    pub fn new(
        conversation_repo: C,
        message_repo: M,
        participant_repo: P,
        event_publisher: E,
    ) -> Self {
        Self {
            conversation_repo,
            message_repo,
            participant_repo,
            event_publisher,
        }
    }

    /// Create a new conversation and add the creator as participant
    pub async fn create_conversation(
        &self,
        tenant_id: String,
        title: Option<String>,
        creator_user_sub: String,
    ) -> Result<Conversation, ChatError> {
        let conversation = Conversation::new(tenant_id, title);
        self.conversation_repo.create(&conversation).await?;

        let participant = Participant::new(conversation.id, creator_user_sub);
        self.participant_repo.add(&participant).await?;

        self.event_publisher
            .publish_conversation_created(&conversation)
            .await?;

        Ok(conversation)
    }

    /// List conversations for a tenant
    pub async fn list_conversations(&self, tenant_id: &str) -> Result<Vec<Conversation>, ChatError> {
        self.conversation_repo.list_by_tenant(tenant_id).await
    }

    /// Get a single conversation by ID
    pub async fn get_conversation(&self, id: &Uuid) -> Result<Conversation, ChatError> {
        self.conversation_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ChatError::ConversationNotFound(id.to_string()))
    }

    /// Send a message to a conversation
    pub async fn send_message(
        &self,
        conversation_id: Uuid,
        sender: MessageSender,
        content: String,
    ) -> Result<Message, ChatError> {
        if content.trim().is_empty() {
            return Err(ChatError::InvalidInput("Message content cannot be empty".into()));
        }

        // Verify conversation exists
        self.get_conversation(&conversation_id).await?;

        let message = Message::new(conversation_id, sender, content);
        self.message_repo.save(&message).await?;
        self.event_publisher.publish_message_sent(&message).await?;

        Ok(message)
    }

    /// Get messages for a conversation with pagination
    pub async fn get_messages(
        &self,
        conversation_id: &Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, ChatError> {
        self.message_repo
            .list_by_conversation(conversation_id, limit, offset)
            .await
    }

    /// Get participants for a conversation
    pub async fn get_participants(&self, conversation_id: &Uuid) -> Result<Vec<Participant>, ChatError> {
        self.participant_repo.list_by_conversation(conversation_id).await
    }

    /// Update conversation title
    pub async fn update_title(&self, conversation_id: &Uuid, title: &str) -> Result<(), ChatError> {
        if title.trim().is_empty() {
            return Err(ChatError::InvalidInput("Title cannot be empty".into()));
        }
        self.conversation_repo.update_title(conversation_id, title).await
    }
}
