# services/event-bus

> Event bus abstraction — EventBus trait, Outbox pattern, inter-service communication.

## Status
- [ ] Phase 0: Stub — no implementation
- [ ] Phase 1: In-memory event bus (dev/test)
- [ ] Phase 2: NATS JetStream (production)

## Dependencies
- `packages/core/kernel` (AppError, domain events)
- `packages/contracts/events` (event schema definitions)

## Architecture
- `ports/` — EventBus trait (publish, subscribe, consume)
- `adapters/memory/` — Phase 1: in-memory implementation for dev/test
- `adapters/nats/` — Phase 2: NATS JetStream implementation
- `outbox/` — Outbox pattern for reliable delivery (write event to local DB + publish)

## Features
- `memory` — In-memory event bus (default, for dev/test)
- `nats` — NATS JetStream event bus (production)

## Design Principles
1. Services communicate through events, not direct API calls
2. All domain events go through the outbox for reliability
3. Event schema is defined in `packages/contracts/events`
4. EventBus is injected via ports — no direct dependency on NATS
