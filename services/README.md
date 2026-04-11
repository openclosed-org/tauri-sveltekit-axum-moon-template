# Services

Future independent microservices. Each service is a **standalone Cargo workspace member** that can be extracted into its own repository with zero business logic modification.

## Structure

```
services/
├── user-service/         # User domain: authentication, profiles, permissions
├── agent-service/        # Agent domain: configuration, task execution, results
├── chat-service/         # Chat domain: messages, sessions, real-time push
├── counter-service/      # Counter domain: counting, statistics, analytics
└── event-bus/            # Shared event bus: async-nats adapter + Outbox pattern
```

## Service Architecture

Each service follows **Clean Architecture** with four layers:

```
service/
├── src/
│   ├── domain/           # 【Pure Rust】Entities, value objects, events
│   ├── application/      # 【Pure Rust】UseCases · depends on trait abstractions
│   ├── interfaces/       # 【Axum/gRPC】HTTP controllers + DTOs
│   └── infrastructure/   # 【sqlx/redis-rs】DB/Cache implementations
├── Cargo.toml            # Independent crate · can build/deploy alone
└── openapi.yaml          # Contract source · CI generates SDKs
```

## Current Status

⚠️ **All services are stubs.** Business logic currently lives in:
- `packages/core/usecases/` — Application logic
- `servers/api/` — HTTP composition

Services will become independent when the **Phase 1 trigger** is met:
> Service change frequency > 3/week OR team > 5 people

## Evolution Path

| Phase | Location | Deployment |
|-------|----------|------------|
| **Phase 0** (Current) | `packages/core/usecases/` | Part of monolith binary |
| **Phase 1** | `services/` (independent `cargo build -p xxx`) | Same binary, independent build |
| **Phase 2** | `services/` (separate binaries) | Independent Docker images + K8s Deployments |

## Extracting a Service

To extract `user-service` into its own repo:

1. Copy `services/user-service/` directory
2. Change `Cargo.toml` workspace path → standalone crate
3. Update deploy config (Dockerfile / K8s manifest)
4. **Zero modification** to `domain/`, `application/`, `interfaces/` code

See [GOAL.md](../docs/GOAL.md) §3 for detailed acceptance criteria.
