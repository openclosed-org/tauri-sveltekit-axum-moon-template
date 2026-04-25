# Docs Index

> Entry point for `docs/`. For agent coordination rules, see `AGENTS.md`.
> For machine-readable directory boundaries, see `agent/codemap.yml`.

## Project Status

This is an **experimental architecture template** — a semantic-first, topology-late, agent-native backend design for Axum. Functional but not production-hardened. Before deploying to production, address:

1. **Documentation drift**: some docs are in Chinese, some in English, some describe target-state rather than current-state. Always verify against code.
2. **Generated artifacts**: `deployables.generated.yaml` and `architecture.generated.md` may drift after structural changes. Run `just commit-golden-baseline` to resync.
3. **Secrets**: SOPS templates use `minioadmin` defaults. Production must use proper secret injection (SOPS/Kustomize/Flux), not `.env` files.
4. **Stubs**: `auth-service`, `indexing-service`, `scheduler-worker`, `sync-reconciler-worker` are stubs — not production-ready.
5. **Release flow**: release preparation is now handled by `release-plz` for the Rust workspace; prefer conventional commits for readable changelogs.

## Default Reading Order

1. `docs/operations/counter-service-reference-chain.md` — primary backend reference
2. Task-specific: `services/*/README.md`, `workers/*/README.md`, `ops/runbooks/**`
3. `docs/adr/**` — architecture decision records

When docs conflict with code, trust code, schemas, validators, and gates.

## Default Backend Anchor

`counter-service` is the default backend reference anchor — not a minimal demo. It carries two chains:

1. **Business main chain**: `service → contracts → server → outbox → relay → projector`
2. **Engineering cross-cutting chain**: `platform model → secrets → deploy → GitOps → promotion → drift → runbook`

New backend capabilities should align with `counter-service` first. Detailed status → `docs/operations/counter-service-reference-chain.md`.

Current validation policy follows the same split:

1. Default backend admission lane: `just verify-backend-primary` and `just test-backend-primary`
2. Optional auth lane: `just verify-auth-optional` and `just test-auth-optional`
3. Secondary/governance workflow: `.github/workflows/quality-gate.yml` covers secondary backend and governance validation, not the default business-chain admission lane

## Documentation Strategy

Two types of docs enter the main context:

- **Class A**: repo-level rules, boundaries, source of truth — `AGENTS.md`, `agent/codemap.yml`, `agent/manifests/routing-rules.yml`, `agent/manifests/gate-matrix.yml`
- **Class B**: reference chains, owner docs, runbooks — `docs/operations/`, `services/*/README.md`, `ops/runbooks/`

Before deleting/archiving docs, confirm their toolchain info has entered Class A or B.

## Current Refactor Guidance

For the current architecture convergence work, read these two documents after the default backend anchor docs:

1. `docs/architecture/refactor-backlog-monolith-first-topology-late.md` — executable refactor backlog
2. `docs/adr/009-canonical-monolith-first-topology-late-backend.md` — canonical architecture statement

## Helper Scripts

These scripts support agent routing, scoped gates, and handoff verification:

| Script | Purpose |
|--------|---------|
| `scripts/route-task.ts` | Route tasks to subagents |
| `scripts/run-scoped-gates.ts <subagent>` | Run scoped verification gates |
| `scripts/verify-handoff.ts <subagent>` | Verify handoff between agents |

For all available commands: `just --list`

## Future Direction: Macro-Based Boilerplate Reduction

A key area for future improvement — reducing per-service boilerplate through declarative macros:

1. **Derive macros for CAS + idempotency + outbox**: auto-generate `commit_mutation` wrappers from `model.yaml` declarations.
2. **Handler generation from contracts**: auto-generate Axum stubs from `packages/contracts` API definitions.
3. **Worker scaffolding macros**: generate retry, checkpoint, dedup, and metrics from `model.yaml`.
4. **Platform model codegen**: emit Rust types and drift checks from `platform/schema`.

This is deliberately deferred — the semantic model must be validated by real usage before encoding it in macros.
