use async_trait::async_trait;

use crate::domain::{Conversation, Message};
use crate::domain::error::ChatError;
use crate::ports::ChatEventPublisher;

/// In-memory event publisher for chat events
/// In production, this would publish to the event bus (NATS or similar)
pub struct InMemoryChatEventPublisher;

#[async_trait]
impl ChatEventPublisher for InMemoryChatEventPublisher {
    async fn publish_message_sent(&self, _message: &Message) -> Result<(), ChatError> {
        // In production: publish to event bus
        // For now: no-op (events are written to outbox table by service layer)
        Ok(())
    }

    async fn publish_conversation_created(&self, _conversation: &Conversation) -> Result<(), ChatError> {
        // In production: publish to event bus
        Ok(())
    }
}
