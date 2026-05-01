# Counter-Service Reference Chain

This document describes the current default backend reference chain. It is intentionally short: detailed ownership lives in code, contracts, models, gates, and runbooks.

## Purpose

`counter-service` is the default backend reference anchor because it exercises the smallest useful business capability while touching the main engineering seams:

```text
service library
  -> shared contracts
  -> web-bff composition root
  -> CAS + idempotency intent + event_outbox
  -> outbox-relay worker
  -> projector worker
  -> replayable read model
```

It is not a toy demo. It is also not a fully proven production platform.

## Evidence Level

Current claim level:

1. service/library boundary: `tested`
2. contracts-first HTTP and event shape: `checked` to `tested`, depending on the path
3. CAS mutation and outbox write: `tested`
4. idempotency semantics: `declared` to `tested` for the happy path, not production-grade failure recovery
5. worker relay and projector structure: `checked` to `tested`, not multi-replica production-proven
6. secrets, overlay, Flux, and delivery wiring: `declared` to `checked`
7. independent `counter-service` deployable: `declared`, not the default runtime path

Do not describe this chain above its evidence level unless the stronger runtime and operational evidence has been added.

## Current Executable Chain

The current default path is:

1. `services/counter-service/model.yaml` declares service-local semantics.
2. `services/counter-service/src/**` implements the service library using domain, application, ports, and infrastructure layers.
3. `packages/contracts/**` carries external DTO, event, and error shapes.
4. `servers/bff/web-bff/src/handlers/counter.rs` exposes the synchronous HTTP entrypoint.
5. `services/counter-service/src/infrastructure/libsql_adapter.rs` provides the current libSQL persistence adapter.
6. `services/counter-service/migrations/001_create_counter.sql` creates current counter, idempotency, and outbox tables.
7. `workers/outbox-relay` reads `event_outbox` and publishes events.
8. `workers/projector` replays `event_outbox` and builds the counter read model.

The default composition root is still `web-bff`. The service crate remains a library.

## Declared Metadata

These files are navigation and control-plane declarations, not proof by themselves:

1. `services/counter-service/model.yaml`
2. `platform/model/services/counter-service.yaml`
3. `platform/model/deployables/web-bff.yaml`
4. `platform/model/deployables/outbox-relay-worker.yaml`
5. `platform/model/deployables/projector-worker.yaml`
6. `platform/model/state/ownership-map.yaml`

Use them to understand intent, then verify behavior through code, schemas, tests, validators, gates, scripts, or command output.

## Current Gaps

These are known gaps, not hidden current facts:

1. Idempotency currently lacks a durable request hash/status/result claim inside the same causal transaction. Treat it as a reference gap before production-grade retry semantics.
2. `event_outbox` is intended to be service-agnostic, but the current schema still lives in the counter-service migration path. Do not claim shared migration ownership until sources and gates agree.
3. `counter-service` has declared independent deployable metadata and secret templates, but the default runtime path still embeds it through `web-bff`.
4. `outbox-relay` and `projector` are held at `replicas=1` in the current reference profile. Multi-replica worker behavior requires durable ownership, checkpoint, dedupe, and recovery evidence.
5. GitOps, promotion, rollback, and drift handling have real entrypoints, but are not a fully proven release pipeline.

## Copy Rules For New Services

When adding a backend service, copy the pattern, not the current accidents:

1. Create `services/<name>/model.yaml` as the service-local declared semantics index.
2. Keep domain rules in `domain/`, orchestration in `application/`, external dependencies in `ports/`, and concrete local adapters in `infrastructure/` only when appropriate for the service boundary.
3. Put external DTOs, events, and error codes in `packages/contracts/**` before exposing them through HTTP, RPC, or messages.
4. Use a transactional mutation boundary for state change plus outbox write.
5. Keep broker/runtime publishing out of service libraries; publish through worker or composition-root paths.
6. Make projections replayable and rebuildable; never treat a read model as authoritative business state.
7. Mark stub, deferred, or reserved capabilities explicitly.
8. Run path-scoped guardrails first, then escalate gates based on contract, replay, delivery, topology, or P0 correctness risk.

## Verification Entry Points

Useful current commands include:

```bash
cargo check -p counter-service
cargo test -p counter-service
just check-backend-primary
just test-backend-primary
just verify-counter-delivery strict
```

Run only the gates relevant to the changed paths and risk level. Do not claim gates passed unless they were executed.

## One-Line Rule

`counter-service` is the default backend reference anchor: follow it for the current path, but keep declared metadata, target state, and proven runtime behavior separate.
