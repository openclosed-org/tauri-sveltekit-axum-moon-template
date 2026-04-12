# Verification Suite

Cross-module testing infrastructure. Organized by test category:

| Directory | Purpose |
|-----------|---------|
| `e2e/` | End-to-end tests (Playwright, full-stack) |
| `contract/` | Contract compatibility, SDK roundtrip, event schema validation |
| `topology/` | Deployment topology verification |
| `resilience/` | Retry, idempotency, outbox, failover tests |
| `performance/` | Performance benchmarks (BFF, gateway, cache) |
| `golden/` | Golden baseline for generated artifacts |

## Running Tests

```bash
# All E2E tests
just test-e2e-full

# Contract tests
bun run verification/contract/backward-compat/contract.test.ts
bun run verification/contract/event-schema/event.test.ts

# Resilience tests (Rust, require full server + workers)
cargo test --test resilience_ -- --ignored

# Full verification
just verify
```

## Adding New Tests

1. Create a subdirectory under the appropriate category
2. Add test files (`.test.ts` for Bun/Playwright, `_test.rs` for Rust)
3. Document the test scenarios in a `README.md`
4. Add run instructions to this file

## Test Categories

### E2E Tests
- `demo-counter/` — Counter increment/decrement/reset flows
- `multi-tenant/` — Tenant isolation and onboarding
- `settings/` — User-scoped settings CRUD
- `desktop-web-roundtrip/` — Tauri vs Web parity

### Contract Tests
- `backward-compat/` — No breaking changes in DTOs
- `sdk-roundtrip/` — Generated types are usable
- `event-schema/` — Event schemas are well-formed

### Topology Tests
- `single-vps/` — All-in-one deployment
- `split-workers/` — Workers as separate processes

### Resilience Tests
- `retry/` — Transient failure handling
- `idempotency/` — Duplicate message safety
- `outbox/` — At-least-once delivery guarantees
- `failover/` — Component failure isolation

### Performance Tests
- `bff/` — BFF response latency
- `gateway/` — Gateway throughput
- `cache/` — Cache hit rates and staleness
