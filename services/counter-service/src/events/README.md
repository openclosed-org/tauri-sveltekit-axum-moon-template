# Counter Service — Events

> Event definitions for the counter domain.
>
> **Single Source of Truth**: Event schemas are defined in `packages/contracts/events/`.
> This module re-exports them for local discoverability.

## Published Events

| Event | Schema | Status |
|-----|--------|--------|
| CounterChanged | `packages/contracts/events/src/lib.rs::CounterChanged` | ✅ Implemented |

## Event: `counter.changed`

Emitted after any successful counter mutation (increment, decrement, or reset).

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `tenant_id` | `String` | Tenant scope identifier |
| `counter_key` | `String` | Counter identifier within the tenant |
| `operation` | `String` | Operation type: `increment`, `decrement`, `reset` |
| `new_value` | `i64` | Counter value after the operation |
| `delta` | `i64` | Change amount (positive for increment, negative for decrement/reset) |
| `version` | `i64` | CAS version number after the operation |

### Dedupe Rule

`tenant_id + counter_key + version`

### Ordering Scope

per-tenant

### Replay

依据 `services/counter-service/model.yaml`，当前声明语义为：

- replayable: `true`
- retention: `P30D`
- compatibility_policy: `backward`

这些字段目前以 service semantics 声明为准，不应被额外放大成“所有下游保留与 broker 策略都已完全落地”。
