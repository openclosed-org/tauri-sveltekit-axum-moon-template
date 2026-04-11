# Dependency Direction Rules

> **Status**: Draft — enforced by `just quality boundary`

## Allowed Dependencies

| Layer | May Depend On | Must NOT Depend On |
|-------|--------------|-------------------|
| `apps/*` | `packages/contracts/sdk`, `packages/ui` | `services/*`, `packages/adapters/*` |
| `apps/bff/*` | `services/*` (via trait), `packages/contracts` | `apps/*`, domain logic |
| `services/*` | `packages/core/*`, `packages/contracts` | Other `services/*`, `apps/*`, `axum`, `tokio` |
| `packages/adapters/*` | `packages/core/*`, external crates | `services/*`, `apps/*` |
| `packages/contracts` | `packages/core/kernel` | `services/*`, `apps/*` |
| `packages/core/*` | Nothing (or `packages/core/kernel`) | External crates except serde/thiserror/uuid/chrono |

## Forbidden Patterns

- ❌ `services/*` → other `services/*` (use `contracts/events`)
- ❌ `apps/*` → `services/*` (must go through BFF)
- ❌ `packages/core` → external crates (except allowed set)
- ❌ `packages/adapters` → business logic
- ❌ Direct `sqlx::query!` in services (use port traits)

## Verification

```bash
just quality boundary    # Dependency direction check
cargo hack check --workspace --feature-powerset  # Feature compatibility
```

## Circular Dependency Prevention

Dependency graph must be a DAG (Directed Acyclic Graph).
Any cycle = build failure + CI rejection.
