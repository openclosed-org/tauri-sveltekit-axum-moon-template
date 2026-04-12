# ADR-007: Workers-First Async Architecture

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context
The system needs to handle asynchronous tasks including:
- Event processing from message brokers
- Data indexing and transformation
- Read model projections
- Outbox pattern for reliable event publishing
- Scheduled jobs and cron-like tasks
- Synchronization and reconciliation
- Workflow execution

Traditional approaches often treat workers as secondary to HTTP services, leading to:
- Workers bolted onto existing services
- No clear ownership of async logic
- Shared state between sync and async paths
- Difficult scaling and monitoring

## Decision
We defined **workers as first-class citizens** in `workers/`:

### Worker Types
```
workers/
├── indexer/          # Event sourcing: read streams, transform, sink
├── outbox-relay/     # Reliable event publishing: poll outbox, publish
├── projector/        # Read model projections: consume events, build views
├── scheduler/        # Scheduled jobs: cron-like task execution
├── sync-reconciler/  # Synchronization: detect and resolve drift
└── workflow-runner/  # Workflow execution: step-by-step orchestration
```

### Worker Characteristics
- **Single responsibility**: Each worker has one core purpose
- **Independent deployment**: Can scale workers independently
- **Checkpoint/retry**: All workers track progress and recover from failures
- **Deduplication**: Idempotent processing of duplicate messages
- **Observability**: Metrics, logs, traces for monitoring

### Architecture
```
Event Source (NATS/Turso/etc.)
  └── Worker (poll/stream)
        └── Checkpoint (track progress)
        └── Dedupe (prevent duplicates)
        └── Process (business logic from services/*)
        └── Sink (write results)
```

### Integration with Services
- Workers consume `services/*` as libraries (same as servers)
- Workers use `packages/runtime/ports/` for messaging and state
- Workers implement their own ports for external integrations
- Services remain unaware of whether they're called by server or worker

### Reliability Patterns
- **Outbox pattern**: Turso outbox table for reliable event publishing
- **Inbox pattern**: Deduplication via tracking processed message IDs
- **Checkpoint pattern**: Persistent cursor for stream processing
- **Retry policies**: Exponential backoff with dead letter queues
- **Circuit breakers**: Prevent cascade failures

### Rationale
1. **Independent scaling**: Scale workers without scaling servers
2. **Clear ownership**: Async logic has a single home
3. **Reliability**: Checkpoint/retry/dedupe built-in
4. **Observability**: Workers are independently monitored
5. **Flexibility**: Different workers can use different strategies

## Consequences
### What becomes easier
- Scaling: Scale specific workers based on load
- Debugging: Isolate async issues to specific workers
- Reliability: Independent failure domains
- Monitoring: Worker-specific metrics and dashboards
- Development: Test workers independently

### What becomes more difficult
- Complexity: More deployment units to manage
- Coordination: Cross-worker transactions require careful design
- Testing: Need to test checkpoint/retry/dedupe behavior
- Operations: Monitor and alert on multiple workers

### Trade-offs
- **Pros**: Independence, reliability, observability, flexibility
- **Cons**: Operational complexity, coordination challenges

### Implementation Status
- ✅ Worker directory structure defined
- ✅ Indexer worker scaffolded with runtime ports
- ✅ Outbox relay worker scaffolded with runtime ports
- ✅ Runtime ports for pubsub, state, queue implemented
- ⏳ Full event processing implementations deferred
- ⏳ Production NATS integration deferred

## References
- `workers/` - Worker implementations
- `packages/runtime/ports/` - Runtime abstractions used by workers
- `platform/model/workflows/` - Workflow definitions
- `platform/model/deployables/` - Worker deployment units
- [Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
