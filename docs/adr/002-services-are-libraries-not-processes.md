# ADR-002: Services Are Libraries, Not Processes

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context
In traditional microservice architectures, each "service" is typically a standalone process with its own HTTP server, message consumer, and deployment lifecycle. This creates tight coupling between business logic and runtime infrastructure, making it difficult to:
- Test business logic without starting infrastructure
- Reuse service logic across different deployment contexts
- Switch between direct calls and message-based communication
- Deploy in different topologies (monolith vs. microservices)

## Decision
We defined `services/*` as **pure business logic libraries**, not independent processes:

- Each `services/<name>` contains domain logic, application use cases, policies, ports, events, and contracts
- Services have NO `main.rs`, NO HTTP server, NO message consumer loop
- All external dependencies are abstracted through `ports/`
- Services can be tested in isolation with in-memory port implementations
- `servers/*` provides synchronous request entry (HTTP/BFF)
- `workers/*` provides asynchronous execution units
- Both `servers/*` and `workers/*` consume services as libraries

### Standard Service Structure
```
services/<name>/
├── src/
│   ├── domain/        # Entities, value objects, domain rules
│   ├── application/   # Use cases, command/query handlers
│   ├── policies/      # Business policies, validation rules
│   ├── ports/         # Abstract interfaces for external dependencies
│   ├── events/        # Domain event definitions
│   ├── contracts/     # Public API contracts
│   └── lib.rs         # Public API
├── tests/             # Unit and integration tests
├── migrations/        # Database migrations
└── Cargo.toml
```

### Rationale
1. **Testability**: Business logic can be tested without infrastructure
2. **Flexibility**: Same service can be used by BFF, worker, or CLI
3. **Deployment independence**: Topology changes don't require code changes
4. **Clear boundaries**: Domain logic is isolated from framework concerns
5. **Hexagonal architecture**: Ports define what service needs, adapters provide how

## Consequences
### What becomes easier
- Testing: `cargo test -p service-name` works without DB/network
- Reuse: Same service logic in web BFF, admin BFF, workers
- Migration: Switch from direct calls to message passing without changing service code
- Onboarding: Clear separation between business logic and infrastructure

### What becomes more difficult
- More boilerplate: Need ports, adapters, wiring code
- Indirection: New developers must understand the port/adapter pattern
- Initial complexity: Setting up the abstraction layer takes time

### Trade-offs
- **Pros**: Testability, flexibility, deployment independence, clear boundaries
- **Cons**: More boilerplate, indirection, learning curve

## References
- `agent/codemap.yml` Section 3.6 - Services rules
- `services/auth-service/` - Reference implementation
- `servers/web-bff/` - Example service consumer
- `workers/` - Example async service consumer
