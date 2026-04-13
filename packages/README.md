# Packages

Shared layer — cross-platform reusable components.

## Layer Rules

1. **No composition logic** — packages define and implement contracts, they don't wire things together
2. **Dependency direction is strict** — see the table below
3. **Everything here is independently testable**
4. **No empty stubs** — only crates with real code or clear near-term purpose are kept

## Structure

```
packages/
├── kernel/         # Foundation types: TenantId, UserId, Cursor, AppError
├── platform/       # Platform capability traits (ConfigProvider, Clock, TelemetryProvider) — zero dependents, reserved
├── runtime/        # Runtime ports + memory adapters (Invocation, PubSub, State, Workflow, Lock, etc.)
├── contracts/      # Protocol truth: api, auth, events, errors
├── core/
│   ├── domain/     # Database port traits (LibSqlPort, SurrealDbPort) + TenantId re-export from kernel
│   └── workspace-hack/  # cargo-hakari dependency optimization
├── features/       # Feature traits: admin, agent, auth, chat, counter, settings
├── adapters/
│   ├── auth/google/
│   ├── hosts/tauri/
│   ├── storage/surrealdb/
│   ├── storage/turso/
│   └── telemetry/{otel,tracing}/
├── shared/
│   └── utils/      # ID generation, time formatting, crypto utilities
├── ui/             # Svelte component kit
└── sdk/            # Reserved (strategy in README.md)
```

## Dependency Direction

```
kernel/       ←  Foundation types (most stable)
    ↑
contracts/    ←  Protocol definitions (DTOs, events, errors)
    ↑
features/     ←  Feature traits, NO implementations
    ↑
runtime/      ←  Runtime ports (distributed system primitives)
    ↑
core/domain/  ←  Database port traits
    ↑
adapters/     ←  External protocol implementations
    ↑
services/     ←  Business logic implementations
    ↑
servers/      ←  Composition (HTTP routes, middleware)
    ↑
apps/         ←  Frontend / clients
```

**Violating this direction is a build failure** (enforced by `just quality boundary`).

## Cleaned Items

This directory was cleaned of 30+ empty/stub crates. See `LAYERING.md` for the full list of removed items and rationale.
