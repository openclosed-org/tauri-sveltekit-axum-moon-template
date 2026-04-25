# Event Bus — Shared Event Contracts And Bus Abstraction

> `packages/messaging` owns the shared EventBus abstraction and the canonical `event_outbox` schema surface.
> It does **not** define the canonical background relay loop; that lives in `workers/outbox-relay`.

## Architecture

```text
┌────────────────────────────────────────────┐
│  ports/         (EventBus trait)            │  ← Services depend on this
├────────────────────────────────────────────┤
│  adapters/      (InMemoryEventBus)         │  ← In-process
│                 (NatsEventBus)              │  ← Distributed
├────────────────────────────────────────────┤
│  outbox/        (event_outbox schema +     │  ← Unified outbox persistence surface
│                  OutboxEntry)               │
└────────────────────────────────────────────┘
```

## Key Design

- `event_outbox` is the **only** canonical event persistence table — no per-service private outbox tables
- Schema: `sequence INTEGER PRIMARY KEY AUTOINCREMENT` + `event_id TEXT UNIQUE` (UUID v7)
- `status` / `retry_count` / `published_at` track delivery state
- canonical relay path is `event_outbox -> outbox-relay worker -> event backbone -> consumers`
- current relay implementation also fans the canonical envelope into `runtime::PubSub`, but that compatibility path is not a second canonical outbox owner

## Ownership

- Schema definition: `src/outbox/outbox_entry.rs`
- Canonical relay worker: `workers/outbox-relay/`
- Event types: `packages/contracts/events/`

## Feature Flags

- `memory` (default) — in-memory event bus via tokio broadcast channels
- `nats` (future) — NATS JetStream implementation for production
