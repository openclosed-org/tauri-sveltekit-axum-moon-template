# Service Catalog

> Auto-generated catalog of all services in the platform.

## Overview

| Service | Type | Description | Status |
|---------|------|-------------|--------|
| auth-service | Business Logic | Authentication, sessions, OAuth flows | ✅ Implemented |
| user-service | Business Logic | User management, profiles, preferences | ✅ Implemented |
| tenant-service | Business Logic | Multi-tenant isolation, onboarding, members | ✅ Implemented |
| settings-service | Business Logic | User and system settings | ✅ Implemented |
| counter-service | Business Logic | Demo counter service | ✅ Implemented |
| agent-service | Business Logic | AI agent conversations | ✅ Implemented |
| chat-service | Business Logic | Chat and messaging | ✅ Implemented |
| admin-service | Business Logic | Admin dashboard and management | ✅ Implemented |
| indexing-service | Business Logic | Data indexing and search | ✅ Implemented |
| event-bus | Infrastructure | Event bus for cross-service communication | ✅ Implemented |

---

## auth-service

**Path**: `services/auth-service/`

### Description
Handles user authentication, session management, OAuth flows, and token generation/validation.

### Domain Entities
- User
- Session
- Token (Access/Refresh)

### Ports
- `TokenRepository` — JWT token generation and validation
- `SessionRepository` — Session persistence
- `OAuthProvider` — OAuth flow handling

### Adapters
- `JwtTokenRepository` — JWT implementation (15min access, 7d refresh)
- `LibSqlSessionRepository` — LibSQL-based session storage
- `MockOAuthProvider` — Mock OAuth for development

### Events
- `UserLoggedIn` — User successfully authenticated
- `TokenRefreshed` — Token was refreshed
- `UserCreated` — New user registered via OAuth

### Contracts
- `TokenPair` — Access + refresh tokens
- `OAuthCallback` — OAuth callback parameters
- `UserProfile` — User profile data
- `UserSession` — Authenticated session data

### Dependencies
- `packages/kernel` (ids, error, time)
- `packages/platform` (config, health)
- `packages/contracts/auth`
- `packages/data/turso`
- `packages/security/crypto`

---

## user-service

**Path**: `services/user-service/`

### Description
Manages user profiles, preferences, and tenant memberships.

### Domain Entities
- User
- UserProfile
- TenantMembership

### Ports
- `UserRepository` — User data persistence
- `EventPublisher` — Domain event publishing

### Events
- `UserCreated` — New user created
- `UserLoggedIn` — User logged in
- `UserUpdated` — User profile updated
- `UserDeleted` — User deleted
- `TenantInitialized` — Tenant initialized for user

### Contracts
- `InitTenantRequest` — Tenant initialization request
- `InitTenantResponse` — Tenant initialization response

### Dependencies
- `packages/kernel` (ids, error, tenancy, time)
- `packages/platform` (config, health)
- `packages/contracts/api`
- `packages/data/turso`

---

## tenant-service

**Path**: `services/tenant-service/`

### Description
Manages tenants, tenant isolation, member invitations, and role management.

### Domain Entities
- Tenant
- TenantMember
- Invitation

### Ports
- `TenantRepository` — Tenant data persistence
- `MemberRepository` — Member management
- `EventPublisher` — Domain event publishing

### Events
- `TenantCreated` — New tenant created
- `TenantUpdated` — Tenant updated
- `TenantDeleted` — Tenant deleted
- `MemberAdded` — Member added to tenant
- `MemberRemoved` — Member removed from tenant
- `MemberRoleChanged` — Member role changed

### Dependencies
- `packages/kernel` (ids, error, tenancy, time)
- `packages/platform` (config, health)
- `packages/contracts/api`
- `packages/data/turso`
- `packages/authz` (tenant isolation)

---

## settings-service

**Path**: `services/settings-service/`

### Description
Manages user and system settings with PII redaction.

### Domain Entities
- Settings

### Ports
- `SettingsRepository` — Settings persistence

### Contracts
- Settings DTO (API key redacted)

### Dependencies
- `packages/kernel` (error, tenancy)
- `packages/platform` (config, health)
- `packages/security/redaction`
- `packages/data/turso`

---

## counter-service

**Path**: `services/counter-service/`

### Description
Demo counter service for testing increment/decrement/reset operations.

### Domain Entities
- Counter

### Ports
- `CounterRepository` — Counter persistence

### Events
- `CounterChanged` — Counter value changed

### Contracts
- `CounterResponse` — Counter value response

### Dependencies
- `packages/kernel` (ids, error, tenancy)
- `packages/platform` (config, health)
- `packages/contracts/api`
- `packages/data/turso`

---

## agent-service

**Path**: `services/agent-service/`

### Description
Manages AI agent conversations, messages, and tool calls.

### Domain Entities
- Conversation
- Message
- ToolCall

### Ports
- `ConversationRepository` — Conversation persistence
- `MessageRepository` — Message persistence
- `LLMProvider` — LLM API integration

### Events
- `ChatMessageSent` — Message sent in conversation

### Contracts
- `ConversationSummary` — Conversation metadata
- `ConversationDetail` — Conversation with messages
- `CreateConversationRequest` — New conversation request
- `ChatRequest` — Chat message request

### Dependencies
- `packages/kernel` (ids, error, tenancy, time)
- `packages/platform` (config, health)
- `packages/contracts/api`
- `packages/data/turso`
- `packages/security/redaction` (API key redaction)

---

## chat-service

**Path**: `services/chat-service/`

### Description
Real-time chat with conversation management and SSE streaming.

### Domain Entities
- Conversation
- Message
- Participant

### Ports
- `ChatRepository` — Chat data persistence
- `EventPublisher` — Chat event publishing

### Events
- `MessageSent` — Message sent
- `ConversationCreated` — New conversation created

### Contracts
- `CreateConversationRequest` — New conversation
- `ConversationSummary` — Conversation list item
- `MessageDto` — Message data transfer object
- `SendMessageRequest` — Send message request

### Dependencies
- `packages/kernel` (ids, error, tenancy, time)
- `packages/platform` (config, health)
- `packages/data/turso`
- `packages/messaging/nats`

---

## admin-service

**Path**: `services/admin-service/`

### Description
Admin dashboard statistics, tenant management, and system metrics.

### Domain Entities
- DashboardStats
- SystemMetrics

### Ports
- `StatsRepository` — Statistics aggregation
- `TenantRepository` — Tenant listing

### Contracts
- `AdminDashboardStats` — Dashboard statistics

### Dependencies
- `packages/kernel` (ids, error, time)
- `packages/platform` (config, health, service_meta)
- `packages/data/turso`

---

## indexing-service

**Path**: `services/indexing-service/`

### Description
Data indexing and search preparation.

### Domain Entities
- IndexEntry
- Document

### Ports
- `IndexRepository` — Index persistence
- `EventConsumer` — Event consumption for indexing

### Dependencies
- `packages/kernel` (ids, error, tenancy)
- `packages/platform` (config, health)
- `packages/data/turso`
- `packages/messaging/nats`

---

## event-bus

**Path**: `services/event-bus/`

### Description
Cross-service event bus for in-process event publishing.

### Ports
- `EventBus` — Publish/subscribe interface

### Types
- `EventEnvelope` — Event wrapper with metadata
- `EventId` — UUID v7 event identifier
- `AppEvent` — Unified event enum

### Dependencies
- `packages/kernel` (ids)
- `packages/contracts/events`
- `packages/runtime/ports/pubsub`
