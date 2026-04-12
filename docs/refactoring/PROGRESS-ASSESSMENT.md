# Refactoring Progress Assessment & Phased Roadmap

> **Assessment Date**: 2026-04-12  
> **Architecture Constitution**: `docs/ARCHITECTURE.md` (99% authoritative)  
> **Current State**: ~35% scaffolded, ~15% functionally implemented, ~50% stub/placeholder

---

## Executive Summary

The project has successfully established the **complete directory structure** and **scaffolding** matching the ARCHITECTURE.md target state. However, actual functional implementation is concentrated in a few areas:

### What Works Today
- ✅ Tauri desktop shell (window management, tray, Turso DB, OAuth, commands)
- ✅ SvelteKit web app (5 routes, counter demo feature working)
- ✅ Axum API server (config, middleware, routes, OpenAPI)
- ✅ Counter service (most complete - reference "golden module")
- ✅ Platform model (35 YAML files defining services, deployables, resources, topologies)
- ✅ Platform generator (catalog generation works)
- ✅ Agent constraints/checklists/templates (populated)
- ✅ CI workflows defined (but with stale paths)

### What's Stub/Placeholder
- 🔶 All other services (user, tenant, agent, settings, chat, admin) - Clean Architecture skeletons present, sync layers stubbed
- 🔶 All workers (indexer, outbox-relay, projector, scheduler, sync-reconciler) - log messages only
- 🔶 Storage adapters (Turso wired, SurrealDB/SQLite optional)
- 🔶 Auth adapters (Google OAuth scaffolding, others skeleton crates)
- 🔶 All chain/protocol adapters (Base, EVM, Solana, TON, AT-Proto, Farcaster, Nostr) - skeleton crates
- 🔶 Platform validators (all 6 empty)
- 🔶 Platform generators (only catalog generator works; compose, SDK, kustomize, flux, contracts, docs generators empty)
- 🔶 Verification E2E tests (README stubs only)
- 🔶 Documentation (most architecture diagram directories empty)

### What's Broken
- ❌ CI workflows reference stale paths (`apps/client/` should be `apps/`)
- ❌ Bun workspace only has 3 packages (web, browser-extension stub, ui)
- ❌ Missing root-level files: `typos.toml`, `.editorconfig`, `.cargo/audit.toml`, `.config/nextest.toml`
- ❌ `packages/` structure deviates from ARCHITECTURE.md (uses `packages/core/`, `packages/shared/`, `packages/adapters/`, `packages/features/` instead of `packages/kernel/`, `platform/`, `contracts/`, `sdk/`, `runtime/`, etc.)

---

## Detailed Progress by Directory

### ✅ Complete or Near-Complete (80%+)

| Directory | Progress | Notes |
|-----------|----------|-------|
| `agent/` | 90% | Constraints, checklists, prompts, templates populated. Missing: `templates/service/`, `templates/worker/`, `templates/bff/`, `templates/contract/`, `templates/platform-model/` |
| `platform/model/` | 95% | All 7 subdirectories populated with YAML. Missing: `authn-zitadel.yaml`, `authz-openfga.yaml`, `object-storage.yaml`, `secrets.yaml`, `wasm-runtime.yaml` in resources |
| `platform/schema/` | 100% | All 6 schemas present |
| `platform/catalog/` | 100% | Generated output complete |
| `.github/workflows/` | 80% | 5 workflows defined, but paths need fixing |
| `apps/desktop/src-tauri/` | 85% | Functional Tauri app with commands, DB, OAuth. Missing: updater plugin, full E2E |
| `apps/web/` | 70% | 5 routes working, missing: feed, notifications, payments, profile, social-graph, wallet pages |
| `services/counter-service/` | 85% | Reference golden module. Sync is stubbed |
| `servers/api/` | 75% | Functional Axum server with middleware, routes, OpenAPI |

### 🔶 Partially Implemented (30-70%)

| Directory | Progress | Notes |
|-----------|----------|-------|
| `services/user-service/` | 60% | Clean Architecture skeleton complete, sync stubbed |
| `services/tenant-service/` | 60% | Clean Architecture skeleton complete, sync stubbed |
| `services/agent-service/` | 60% | Clean Architecture skeleton complete, sync stubbed |
| `services/settings-service/` | 60% | Clean Architecture skeleton complete, sync stubbed |
| `services/chat-service/` | 50% | Skeleton present, marked as "stub" in platform model |
| `services/admin-service/` | 50% | Skeleton present, migrations are placeholder SQL |
| `services/event-bus/` | 50% | Skeleton present |
| `servers/bff/web-bff/` | 50% | Scaffolded but not fully wired to services |
| `servers/bff/admin-bff/` | 50% | Scaffolded but not fully wired to services |
| `servers/gateway/` | 40% | Edge gateway scaffolded |
| `servers/indexer/` | 40% | Indexer server scaffolded |
| `workers/*` (all 5) | 30% | All have main.rs with stub implementations logging messages |
| `packages/contracts/*` | 50% | 4 contract crates (api, auth, events, errors) present but content needs review against ARCHITECTURE.md structure |
| `verification/` | 40% | Contract tests (3 .ts), resilience tests (4 .rs), topology test present. E2E, performance, golden mostly empty |
| `docs/` | 30% | ARCHITECTURE.md and repo-layout.md present. ADRs, architecture diagrams, contracts docs empty |

### ❌ Not Started or Minimal (<30%)

| Directory | Progress | Notes |
|-----------|----------|-------|
| `packages/kernel/` | 0% | Should contain: ids, error, money, pagination, tenancy, time. Currently using `packages/core/kernel/` instead |
| `packages/platform/` | 0% | Should contain: config, health, buildinfo, env, release, service_meta. Structure deviates |
| `packages/runtime/` | 0% | Critical missing: ports/, policy/, adapters/ (direct, dapr, memory) |
| `packages/sdk/` | 0% | TypeScript/Rust SDK generation not implemented |
| `packages/authn/` | 0% | Should contain: oidc, pkce, session, token |
| `packages/authz/` | 0% | Should contain: model, ports, caching, decision, adapters/openfga |
| `packages/data/` | 0% | Should contain: turso, sqlite, migration, outbox, inbox, common-sql |
| `packages/messaging/` | 0% | Should contain: nats, envelope, codec |
| `packages/cache/` | 0% | Should contain: api, policies, adapters (moka, valkey, dragonfly) |
| `packages/storage/` | 0% | Should contain: api, paths, policies, adapters (s3, minio, localfs) |
| `packages/observability/` | 0% | Should contain: tracing, metrics, logging, baggage, otel |
| `packages/security/` | 0% | Should contain: crypto, signing, redaction, pii |
| `packages/web3/` | 10% | README present, traits and protocol adapters not implemented |
| `packages/wasm/` | 0% | Should contain: wit, host, guest-sdk, components |
| `packages/ui/` | 40% | Present in bun workspace, content needs review |
| `packages/devx/` | 0% | Should contain: testkit, fixture-loader, contract-test, perf-harness, snapshot |
| `infra/local/` | 30% | compose/ and seeds/ directories partially populated |
| `infra/kubernetes/` | 20% | bootstrap/ scripts present, base/, addons/, rendered/, overlays/ empty |
| `infra/gitops/` | 10% | flux/ directory scaffolded but empty |
| `infra/security/` | 10% | sops/, supply-chain/, cluster-policies/ scaffolded but empty |
| `infra/images/` | 20% | Some Dockerfiles may exist, needs review |
| `ops/` | 15% | migrations/, benchmark/, resilience/, backup-restore/, runbooks/, scripts/ mostly empty |
| `fixtures/` | 20% | Some seed data present, needs completion |
| `tools/` | 30% | web3/ scripts present, codegen/, loadtest/, diagnostics/ empty |
| `apps/mobile/` | 0% | Not in workspace yet |
| `apps/browser-extension/` | 5% | Empty stub with .gitkeep + README |

---

## Key Structural Deviations from ARCHITECTURE.md

### 1. Package Organization Mismatch

**ARCHITECTURE.md specifies:**
```
packages/
├── kernel/
├── platform/
├── contracts/
├── sdk/
├── runtime/
├── authn/
├── authz/
├── data/
├── messaging/
├── cache/
├── storage/
├── observability/
├── security/
├── web3/
├── wasm/
├── ui/
└── devx/
```

**Current structure uses:**
```
packages/
├── core/          # Contains: workspace-hack, domain, kernel, platform
├── contracts/     # Contains: api, auth, events, errors
├── shared/        # Contains: errors, utils, config, env, testing, types
├── adapters/      # Contains: telemetry, hosts, storage, auth, cache, chains, protocols
└── features/      # Contains: auth, counter, admin, agent, chat, settings, feed, notifications, payments, profile, social-graph, wallet
```

**Impact**: This is a significant deviation. The current structure mixes concerns that ARCHITECTURE.md keeps separate. For example:
- `packages/core/kernel` should be `packages/kernel`
- `packages/core/platform` should be `packages/platform`
- `packages/shared/*` doesn't exist in ARCHITECTURE.md - these should be distributed to appropriate packages
- `packages/adapters/*` is too flat - should be nested under capability packages (cache/adapters/, storage/adapters/, etc.)
- `packages/features/*` doesn't exist in ARCHITECTURE.md - features should emerge from services + contracts
- Missing: `packages/runtime/`, `packages/sdk/`, `packages/authn/`, `packages/authz/`, `packages/data/`, `packages/messaging/`, `packages/observability/`, `packages/security/`, `packages/wasm/`, `packages/devx/`

### 2. Services vs Features Confusion

ARCHITECTURE.md has `services/*` as the only business logic location. The current `packages/features/*` crates duplicate this concern and should be either:
- Merged into `services/*`
- Moved to UI-level feature components in `apps/web/src/lib/features/`

### 3. Contract Structure

ARCHITECTURE.md specifies `packages/contracts/` should contain: `http/`, `events/`, `rpc/`, `jsonschema/`, `error-codes/`, `compat/`, `sdk-gen/`

Current structure has: `packages/contracts/api/`, `auth/`, `events/`, `errors/`

---

## Risk Assessment

### High Risk
1. **Package structure deviation** - Makes it harder to align with ARCHITECTURE.md rules about dependency directions
2. **CI workflows broken** - Stale paths mean no actual CI validation
3. **No validators** - Platform model has no validation enforcement
4. **Workers are all stubs** - Core async architecture not functional
5. **Missing runtime abstraction** - `packages/runtime/` is critical for Dapr/direct/memory switching

### Medium Risk
1. **Services incomplete** - Business logic skeletons present but not wired
2. **Documentation gaps** - ADRs missing, architecture diagrams empty
3. **No E2E tests** - Can't verify full system integration
4. **Missing root files** - typos.toml, .editorconfig, etc.

### Low Risk
1. **Infra incomplete** - Expected for early stage
2. **Ops empty** - Expected for early stage
3. **Mobile app missing** - Can be added later

---

## Recommendations

### Priority 1: Fix Foundation (Week 1-2)
1. Fix CI workflow paths
2. Add missing root files
3. Implement platform validators
4. Wire up existing services to BFFs
5. Get counter service E2E test passing

### Priority 2: Restructure Packages (Week 3-4)
1. Plan migration from current package structure to ARCHITECTURE.md structure
2. Implement `packages/runtime/ports/` (critical path)
3. Implement `packages/kernel/` (move from core)
4. Restructure adapters into capability packages

### Priority 3: Implement Workers (Week 5-6)
1. Implement indexer worker with real event source
2. Implement outbox-relay with real polling
3. Implement projector with real event consumption
4. Add checkpoint/retry/dedupe to all workers

### Priority 4: Complete Services (Week 7-8)
1. Complete user service domain + application
2. Complete tenant service with onboarding workflow
3. Wire auth service with Google OAuth
4. Complete settings service

### Priority 5: Add Missing Packages (Week 9-10)
1. Implement `packages/runtime/adapters/` (memory, direct, dapr)
2. Implement `packages/messaging/nats/`
3. Implement `packages/cache/` with adapters
4. Implement `packages/observability/`

### Priority 6: Infrastructure & Ops (Week 11-12)
1. Complete local-dev compose setup
2. Implement Kubernetes base manifests
3. Add runbooks
4. Set up backup/restore演练

---

## Success Metrics

- [ ] All CI workflows pass
- [ ] Platform validators catch model errors
- [ ] Counter service works E2E (web → BFF → service → DB)
- [ ] At least 1 worker processes real events
- [ ] Package structure matches ARCHITECTURE.md
- [ ] Dependency direction rules enforced
- [ ] Generated artifacts zero-drift in CI
- [ ] At least 3 ADRs documenting key decisions

---

## Next Steps

See `REFACTORING-ROADMAP.md` for detailed phased plan with handoff protocol.
