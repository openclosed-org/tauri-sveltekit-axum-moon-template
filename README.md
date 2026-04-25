# Backend-in-Rust Template and Reference Architecture

> Backend-first, semantic-first, topology-late, agent-native template architecture — with DDD, CAS, unified outbox, projection, workflow, and a multi-agent AI harness. Built entirely in Rust.
>
> Single VPS → K3s cluster → multi-service topology. Same code, different platform model.

**What this is:** A backend-first template and living reference architecture for building resilient Rust backends. Every service owns its state, every mutation uses CAS + idempotency, every event flows through a unified outbox, and every projection is rebuildable from source.

**What makes it different:** A [multi-agent AI harness](#-for-ai-agents) that routes tasks to domain-specialized subagents, enforces architectural boundaries, and gates every change — humans and AI co-develop under the same protocol.

## Reality Check

This repository is an exploration and learning template for the agent-native development era.

It does **not** yet have confidence from a long-running real production business system. Current confidence comes primarily from repository structure, focused local verification, architectural gates, and GitHub CI.

Use it as a strong starting point, not as proof that every pattern here is already production-proven for your workload. You should evaluate each pattern against your own latency budget, reliability target, compliance requirements, team maturity, and operational model.

---

## Highlights

- **DDD service structure** — `domain/` → `application/` → `ports/` → `infrastructure/`. Services are pure libraries; servers and workers compose them.
- **CAS + idempotency + unified outbox** — Every mutation uses optimistic concurrency control, every command declares an idempotency key, every event writes to a single `event_outbox` table shared across services.
- **Projection that rebuilds** — Read models are disposable. Checkpoint from `event_outbox`, replay on demand, lag-aware SLO.
- **Contracts-first** — API/Event/DTO/ErrorCode changes land in `packages/contracts/` first, with drift detection gates.
- **Platform model as truth source** — `platform/model/*` declares deployables, workflows, topologies, resources, environments. No implicit infrastructure.
- **Single VPS to K3s** — `platform/model/topologies/*.yaml` defines the deployment shape. Same binary, different topology.
- **SOPS + Flux GitOps** — Backend secrets flow through SOPS/Kustomize/Flux, not `.env` files. `just sops-run <deployable>` injects env vars locally; Flux reconciles in-cluster.
- **Multi-agent AI harness** — 8 specialized agents (planner, platform-ops, contract, service, server, worker, app-shell) with routing rules, scoped gates, and boundary checks.

## Architecture at a Glance

```text
request
  → web-bff (Axum)
  → service (DDD: domain + application + ports)
  → CAS mutation + idempotency check + outbox write (atomic)
  → event_outbox (unified, service-agnostic)
  → outbox-relay worker (poll → dedup → publish to NATS)
  → projector worker (consume → checkpoint → read model)
  → replay / rebuild path
```

Cross-cutting:

```text
platform model (YAML truth source)
  → deployables / ownership / topology / validators
  → SOPS templates → encrypted artifacts → Kustomize overlay
  → Flux GitOps reconciliation
  → gates (typecheck, boundary, drift, replay, resilience)
```

`counter-service` is the default reference anchor — not a toy demo, but the smallest business unit with the most complete engineering chain.

## Repository Map

```text
platform/          Truth source
  model/           Deployables, workflows, topologies, resources, environments
  schema/          JSON Schema for every model entity
  validators/      Cargo binaries: model-lint, dependency-graph, contract-drift, topology-check, security, observability
  generators/      Platform catalog generator

services/          Business capabilities (pure libraries, no process entry)
  counter-service/ Reference anchor — CAS, idempotency, outbox, projection
  tenant-service/  Multi-tenant, multi-entity, workflow, compensation

servers/           Sync request entrypoints
  bff/web-bff/     BFF — Axum, tenant context, cache invalidation
  gateway/         API gateway

workers/           Async executors
  outbox-relay/    Outbox → message backbone (NATS), dedup, checkpoint
  projector/       Event consumption → read model, replay, lag SLO
  indexer/         (stub)
  scheduler/       (stub)
  sync-reconciler/ (stub)

packages/          Shared abstractions
  contracts/       API/Event/DTO/ErrorCode truth source (contracts-first)
  kernel/          Core domain primitives
  messaging/       Unified outbox schema, EventBus, PubSub
  data-traits/     Storage port traits (SqlTransaction, LibSqlPort)
  data-adapters/   Turso, SurrealDB adapters
  observability/   OpenTelemetry + tracing adapters
  authn/           Authentication adapters (Google OAuth)
  authz/           Authorization (Cedar-like tuples)
  sdk/rust/        Generated Rust SDKs

infra/             Infrastructure declarations
  security/sops/   SOPS templates, encrypted artifacts, scripts
  k3s/             Kustomize base + overlays (dev, staging)
  gitops/flux/     Flux Kustomization per deployable
  local/           Local dev infra (NATS, Valkey, MinIO)

ops/               Operations
  runbooks/        Counter delivery, health checks, backup/restore
  observability/   OTel collector, Vector configs

agent/             Multi-agent AI harness rules
  codemap.yml      Module constraints, dependency rules, anti-patterns
  manifests/       Routing rules, gate matrix

apps/              Optional frontend shells (web, desktop, mobile)
```

## Who This Repo Is For

There are two primary user modes for this repository:

1. **Template users** — teams that want to click "Use this template" and start a backend project from a pre-structured foundation.
2. **Contributors / maintainers** — people who want to evolve the template itself, improve the architecture, and keep the repository publishable as an open-source project.

Template users should treat the repository release as the main contract and keep only the parts relevant to their project.

The current release strategy is one repository-level `0.1.x` line for the template as a whole. Cargo crate versions remain internal workspace metadata, not separate product release channels.

Contributors should treat the repository structure, gates, agent protocol, and documentation conventions as part of the design, not as optional extras.

A dedicated `just template-init` cleanup flow now exists as a conservative planning/dry-run entrypoint. It does **not** delete files yet; it previews which upstream-maintainer and open-source governance materials a derived project may want to remove after adopting the template.

## Quick Start

```bash
# 1. Install toolchain
just setup
just setup-deps
just doctor

# 2. Start local infra (NATS, Valkey, MinIO)
bash infra/local/scripts/bootstrap.sh up

# 3. Run the backend
just dev-api          # web-bff
just sops-run web-bff # with SOPS-decrypted secrets

# 4. Run all quality checks
just verify

# 5. Platform health
just validate-platform
just platform-doctor
```

```bash
just --list    # See all available commands
```

## Using This Template

If you adopt this repository via GitHub "Use this template", the shortest useful path is:

1. `README.md`
2. `docs/operations/local-dev.md`
3. `docs/operations/secret-management.md`
4. `docs/operations/counter-service-reference-chain.md`
5. `just template-init PROFILE=backend-core MODE=dry-run`

The current `template-init` flow is intentionally conservative. It is meant to help derived projects identify upstream-maintainer artifacts to review or remove; it is not a code-pruning tool.

## For AI Agents

This repository has a built-in multi-agent collaboration protocol.

- `AGENTS.md` — master protocol: reading order, truth source hierarchy, routing rules, global constraints
- `agent/codemap.yml` — module boundaries, dependency rules, anti-patterns, required files per entity type
- `agent/manifests/routing-rules.yml` — path → subagent dispatch map
- `agent/manifests/gate-matrix.yml` — subagent → gate mapping (typecheck, boundary, drift, replay, resilience)

Gates enforce architectural boundaries. Every agent dispatch ends with scoped verification:

```bash
bun run scripts/route-task.ts              # Route a task to the right subagent
bun run scripts/run-scoped-gates.ts <agent> # Run gates for a specific agent
just verify                                 # Total verification (final gate)
```

More repository policy and documentation-boundary details live in `docs/README.md`.

## Project Status

This is an actively developed backend-first template and reference architecture. The backend core (DDD, CAS, outbox, projection, platform model) is functional and gated. The multi-agent harness is operational. Some workers (indexer, scheduler, sync-reconciler) and the frontend shells are stubs awaiting future development.

The default development path follows the `counter-service` reference chain. New backend capabilities should be compared against this anchor before extending new patterns.

The template is already useful, but it should still be adopted with engineering judgment. CI green status means the repository passed its current automated checks; it does not mean every pattern has been validated under real business traffic, long-lived operational load, or your own production constraints.

## License

Apache 2.0. See `LICENSE`.

---

*Built with Rust, Axum, Tokio, NATS, libSQL/Turso, SOPS, Flux, and a multi-agent AI harness.*
