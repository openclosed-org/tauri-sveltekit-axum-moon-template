# ADR-007: Workers-First Async Architecture

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

> Implementation status: `outbox-relay` and `projector` are the only real async reference workers.
> Their current reliability semantics are still partial: checkpointing is local-file-based, and shared
> multi-replica dedupe / idempotency semantics are not yet the default runtime path.
> `indexer`, `scheduler`, and `sync-reconciler` remain mostly stub-level.

## Context

The repository needs explicit ownership for asynchronous execution, including:

- outbox publication
- projection and replay
- indexing and transformation
- scheduling and reconciliation
- future workflow execution

Treating async execution as an afterthought inside HTTP services makes scaling, recovery, and debugging much harder.

## Decision

We treat `workers/*` as the home for asynchronous execution units.

### Current real worker reference set

```text
workers/
  outbox-relay/   Poll event_outbox, publish, checkpoint
  projector/      Replay/apply events into read models
```

Other worker directories may exist, but they are not yet equal to the default async reference path.

### Worker characteristics

- single responsibility per worker
- explicit progress tracking
- explicit replay/recovery assumptions
- explicit health/readiness surface
- business logic consumed from `services/*` libraries where appropriate

### Current reliability interpretation

The repository wants worker reliability to include:

1. checkpointing
2. dedupe/idempotency
3. retry/recovery ordering
4. replay-safe processing

But the current implementation is only partially there:

1. checkpointing exists, but is still local-file-based in the main reference workers
2. shared dedupe/idempotency semantics are not yet standardized across workers
3. durable broker checkpoint semantics are not yet the default path

### Current integration rule

- workers consume `services/*` as libraries, just like servers do
- workers may use shared infrastructure packages, but `packages/runtime` is not the canonical backend foundation
- services should remain unaware of whether they are invoked from a server or a worker

## Consequences

### What becomes easier

- async execution has an explicit home
- async constraints are easier to reason about
- worker-specific health and recovery can evolve separately from HTTP servers
- replayable read paths can be modeled explicitly

### What becomes more difficult

- more deployment units to reason about
- cross-worker coordination still needs careful design
- testing reliability semantics remains non-trivial

### Trade-offs

- **Pros**: explicit async ownership, clearer recovery boundaries, better long-term topology fit
- **Cons**: operational complexity, current reliability gaps still need explicit closure

### Implementation Status

- ✅ Worker directory structure exists
- ✅ `outbox-relay` has a real database-backed outbox reader and NATS publish path
- ✅ `projector` has replay + read model + optional live tail path
- ⏳ Shared checkpoint/dedupe/idempotency semantics are deferred
- ⏳ Stub workers remain to be implemented or intentionally narrowed

## References

- `workers/outbox-relay/` - current relay reference worker
- `workers/projector/` - current projection reference worker
- `platform/model/deployables/` - worker deployment units
- `docs/adr/009-canonical-monolith-first-topology-late-backend.md`
- [Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
