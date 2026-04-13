use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use chat_service::application::ChatService;
use chat_service::domain::error::ChatError;
use chat_service::domain::{Conversation, Message, MessageSender, Participant};
use chat_service::ports::{
    ChatEventPublisher, ConversationRepository, MessageRepository, ParticipantRepository,
};

// ── Mock Repositories ─────────────────────────────────────────

#[derive(Clone, Default)]
struct MockConversationRepository {
    conversations: Arc<Mutex<Vec<Conversation>>>,
}

#[async_trait]
impl ConversationRepository for MockConversationRepository {
    async fn create(&self, conversation: &Conversation) -> Result<(), ChatError> {
        self.conversations.lock().await.push(conversation.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Conversation>, ChatError> {
        let conversations = self.conversations.lock().await;
        Ok(conversations.iter().find(|c| &c.id == id).cloned())
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> Result<Vec<Conversation>, ChatError> {
        let conversations = self.conversations.lock().await;
        Ok(conversations
            .iter()
            .filter(|c| c.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn update_title(&self, id: &Uuid, title: &str) -> Result<(), ChatError> {
        let mut conversations = self.conversations.lock().await;
        if let Some(conv) = conversations.iter_mut().find(|c| &c.id == id) {
            conv.title = Some(title.to_string());
            Ok(())
        } else {
            Err(ChatError::ConversationNotFound(id.to_string()))
        }
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ChatError> {
        let mut conversations = self.conversations.lock().await;
        let len_before = conversations.len();
        conversations.retain(|c| &c.id != id);
        if conversations.len() == len_before {
            Err(ChatError::ConversationNotFound(id.to_string()))
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Clone)]
struct MockMessageRepository {
    messages: Arc<Mutex<Vec<Message>>>,
}

#[async_trait]
impl MessageRepository for MockMessageRepository {
    async fn save(&self, message: &Message) -> Result<(), ChatError> {
        self.messages.lock().await.push(message.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Message>, ChatError> {
        let messages = self.messages.lock().await;
        Ok(messages.iter().find(|m| &m.id == id).cloned())
    }

    async fn list_by_conversation(
        &self,
        conversation_id: &Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, ChatError> {
        let messages = self.messages.lock().await;
        let filtered: Vec<Message> = messages
            .iter()
            .filter(|m| &m.conversation_id == conversation_id)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

#[derive(Default, Clone)]
struct MockParticipantRepository {
    participants: Arc<Mutex<Vec<Participant>>>,
}

#[async_trait]
impl ParticipantRepository for MockParticipantRepository {
    async fn add(&self, participant: &Participant) -> Result<(), ChatError> {
        self.participants.lock().await.push(participant.clone());
        Ok(())
    }

    async fn list_by_conversation(
        &self,
        conversation_id: &Uuid,
    ) -> Result<Vec<Participant>, ChatError> {
        let participants = self.participants.lock().await;
        Ok(participants
            .iter()
            .filter(|p| &p.conversation_id == conversation_id)
            .cloned()
            .collect())
    }

    async fn remove(&self, conversation_id: &Uuid, user_sub: &str) -> Result<(), ChatError> {
        let mut participants = self.participants.lock().await;
        let len_before = participants.len();
        participants.retain(|p| &p.conversation_id != conversation_id || p.user_sub != user_sub);
        if participants.len() == len_before {
            Err(ChatError::ConversationNotFound(conversation_id.to_string()))
        } else {
            Ok(())
        }
    }
}

struct MockEventPublisher;

#[async_trait]
impl ChatEventPublisher for MockEventPublisher {
    async fn publish_message_sent(&self, _message: &Message) -> Result<(), ChatError> {
        Ok(())
    }

    async fn publish_conversation_created(
        &self,
        _conversation: &Conversation,
    ) -> Result<(), ChatError> {
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_conversation_adds_creator_as_participant() {
    let conv_repo = MockConversationRepository::default();
    let msg_repo = MockMessageRepository::default();
    let part_repo = MockParticipantRepository::default();
    let events = MockEventPublisher;

    let service = ChatService::new(conv_repo, msg_repo, part_repo.clone(), events);

    let conversation = service
        .create_conversation(
            "tenant-1".to_string(),
            Some("Test Chat".to_string()),
            "user-abc".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(conversation.tenant_id, "tenant-1");
    assert_eq!(conversation.title, Some("Test Chat".to_string()));

    let participants = part_repo
        .list_by_conversation(&conversation.id)
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].user_sub, "user-abc");
}

#[tokio::test]
async fn test_send_message_rejects_empty_content() {
    let conv_repo = MockConversationRepository::default();
    let msg_repo = MockMessageRepository::default();
    let part_repo = MockParticipantRepository::default();
    let events = MockEventPublisher;

    let service = ChatService::new(conv_repo, msg_repo, part_repo, events);

    // Create a conversation first
    let conv = service
        .create_conversation("t1".to_string(), None, "u1".to_string())
        .await
        .unwrap();

    // Empty content should fail
    let result = service
        .send_message(
            conv.id,
            MessageSender::User {
                user_sub: "u1".to_string(),
            },
            "".to_string(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ChatError::InvalidInput(msg) => assert!(msg.contains("empty")),
        other => panic!("Expected InvalidInput, got {:?}", other),
    }
}

#[tokio::test]
async fn test_list_conversations_filters_by_tenant() {
    let conv_repo = MockConversationRepository::default();
    let msg_repo = MockMessageRepository::default();
    let part_repo = MockParticipantRepository::default();
    let events = MockEventPublisher;

    let service = ChatService::new(conv_repo.clone(), msg_repo, part_repo, events);

    // Create conversations in different tenants
    service
        .create_conversation(
            "tenant-a".to_string(),
            Some("A Chat".to_string()),
            "u1".to_string(),
        )
        .await
        .unwrap();

    service
        .create_conversation(
            "tenant-b".to_string(),
            Some("B Chat".to_string()),
            "u2".to_string(),
        )
        .await
        .unwrap();

    let tenant_a_convs = service.list_conversations("tenant-a").await.unwrap();
    assert_eq!(tenant_a_convs.len(), 1);
    assert_eq!(tenant_a_convs[0].title, Some("A Chat".to_string()));

    let tenant_b_convs = service.list_conversations("tenant-b").await.unwrap();
    assert_eq!(tenant_b_convs.len(), 1);
}
