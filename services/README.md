# Services

> `services/` contains business capability and state-boundary libraries.
> In the current harness, service directories fall into three classes:
> `reference`, `stub`, and `deprecated`.

## Current Inventory

| Directory | Class | Purpose |
|---|---|---|
| `counter-service/` | reference | Smallest complete command/query/event sample |
| `tenant-service/` | reference-secondary | Multi-entity, workflow-driven semantics sample |
| `auth-service/` | stub | Planned auth capability placeholder |
| `user-service/` | stub | Planned identity/profile capability placeholder |
| `indexing-service/` | stub | Planned indexing/search capability placeholder |

## Rules

1. Every service directory must carry a `model.yaml` that explains its current semantic status.
2. `counter-service` is the only default copy target for new business services.
3. `tenant-service` may be consulted as a secondary semantics reference when a feature truly needs multi-entity, workflow, or compensation semantics.
3. Stub services may keep minimal or legacy code, but they are not reference modules and should not drive new design decisions.

## Reference Skeleton

Reference services should converge on this shape:

```text
services/<name>/
├── model.yaml
├── Cargo.toml
├── src/
│   ├── domain/
│   ├── application/
│   ├── policies/
│   ├── ports/
│   ├── events/
│   ├── contracts/
│   └── lib.rs
├── tests/
├── migrations/
└── README.md
```

Concrete adapter modules can remain temporarily while existing servers/apps/workers still depend on them, but they are legacy outer-edge code, not the target template.
