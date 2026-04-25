# ADR-003: Runtime Abstraction (Direct + Dapr)

## Status
- [x] Proposed
- [ ] Accepted
- [x] Deprecated
- [ ] Superseded

> Historical note: this ADR records an earlier runtime-first direction.
> It is **not** the current default backend path.
> In current practice, services interact through their own ports, and async delivery is centered on
> `event_outbox -> outbox-relay -> projector`.
> The originally described direct + dapr runtime matrix was never fully implemented.

## Context

The original goal was to support multiple deployment topologies:

1. local development with mostly in-process execution
2. single-node deployments with optional messaging
3. cluster deployments with more explicit runtime boundaries
4. a possible future Dapr sidecar path

Without some form of runtime abstraction, topology switching looked likely to leak into business code.

## Decision

We originally proposed a runtime abstraction layer in `packages/runtime/`.

Only part of that proposal survived into code:

1. runtime ports are present
2. memory adapters exist for multiple ports
3. a partial NATS pubsub adapter exists

The following parts described in the original direction did **not** land as the default backend path:

1. direct adapter family
2. dapr adapter family
3. a shared policy engine for retry/backpressure/circuit breaking

### What exists today

- `packages/runtime/ports/` defines runtime-oriented traits
- `packages/runtime/adapters/memory/` contains in-memory implementations for several ports
- `packages/runtime/adapters/nats/` contains a NATS pubsub adapter fragment

### What does not exist today

- `direct/` adapter family
- `dapr/` adapter family
- `packages/runtime/policy/` modules for timeout/retry/idempotency/backpressure/circuit breakers
- a repository-wide rule that services must depend on runtime ports first

### Current interpretation

`packages/runtime` should now be read as a partial infrastructure package, not as the canonical backend foundation.

The current canonical backend foundation is instead:

1. service-local ports in `services/*`
2. shared contracts in `packages/contracts/**`
3. unified outbox in `packages/messaging/**`
4. explicit server/worker composition

## Consequences

### What becomes easier

- keeping historical intent visible
- reusing the small subset of runtime code that is still useful

### What becomes more difficult

- agents must not mistake this ADR for the current backend architecture
- some runtime modules may need further narrowing or de-emphasis later

### Trade-offs

- **Pros**: historical context, partial reusable code
- **Cons**: easy to misread unless clearly marked deprecated

### Implementation Status

- ✅ Runtime ports exist
- ✅ Memory adapters exist for multiple ports
- ✅ Partial NATS pubsub adapter exists
- ❌ Direct adapters not implemented
- ❌ Dapr adapters not implemented
- ❌ Full policy engine not implemented

## References

- `packages/runtime/ports/` - runtime port definitions
- `packages/runtime/adapters/` - current adapter set
- `docs/adr/009-canonical-monolith-first-topology-late-backend.md`
