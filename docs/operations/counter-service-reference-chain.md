# Counter-Service Reference Chain

> **Purpose**: This is the definitive guide for agents and developers to understand the complete counter-service reference chain.
> **Status**: Production-grade reference module — copy this pattern for new services/workers.

---

## Overview

`counter-service` is the **reference origin** for backend development in this repository. It demonstrates:

1. Service-local domain semantics (increment/decrement/reset counter)
2. Contracts-first API design
3. CAS (Compare-And-Swap) versioning for optimistic concurrency
4. Idempotency for safe retries
5. Outbox pattern for event publication
6. Event relay to NATS/event bus
7. Projection into read models
8. BFF integration
9. Production-style deploy/config path

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        Client (Web/App)                       │
└────────────────────────┬─────────────────────────────────────┘
                         │ HTTP REST
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                     web-bff (Server)                          │
│  - DTO-first handlers (contracts_api::CounterResponse)       │
│  - Cache-first reads                                         │
│  - Forward to counter-service via embedded/remote backend    │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                   counter-service (Service)                   │
│                                                               │
│  Domain Layer:                                                │
│    - Counter aggregate (CounterId, Counter state)            │
│    - CounterDomainError (with CAS conflict semantics)        │
│                                                               │
│  Application Layer:                                           │
│    - CounterApplicationService                               │
│    - Idempotency checks (skip if same key already processed) │
│    - CAS version validation (fail on conflict)               │
│                                                               │
│  Infrastructure Layer:                                        │
│    - LibSqlCounterRepository<P: LibSqlPort>                  │
│    - SQL schema: migrations/001_create_counter.sql           │
│    - Outbox write: counter_outbox table                      │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                  counter_outbox (Event Store)                 │
│  - id (AUTOINCREMENT)                                        │
│  - event_type (e.g., "counter.changed")                      │
│  - payload (JSON serialized AppEvent)                        │
│  - source_service = "counter-service"                        │
│  - published (0 = pending, 1 = published)                    │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│               outbox-relay-worker (Worker)                    │
│                                                               │
│  Main Loop:                                                   │
│    1. LibSqlOutboxReader fetches pending entries             │
│    2. IdempotencyStore filters already-processed entries     │
│    3. OutboxPublisher publishes to event bus + pubsub        │
│    4. mark_published() updates outbox table                  │
│    5. CheckpointStore persists last processed ID to disk     │
│    6. MessageDedup prevents duplicate processing             │
│                                                               │
│  Resilience:                                                  │
│    - File-based checkpoint (crash recovery)                  │
│    - Idempotency store (exactly-once semantics)              │
│    - Dedup window (LRU eviction)                             │
└────────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                  Event Bus / NATS (Message)                   │
│  - Subject: counter.changed                                  │
│  - Payload: AppEvent::CounterChanged                         │
└────────────────────────┬─────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
┌──────────────────┐          ┌──────────────────┐
│ indexer-worker   │          │ projector-worker │
│                  │          │                  │
│ Pulls events →   │          │ Consumes events  │
│ Transform →      │          │ CounterState     │
│ Sink (search,    │          │ Consumer builds  │
│ analytics, etc.) │          │ read models      │
└──────────────────┘          └──────────────────┘
```

---

## Key Files

### Service Implementation

| File | Purpose |
|------|---------|
| `services/counter-service/model.yaml` | Domain model truth source (entities, events, operations) |
| `services/counter-service/src/domain/` | Domain entities and errors |
| `services/counter-service/src/application/` | Application service (orchestrates domain + outbox) |
| `services/counter-service/src/infrastructure/libsql_adapter.rs` | Repository implementation with SQL |
| `services/counter-service/migrations/001_create_counter.sql` | SQL schema truth source |

### Contracts

| File | Purpose |
|------|---------|
| `packages/contracts/api/src/lib.rs` | DTOs: `CounterResponse`, `CounterError` |
| `packages/contracts/events/src/lib.rs` | Events: `AppEvent::CounterChanged` |
| `packages/contracts/errors/src/lib.rs` | Shared error types |

### BFF Integration

| File | Purpose |
|------|---------|
| `servers/bff/web-bff/src/main.rs` | BFF composition root |
| `servers/bff/web-bff/src/handlers/counter.rs` | Counter handlers (DTO-first) |

### Workers

| File | Purpose |
|------|---------|
| `workers/outbox-relay/src/main.rs` | Outbox relay worker (libsql-backed, idempotent, checkpointed) |
| `workers/outbox-relay/src/polling/mod.rs` | `LibSqlOutboxReader`, `OutboxPoller` |
| `workers/outbox-relay/src/publish/mod.rs` | `OutboxPublisher` (event bus + pubsub) |
| `workers/outbox-relay/src/checkpoint/mod.rs` | File-based checkpoint store |
| `workers/outbox-relay/src/idempotency/mod.rs` | Idempotency store (exactly-once) |
| `workers/projector/src/consumers/mod.rs` | `CounterStateConsumer` (projects CounterChanged) |
| `workers/indexer/src/main.rs` | Indexer (handles CounterChanged in event routing) |

### Platform Model

| File | Purpose |
|------|---------|
| `platform/model/services/counter-service.yaml` | Platform-level metadata |
| `platform/model/state/ownership-map.yaml` | Service ownership (owner_service: counter-service) |

### Configuration & Deployment

| File | Purpose |
|------|---------|
| `infra/kubernetes/base/configmaps/outbox-relay-worker-config.yaml` | Worker config map |
| `infra/security/sops/templates/dev/outbox-relay-worker.yaml` | Dev secrets template |
| `infra/security/sops/dev/outbox-relay-worker.enc.yaml` | Encrypted dev secrets |
| `platform/model/deployables/outbox-relay-worker.yaml` | Deployable manifest |

---

## Configuration

### counter-service

Environment variables (injected via SOPS/Kustomize/Flux):

| Variable | Type | Description |
|----------|------|-------------|
| `SERVER_HOST` | ConfigMap | HTTP server host |
| `SERVER_PORT` | ConfigMap | HTTP server port |
| `DATABASE_URL` | SOPS Secret | Turso/SQLite database URL |
| `RUST_LOG` | ConfigMap | Logging configuration |

### outbox-relay-worker

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `OUTBOX_DATABASE_URL` | SOPS Secret | `file:/data/web-bff.db` | Outbox table source |
| `OUTBOX_NATS_URL` | SOPS Secret | `nats://localhost:4222` | NATS URL |
| `OUTBOX_NATS_SUBJECT_PREFIX` | ConfigMap | `counter` | Subject prefix |
| `OUTBOX_POLL_INTERVAL_MS` | ConfigMap | `500` | Poll frequency |
| `OUTBOX_BATCH_SIZE` | ConfigMap | `100` | Entries per batch |
| `OUTBOX_CHECKPOINT_PATH` | ConfigMap | `/data/outbox-relay-checkpoint.json` | Checkpoint persistence |
| `OUTBOX_HEALTH_HOST` | ConfigMap | `0.0.0.0` | Health server host |
| `OUTBOX_HEALTH_PORT` | ConfigMap | `3030` | Health server port |
| `OUTBOX_RUST_LOG` | ConfigMap | `info,outbox_relay_worker=debug` | Logging |

---

## Running Locally

### Without Cluster (Quick Inner Loop)

```bash
# counter-service
just sops-run counter-service

# outbox-relay-worker
just sops-run outbox-relay-worker

# web-bff
just sops-run web-bff
```

### With Cluster (K3s)

```bash
# Apply encrypted secrets to cluster
just sops-reconcile dev

# Deploy applications
just deploy-prod dev
```

---

## Event Flow: Step by Step

### 1. Client Increments Counter

```
POST /api/counters/default/increment
Body: { "idempotency_key": "req-123" }
```

### 2. BFF Handler

```rust
// servers/bff/web-bff/src/handlers/counter.rs
async fn increment(
    State(service): State<CounterService>,
    Json(req): Json<IncrementRequest>,
) -> Result<Json<CounterResponse>, AppError> {
    let counter = service.increment(req.counter_key, req.idempotency_key).await?;
    Ok(Json(CounterResponse::from(counter)))
}
```

### 3. Application Service

```rust
// services/counter-service/src/application/service.rs
pub async fn increment(
    &self,
    counter_key: &str,
    idempotency_key: Option<&str>,
) -> Result<Counter, CounterError> {
    // 1. Check idempotency (skip if already processed)
    if let Some(key) = idempotency_key {
        if let Some(cached) = self.check_idempotency(key).await? {
            return Ok(cached);
        }
    }

    // 2. Read current state with CAS version
    let counter = self.repo.get(counter_key).await?;
    let version = counter.version;

    // 3. Apply domain operation
    counter.increment()?;

    // 4. Save with CAS (fail if version mismatch)
    self.repo.save(&counter, version).await?;

    // 5. Write to outbox
    let event = AppEvent::CounterChanged(CounterChanged {
        tenant_id: counter.tenant_id,
        counter_key: counter.key,
        operation: "increment".to_string(),
        new_value: counter.value,
        delta: 1,
        version: counter.version,
    });
    self.repo.write_outbox(&event).await?;

    // 6. Cache idempotency result
    if let Some(key) = idempotency_key {
        self.cache_idempotency(key, &counter).await?;
    }

    Ok(counter)
}
```

### 4. Outbox Write

```sql
INSERT INTO counter_outbox (event_type, payload, source_service)
VALUES ('counter.changed', '{"CounterChanged": {...}}', 'counter-service');
```

### 5. Outbox Relay Worker Polls

```rust
// workers/outbox-relay/src/main.rs
loop {
    let entries = poller.poll_cycle().await;
    
    if !entries.is_empty() {
        // Filter duplicates via idempotency store
        let mut to_publish = Vec::new();
        for entry in &entries {
            if idempotency_store.start(&entry.id) {
                to_publish.push(entry.clone());
            }
        }
        
        // Publish to event bus + NATS
        let (successes, failures) = publisher.publish_batch(&to_publish).await;
        
        // Mark published in database
        if !successes.is_empty() {
            poller.mark_published(&successes).await?;
            for id in &successes {
                idempotency_store.complete(id);
            }
        }
        
        // Advance checkpoint (persisted to disk)
        poller.mark_processed(&entries);
    }
    
    tokio::time::sleep(config.poll_interval()).await;
}
```

### 6. Event Published

```
Subject: counter.changed
Payload: {
  "CounterChanged": {
    "tenant_id": "tenant-1",
    "counter_key": "default",
    "operation": "increment",
    "new_value": 42,
    "delta": 1,
    "version": 5
  }
}
```

### 7. Projector Consumes Event

```rust
// workers/projector/src/consumers/mod.rs
impl EventConsumer for CounterStateConsumer {
    fn is_interested(&self, event: &AppEvent) -> bool {
        matches!(event, AppEvent::CounterChanged(_))
    }

    async fn consume(&self, envelope: &EventEnvelope) -> Result<Option<String>, ProjectorError> {
        let counter_changed = match &envelope.event {
            AppEvent::CounterChanged(event) => event,
            _ => return Ok(None),
        };

        let update = CounterStateUpdate {
            tenant_id: counter_changed.tenant_id.clone(),
            counter_key: counter_changed.counter_key.clone(),
            new_value: counter_changed.new_value,
            version: counter_changed.version as u64,
            operation: counter_changed.operation.clone(),
            projected_at: chrono::Utc::now().to_rfc3339(),
        };

        // Update in-memory state
        self.state.write().unwrap().insert(key, update.clone());

        Ok(serde_json::to_string(&update)?)
    }
}
```

---

## Resilience Guarantees

### Idempotency

- **What**: Same `idempotency_key` produces same result without side effects
- **Where**: `CounterApplicationService::check_idempotency()`
- **Storage**: In-memory cache (upgrade to Redis/DB in production)

### CAS (Compare-And-Swap)

- **What**: Optimistic concurrency control via version field
- **Where**: `LibSqlCounterRepository::save(counter, expected_version)`
- **Failure**: Returns `CounterError::CasConflict` on version mismatch

### Outbox Pattern

- **What**: Event written to database in same transaction as state change
- **Where**: `LibSqlCounterRepository::write_outbox(event)`
- **Guarantee**: Event persistence even if relay is down

### Relay Idempotency

- **What**: Each outbox entry published exactly once
- **Where**: `IdempotencyStore` in outbox-relay-worker
- **Mechanism**: `start()` → `complete()` / `fail()` lifecycle

### Checkpoint Persistence

- **What**: Last processed outbox ID survives crashes
- **Where**: `CheckpointStore::advance(new_value)` writes JSON to disk
- **Recovery**: Loads from disk on worker restart

### Deduplication

- **What**: Duplicate outbox entries skipped
- **Where**: `MessageDedup::is_duplicate(entry_id)`
- **Eviction**: LRU-style (removes 25% of completed/failed when at capacity)

---

## Testing

### Unit Tests

```bash
cargo test -p counter-service
cargo test -p outbox-relay-worker
cargo test -p projector-worker
```

### Integration Tests

```bash
# Full stack test (counter-service with libsql adapter)
cargo test --test full_stack_test -p counter-service

# Outbox relay resilience tests
cargo test --test resilience_tests -p outbox-relay-worker
```

### Verification Gates

```bash
# All workspace checks
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
cargo test --workspace

# Validation scripts
bun run scripts/validate-state.ts --mode strict
bun run scripts/validate-contracts.ts
bun run scripts/validate-imports.ts
bun run scripts/boundary-check.ts
```

---

## Copying This Pattern

When creating a new service (e.g., `order-service`):

1. **Copy counter-service directory structure**
   ```
   cp -r services/counter-service services/order-service
   ```

2. **Update model.yaml**
   - Change service name to `order-service`
   - Define new entities (Order, OrderItem, etc.)
   - Define events (OrderCreated, OrderUpdated, etc.)

3. **Update platform/model/services/**
   - Create `order-service.yaml`
   - Update `ownership-map.yaml`

4. **Create contracts**
   - Add `OrderResponse` to `packages/contracts/api/`
   - Add `OrderCreated` event to `packages/contracts/events/`

5. **Implement service**
   - Domain layer: Order aggregate, OrderDomainError
   - Application layer: OrderApplicationService with idempotency
   - Infrastructure layer: LibSqlOrderRepository

6. **Add outbox**
   - Create `order_outbox` table
   - Write events in same transaction as state changes

7. **Update workers**
   - Add OrderStateConsumer to projector-worker
   - Update indexer-worker to handle OrderCreated event

8. **Configure deployment**
   - Create SOPS templates for order-service
   - Create Kubernetes configmaps
   - Update deployable manifests

---

## Common Pitfalls

1. **Don't skip idempotency** — Always check before applying side effects
2. **Don't ignore CAS conflicts** — Return error, don't silently overwrite
3. **Don't forget outbox writes** — Must be in same transaction as state change
4. **Don't use in-memory reader in production** — Use `LibSqlOutboxReader`
5. **Don't skip checkpoint persistence** — Crashes will lose progress
6. **Don't hardcode SQL strings** — Use migration files as truth source
7. **Don't bypass contracts DTOs** — Always use `contracts_api::CounterResponse`

---

## See Also

- [Backend Configuration Policy](../operations/backend-config-policy.md)
- [Counter-Service Gap Fix Plan](../counter-service-gap-fix-plan.md)
- [Service Agent Skill](../../.agents/skills/service-agent/SKILL.md)
- [Worker Agent Skill](../../.agents/skills/worker-agent/SKILL.md)
