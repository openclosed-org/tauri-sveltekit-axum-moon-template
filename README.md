# Backend-in-Rust Reference Architecture

> Semantic-first, topology-late, agent-native backend architecture — with DDD, CAS, unified outbox, projection, workflow, and a multi-agent AI harness. Built entirely in Rust.
>
> Single VPS → K3s cluster → multi-service topology. Same code, different platform model.

**What this is:** A living reference architecture for building resilient Rust backends. Every service owns its state, every mutation uses CAS + idempotency, every event flows through a unified outbox, and every projection is rebuildable from source.

**What makes it different:** A [multi-agent AI harness](#-for-ai-agents) that routes tasks to domain-specialized subagents, enforces architectural boundaries, and gates every change — humans and AI co-develop under the same protocol.

---

## Highlights

- **DDD service structure** — `domain/` → `application/` → `ports/` → `infrastructure/`. Services are pure libraries; servers and workers compose them.
- **CAS + idempotency + unified outbox** — Every mutation uses optimistic concurrency control, every command declares an idempotency key, every event writes to a single `event_outbox` table shared across services.
- **Projection that rebuilds** — Read models are disposable. Checkpoint from `event_outbox`, replay on demand, lag-aware SLO.
- **Contracts-first** — API/Event/DTO/ErrorCode changes land in `packages/contracts/` first, with drift detection gates.
- **Platform model as truth source** — `platform/model/*` declares deployables, workflows, topologies, resources, environments. No implicit infrastructure.
- **Single VPS to K3s** — `platform/model/topologies/*.yaml` defines the deployment shape. Same binary, different topology.
- **SOPS + Flux GitOps** — Secrets via SOPS/Kustomize/Flux, not `.env` files. `just sops-run <deployable>` injects env vars locally; Flux reconciles in-cluster.
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

## Core Principles

1. **Platform model first** — `platform/model/*` is the truth source; infra and generated artifacts derive from it.
2. **Contracts before implementation** — `packages/contracts/*` changes first; contract drift is detected by gate.
3. **Services are libraries, not processes** — composed by servers and workers, never import each other directly.
4. **Workers are first-class** — every async executor must declare idempotency, retry, checkpoint/replay, and recovery.
5. **Vendor only in adapters** — concrete SDKs live in `packages/*/adapters/`, never in domain code.
6. **Generated directories are read-only** — `sdk/`, `rendered/`, `catalog/` must be regenerable.
7. **Topology changes shape, not semantics** — `platform/model/topologies/*.yaml` switches deployment without touching business logic.

## Why This Design Exists

The old engineering wisdom says: *"don't over-engineer — write simple code, refactor when needed."*

In 2026, with AI agents as persistent co-developers, this advice needs an update. Agents don't struggle with complexity — they struggle with **implicit, unstable architecture**. They thrive on stable conventions, executable gates, and explicit boundaries. What kills agent productivity is tribal knowledge, naming drift, inconsistent patterns, and rules that live in human memory instead of validators.

The new paradigm: **design the system to be semantically unfoldable from day one.** Don't prematurely split into microservices — that's just more moving parts. Instead, make every semantic boundary extractable and every deployment seam switchable, so that future growth becomes a **topology toggle**, not a **semantic rewrite**.

This is what **semantic-first, topology-late, agent-native** means in practice:

- **semantic-first** — Service-local semantics (`services/<name>/model.yaml`), contracts (`packages/contracts/`), outbox/projection/replay mechanisms, and validation scripts are designed upfront with explicit boundaries. The business logic layer is written once.
- **topology-late** — Deployment shape (single binary, multi-process, multi-node) is declared in `platform/model/topologies/` and can be switched without touching service code. The runtime topology unfolds when you need it, not before.
- **agent-native** — The repository is structured so that AI agents can continuously take over, verify, and evolve it. `AGENTS.md`, `agent/codemap.yml`, routing rules, gate matrix, scoped verification — these aren't extras. They're part of the architecture.

**What you gain:** Most future changes become topology switches and adapter migrations, not semantic rewrites. The system grows, the harness stays stable.

**What this doesn't promise:** Crossing from in-process to cross-network changes latency budgets, consistency models, failure recovery, permission boundaries, and observability. These are real problem-domain shifts, not code-generation problems. This architecture compresses future pain from "semantic rewrite" down to "topology + runtime strategy adjustment." That's already a massive win.

> *Agents don't hate big architecture. They hate unstable implicit architecture.*

## For AI Agents

This repository has a built-in multi-agent collaboration protocol:

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

## Project Status

This is an actively developed reference architecture. The backend core (DDD, CAS, outbox, projection, platform model) is functional and gated. The multi-agent harness is operational. Some workers (indexer, scheduler, sync-reconciler) and the frontend shells are stubs awaiting future development.

The default development path follows the `counter-service` reference chain. New backend capabilities should be compared against this anchor before extending new patterns.

## License

Apache 2.0. See `LICENSE`.

---

*Built with Rust, Axum, Tokio, NATS, libSQL/Turso, SOPS, Flux, and a multi-agent AI harness.*
