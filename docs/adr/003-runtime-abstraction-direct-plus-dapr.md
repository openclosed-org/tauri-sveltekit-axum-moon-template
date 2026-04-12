# ADR-003: Runtime Abstraction (Direct + Dapr)

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context
The system needs to support multiple deployment topologies:
1. **Local development**: Direct in-process calls, no external runtime
2. **Single VPS**: Direct calls with optional message broker
3. **K3s cluster**: Full microservices with distributed runtime
4. **Dapr sidecar**: Service mesh with Dapr runtime

Without a runtime abstraction layer, services would be tightly coupled to a specific runtime (direct calls OR Dapr OR message broker), making topology switching require code changes.

## Decision
We implemented a **dual-mode runtime abstraction** in `packages/runtime/`:

### Ports Layer (`packages/runtime/ports/`)
8 core runtime capabilities are abstracted as ports:
- `invocation.rs` - Service-to-service calls
- `pubsub.rs` - Publish/subscribe messaging
- `state.rs` - State management
- `workflow.rs` - Workflow orchestration
- `lock.rs` - Distributed locking
- `binding.rs` - Event bindings
- `secret.rs` - Secret management
- `queue.rs` - Queue operations

### Adapters Layer (`packages/runtime/adapters/`)
- `memory/` - In-memory implementations for testing and local dev
- `direct/` - Direct in-process calls for single-process deployment
- `dapr/` - Dapr sidecar adapter for Kubernetes deployment

### Policy Engine (`packages/runtime/policy/`)
- `timeout/` - Timeout policies
- `retry/` - Retry policies
- `idempotency/` - Idempotency guarantees
- `backpressure/` - Backpressure handling
- `circuit_breaker/` - Circuit breaker patterns

### Rationale
1. **Topology independence**: Switch deployment topology without changing service code
2. **Testability**: Memory adapters enable fast unit tests
3. **Progressive deployment**: Start with direct, move to Dapr when needed
4. **Policy consistency**: Retry, timeout, idempotency handled uniformly
5. **Vendor isolation**: Dapr-specific code isolated in one adapter

## Consequences
### What becomes easier
- Testing: Memory adapters for fast tests
- Local dev: Direct mode, no infrastructure needed
- Production scaling: Dapr adapter for distributed systems
- Policy management: Centralized retry/timeout/idempotency

### What becomes more difficult
- Abstraction complexity: 8 ports × 3 adapters = 24 implementations
- Feature parity: Memory adapter may not support all production features
- Debugging: Indirection makes tracing harder

### Trade-offs
- **Pros**: Topology independence, testability, progressive deployment
- **Cons**: Abstraction overhead, feature parity challenges

### Implementation Status
- ✅ All 8 runtime ports defined
- ✅ Memory adapters implemented for all 8 ports
- ⏳ Direct adapters deferred (can be implemented when needed)
- ⏳ Dapr adapters deferred (can be implemented when deploying to K8s)

## References
- `packages/runtime/ports/` - Runtime port definitions
- `packages/runtime/adapters/memory/` - Memory implementations
- `docs/refactoring/REFACTORING-ROADMAP.md` Phase 3 - Runtime implementation
