# Chat Service

Chat domain service implementing Clean Architecture four-layer pattern.

## Architecture

```
domain/          → Conversation, Message, Participant entities + ChatError
application/     → ChatService (create conversation, send message, list, etc.)
ports/           → ConversationRepository, MessageRepository, ParticipantRepository, ChatEventPublisher
infrastructure/  → LibSQL implementations
events/          → In-memory event publisher (production: event bus)
contracts/       → API DTOs (CreateConversationRequest, MessageDto, etc.)
```

## Use Cases

| Use Case | Method | Description |
|----------|--------|-------------|
| Create conversation | `create_conversation()` | New chat with creator as participant |
| List conversations | `list_conversations()` | All conversations for a tenant |
| Get conversation | `get_conversation()` | Single conversation by ID |
| Send message | `send_message()` | Add message to conversation |
| Get messages | `get_messages()` | Paginated message list |
| Get participants | `get_participants()` | List conversation participants |
| Update title | `update_title()` | Rename conversation |

## Testing

```bash
cargo test -p chat-service
```

## Migrations

```bash
# Migration: 001_create_chat_tables.sql
# Creates: conversations, messages, participants tables
```
