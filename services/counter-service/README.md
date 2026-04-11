# services/counter-service

> Counter domain service — counting, statistics, analytics.

## Status
- [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
- [ ] Phase 1: Implement domain/application/ports
- [ ] Phase 2: Independent deployment

## Dependencies
- `packages/core/kernel` (TenantId, AppError)
- `packages/core/domain` (port traits)
- `packages/contracts/*` (HTTP/Event contracts)
- `packages/features/counter` (CounterService trait, Counter struct)

## Architecture
- `domain/` — Counter entity, value objects, invariants
- `application/` — Use cases (increment, decrement, reset)
- `ports/` — External dependency abstractions (CounterRepository)
- `contracts/` — Stable contract definitions
- `sync/` — OfflineFirst sync strategies
- `infrastructure/` — Database implementations
- `interfaces/` — API route handlers
