# services/chat-service

> Chat domain service — conversations, messages, real-time streaming.

## Status
- [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
- [ ] Phase 1: Implement domain/application/ports
- [ ] Phase 2: Independent deployment

## Dependencies
- `packages/core/kernel` (TenantId, UserId, AppError)
- `packages/core/domain` (port traits)
- `packages/contracts/*` (HTTP/Event contracts)
- `packages/features/chat` (ChatService trait, ChatMessage struct)

## Architecture
- `domain/` — ChatMessage, ChatSession entities
- `application/` — Use cases (send_message, get_history)
- `ports/` — External dependency abstractions (MessageStore)
- `contracts/` — Stable contract definitions
- `sync/` — OfflineFirst sync strategies
- `infrastructure/` — WebSocket/SSE real-time implementations
- `interfaces/` — API route handlers
