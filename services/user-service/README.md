# services/user-service

> User domain service — authentication, profiles, permissions, sessions.

## Status
- [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
- [ ] Phase 1: Implement domain/application/ports
- [ ] Phase 2: Independent deployment (separate binary + Docker image)

## Dependencies
- `packages/core/kernel` (TenantId, UserId, AppError)
- `packages/core/domain` (port traits)
- `packages/contracts/*` (HTTP/Event contracts)
- `packages/features/auth` (AuthService trait)

## Architecture
- `domain/` — Pure domain logic (User entity, value objects, invariants)
- `application/` — Use cases (register, login, update_profile)
- `ports/` — External dependency abstractions (UserRepository, SessionStore)
- `contracts/` — Stable contract definitions (DTOs, events)
- `sync/` — OfflineFirst sync strategies
- `infrastructure/` — External service integrations
- `interfaces/` — API route handlers
