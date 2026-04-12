# Refactoring Roadmap — Phased Plan with Handoff Protocol

> **Target**: Align repository with `docs/ARCHITECTURE.md` (the constitution)  
> **Strategy**: Incremental, verifiable, handoff-enabled phases  
> **Timeline**: 6 phases, each independently completable by different agents  
> **Risk Management**: Each phase has explicit entry/exit criteria, rollback plan, and handoff document

---

## Handoff Protocol Design

### Problem
Agent-based development suffers from:
1. Context loss between phases
2. Progress tracking fragmentation
3. Unclear handoff boundaries
4. Repeated work or conflicting changes

### Solution: Three-Layer Handoff System

#### Layer 1: Phase State File
**Location**: `.refactoring-state.yaml`

```yaml
# Current phase being worked on
current_phase: 1
phase_status:
  1:
    status: in_progress  # pending | in_progress | completed | blocked
    started_at: 2026-04-12
    completed_at: null
    agent_id: "agent-refactor-p1"
    blockers: []
    notes: "Fixing CI paths"
  2:
    status: pending
  3:
    status: pending
  4:
    status: pending
  5:
    status: pending
  6:
    status: pending

# Quick reference for any agent
last_verified:
  date: 2026-04-12
  checks_passed:
    - cargo check --workspace
  checks_failed: []
  manual_review_needed: []
```

**Rules**:
- MUST update this file before starting a phase
- MUST update this file when completing a phase
- MUST update this file when encountering a blocker

#### Layer 2: Phase Task Card
**Location**: `docs/refactoring/tasks/phase-N.md`

Each phase has a dedicated task card with:
- Phase objective
- Detailed checklist
- Files to touch
- Verification commands
- Known risks
- Handoff notes section

#### Layer 3: Agent Context Transfer
**Location**: `docs/refactoring/handoffs/handoff-N-to-N+1.md`

When an agent completes a phase, it MUST create a handoff document:

```markdown
# Handoff: Phase N → Phase N+1

## Completed
- [ ] What was accomplished
- [ ] What was verified

## Partially Complete
- [ ] What needs follow-up
- [ ] What was learned

## Blockers/Decisions
- [ ] What decisions were made and why
- [ ] What blockers remain

## Next Agent Instructions
- [ ] Exact starting point
- [ ] Commands to run to verify current state
- [ ] What to read first

## Changed Files
- [ ] Complete list of files modified
- [ ] Files that may need review

## Open Questions
- [ ] What still needs clarification
```

---

## Phase 1: Foundation Fixes

**Objective**: Fix broken infrastructure, enable CI, establish validation baseline

**Estimated Scope**: 20-30 files, low risk

**Entry Criteria**:
- [x] Current state assessed (PROGRESS-ASSESSMENT.md complete)
- [x] Handoff protocol designed
- [ ] `.refactoring-state.yaml` created

**Exit Criteria**:
- [ ] All CI workflows pass on PR
- [ ] Missing root files added
- [ ] `cargo check --workspace` passes
- [ ] Platform validators implemented and running
- [ ] Generated artifacts verified zero-drift

**Tasks**:

### 1.1 Fix CI Workflows
- [ ] Fix `ci.yml` stale paths (`apps/client/*` → `apps/*`)
- [ ] Fix path triggers for all 5 workflows
- [ ] Test with dry run
- [ ] Verify PR triggers work

**Files to touch**:
- `.github/workflows/ci.yml`
- `.github/workflows/coverage.yml`
- `.github/workflows/e2e-tests.yml`
- `.github/workflows/platform-validation.yml`
- `.github/workflows/quality-gate.yml`

**Verification**:
```bash
# Simulate workflow triggers
act -l  # if using nektos/act
# Or create test PR
```

### 1.2 Add Missing Root Files
- [ ] `typos.toml` - spell checking config
- [ ] `.editorconfig` - editor consistency
- [ ] `.cargo/audit.toml` - audit config
- [ ] `.config/nextest.toml` - test runner config

**Files to create**:
- `typos.toml`
- `.editorconfig`
- `.cargo/audit.toml`
- `.config/nextest.toml`

### 1.3 Implement Platform Validators
- [ ] `platform/validators/model-lint/` - validate YAML models
- [ ] `platform/validators/dependency-graph/` - check dependency directions
- [ ] `platform/validators/contract-drift/` - detect contract drift
- [ ] `platform/validators/topology-check/` - validate topology completeness
- [ ] `platform/validators/security-check/` - basic security checks
- [ ] `platform/validators/observability-check/` - verify telemetry setup

**Files to create**:
- `platform/validators/model-lint/Cargo.toml`
- `platform/validators/model-lint/src/main.rs`
- (repeat for each validator)

**Integration**:
- Add to Justfile: `just validate-platform`
- Add to CI: platform-validation.yml

### 1.4 Wire Up Existing Services
- [ ] Connect counter-service to web-bff
- [ ] Connect user-service to web-bff
- [ ] Verify API calls work end-to-end
- [ ] Add OpenAPI spec alignment

**Files to touch**:
- `servers/bff/web-bff/src/routes/`
- `servers/bff/web-bff/src/handlers/`
- `services/counter-service/src/`
- `services/user-service/src/`

### 1.5 Fix Generated Artifacts
- [ ] Ensure `gen-platform` regenerates catalog
- [ ] Ensure `gen-contracts` produces zero diff
- [ ] Add drift detection to CI

**Verification commands**:
```bash
just validate-platform
just gen-platform
just gen-contracts
git diff --exit-code  # should be clean
```

**Risks**:
- Low risk: mostly config fixes and wiring
- May uncover hidden assumptions in current code

**Rollback Plan**:
- All changes are additive or path fixes
- Git revert is straightforward

**Handoff to Phase 2**:
- Create `docs/refactoring/handoffs/handoff-1-to-2.md`
- Update `.refactoring-state.yaml`
- Run full verification suite

---

## Phase 2: Package Structure Realignment

**Objective**: Migrate from current package structure to ARCHITECTURE.md structure

**Estimated Scope**: 50-80 files, MEDIUM risk

**Entry Criteria**:
- [ ] Phase 1 complete (CI passing, validators working)
- [ ] Full dependency graph mapped
- [ ] Migration plan reviewed

**Exit Criteria**:
- [ ] All packages match ARCHITECTURE.md structure
- [ ] All Cargo.toml workspace members updated
- [ ] `cargo check --workspace` passes
- [ ] No broken imports
- [ ] Dependency direction rules enforced

**Tasks**:

### 2.1 Pre-Migration Analysis
- [ ] Map current packages to target structure
- [ ] Identify all cross-references
- [ ] Create migration script plan
- [ ] Document all decisions

**Deliverable**: `docs/refactoring/package-migration-plan.md`

### 2.2 Create Target Structure (Parallel)
Create new package directories without moving code yet:
- [ ] `packages/kernel/` (ids, error, money, pagination, tenancy, time)
- [ ] `packages/platform/` (config, health, buildinfo, env, release, service_meta)
- [ ] `packages/runtime/` (ports/, policy/, adapters/)
- [ ] `packages/sdk/` (typescript/, rust/, openapi-clients/)
- [ ] `packages/authn/` (oidc, pkce, session, token)
- [ ] `packages/authz/` (model, ports, caching, decision, adapters/openfga)
- [ ] `packages/data/` (turso, sqlite, migration, outbox, inbox, common-sql)
- [ ] `packages/messaging/` (nats, envelope, codec)
- [ ] `packages/cache/` (api, policies, adapters/)
- [ ] `packages/storage/` (api, paths, policies, adapters/)
- [ ] `packages/observability/` (tracing, metrics, logging, baggage, otel)
- [ ] `packages/security/` (crypto, signing, redaction, pii)
- [ ] `packages/wasm/` (wit, host, guest-sdk, components/)
- [ ] `packages/devx/` (testkit, fixture-loader, contract-test, perf-harness, snapshot)

### 2.3 Migrate Core Packages (Sequential - Order Matters)

**Order**: kernel → platform → runtime/ports → contracts → everything else

#### 2.3.1 Kernel
- [ ] Move `packages/core/kernel` → `packages/kernel`
- [ ] Update workspace Cargo.toml
- [ ] Fix all imports
- [ ] Verify `cargo build -p kernel`

#### 2.3.2 Platform
- [ ] Move `packages/core/platform` → `packages/platform`
- [ ] Update all references
- [ ] Verify build

#### 2.3.3 Runtime Ports
- [ ] Create `packages/runtime/ports/` (invocation, pubsub, state, workflow, lock, binding, secret, queue)
- [ ] Extract from current locations
- [ ] Wire up to services

#### 2.3.4 Restructure Contracts
- [ ] Move to: `http/`, `events/`, `rpc/`, `jsonschema/`, `error-codes/`, `compat/`, `sdk-gen/`
- [ ] Update generators
- [ ] Verify typegen still works

#### 2.3.5 Restructure Adapters
- [ ] Move flat `packages/adapters/*` into capability packages:
  - `packages/cache/adapters/` (moka, valkey, dragonfly)
  - `packages/storage/adapters/` (surrealdb, turso, sqlite, extension-storage, indexeddb, tauri-store)
  - `packages/data/turso/`, `data/sqlite/`, etc.
  - `packages/messaging/nats/`
  - `packages/observability/` (otel, tracing)
  - `packages/authn/` (google, dpop, oauth, passkey)
  - `packages/authz/`
  - `packages/web3/` (chains, protocols)

#### 2.3.6 Handle Features
- [ ] Decide: merge into services OR move to app-level features
- [ ] Execute decision
- [ ] Update all references

### 2.4 Update Build System
- [ ] Update workspace Cargo.toml members
- [ ] Update bun-workspace.yaml
- [ ] Update moon.yml tasks
- [ ] Update Justfile paths
- [ ] Update CI workflow paths

### 2.5 Verification
```bash
cargo check --workspace
cargo test --workspace
just gen-contracts
just gen-platform
just boundary-check
git diff --exit-code
```

**Risks**:
- MEDIUM risk: broken imports across codebase
- May need iterative fixes
- Boundary check may fail initially

**Rollback Plan**:
- Each sub-phase should be independently commitable
- Can revert individual package moves
- Feature flag new structure if needed

**Handoff to Phase 3**:
- Complete package structure verification
- Update `.refactoring-state.yaml`
- Create handoff document with full file inventory

---

## Phase 3: Runtime & Workers Implementation

**Objective**: Implement core runtime abstraction and make workers functional

**Estimated Scope**: 40-60 files, MEDIUM-HIGH risk

**Entry Criteria**:
- [ ] Phase 2 complete (package structure aligned)
- [ ] Runtime ports defined
- [ ] Service interfaces stable

**Exit Criteria**:
- [ ] `packages/runtime/ports/` fully defined
- [ ] `packages/runtime/adapters/memory/` working
- [ ] `packages/runtime/adapters/direct/` working
- [ ] At least 2 workers processing real events
- [ ] Checkpoint/retry/dedupe working

**Tasks**:

### 3.1 Runtime Ports Implementation
- [ ] `packages/runtime/ports/invocation.rs`
- [ ] `packages/runtime/ports/pubsub.rs`
- [ ] `packages/runtime/ports/state.rs`
- [ ] `packages/runtime/ports/workflow.rs`
- [ ] `packages/runtime/ports/lock.rs`
- [ ] `packages/runtime/ports/binding.rs`
- [ ] `packages/runtime/ports/secret.rs`
- [ ] `packages/runtime/ports/queue.rs`

### 3.2 Runtime Policy Engine
- [ ] `packages/runtime/policy/timeout/`
- [ ] `packages/runtime/policy/retry/`
- [ ] `packages/runtime/policy/idempotency/`
- [ ] `packages/runtime/policy/backpressure/`
- [ ] `packages/runtime/policy/circuit_breaker/`

### 3.3 Memory Adapters (for testing)
- [ ] `packages/runtime/adapters/memory/` - full implementation
- [ ] Wire to services for unit tests
- [ ] Verify services work without external deps

### 3.4 Direct Adapters
- [ ] `packages/runtime/adapters/direct/` - in-process direct calls
- [ ] Integration with existing services

### 3.5 Worker: Indexer
- [ ] Real EventSource implementation
- [ ] Transform pipeline
- [ ] Sink implementation
- [ ] Checkpoint system
- [ ] Integration test

### 3.6 Worker: Outbox Relay
- [ ] Real outbox table polling
- [ ] Event publishing
- [ ] Deduplication
- [ ] Error handling
- [ ] Integration test

### 3.7 Worker: Projector
- [ ] Event consumers
- [ ] Read model builders
- [ ] Projection state
- [ ] Recovery logic

### 3.8 Integration Testing
```bash
# Start workers independently
cargo run -p worker-indexer
cargo run -p worker-outbox-relay

# Verify checkpoint persistence
# Verify duplicate handling
# Verify recovery after restart
```

**Risks**:
- MEDIUM-HIGH: First real distributed systems code
- May expose design flaws in service interfaces
- Checkpoint schema decisions are critical

**Rollback Plan**:
- Workers can run in stub mode initially
- Feature flag real implementations
- Memory adapters provide fallback

**Handoff to Phase 4**:
- Workers processing events
- Runtime abstraction stable
- Create comprehensive handoff document

---

## Phase 4: Service Completion & Integration

**Objective**: Complete core services and wire full system integration

**Estimated Scope**: 60-100 files, MEDIUM risk

**Entry Criteria**:
- [ ] Phase 3 complete (workers functional)
- [ ] Runtime abstraction stable
- [ ] Service interfaces defined

**Exit Criteria**:
- [ ] User service complete with domain + application
- [ ] Tenant service complete with onboarding workflow
- [ ] Auth service working with Google OAuth
- [ ] Settings service complete
- [ ] All services wired to BFFs
- [ ] E2E counter test passing

**Tasks**:

### 4.1 Complete User Service
- [ ] Domain entities and rules
- [ ] Application use cases
- [ ] Port implementations
- [ ] Event definitions
- [ ] Contracts
- [ ] Tests (unit + integration)
- [ ] Migrations

### 4.2 Complete Tenant Service
- [ ] Domain entities
- [ ] Multi-tenant isolation
- [ ] Onboarding workflow
- [ ] Port implementations
- [ ] Events
- [ ] Contracts
- [ ] Tests
- [ ] Migrations

### 4.3 Complete Auth Service
- [ ] OAuth flow implementation
- [ ] Session management
- [ ] Token handling
- [ ] Integration with user service
- [ ] Integration with tenant service
- [ ] Tests

### 4.4 Complete Settings Service
- [ ] Domain model
- [ ] CRUD operations
- [ ] Validation
- [ ] Events
- [ ] Contracts
- [ ] Tests
- [ ] Migrations

### 4.5 Wire BFFs to Services
- [ ] web-bff → all services
- [ ] admin-bff → admin service
- [ ] OpenAPI spec alignment
- [ ] Error handling
- [ ] Telemetry injection

### 4.6 E2E Integration Tests
- [ ] Demo counter flow
- [ ] Multi-tenant flow
- [ ] Settings flow
- [ ] Desktop + web roundtrip

**Verification**:
```bash
cargo test --workspace
just test-e2e-full
just test-desktop
```

**Risks**:
- MEDIUM: Business logic complexity
- OAuth flows can be tricky
- Multi-tenant isolation critical

**Rollback Plan**:
- Feature flag incomplete services
- Stub implementations as fallback

**Handoff to Phase 5**:
- Core services working
- E2E flows passing
- System integration verified

---

## Phase 5: Infrastructure & Deployment

**Objective**: Complete infrastructure setup and enable deployment

**Estimated Scope**: 40-60 files, LOW-MEDIUM risk

**Entry Criteria**:
- [ ] Phase 4 complete (services working)
- [ ] System functional locally

**Exit Criteria**:
- [ ] Local dev compose setup working
- [ ] Kubernetes base manifests
- [ ] GitOps flux bootstrap
- [ ] Runbooks written
- [ ] Backup/restore tested

**Tasks**:

### 5.1 Local Development
- [ ] `infra/local/compose/core.yaml`
- [ ] `infra/local/compose/observability.yaml`
- [ ] `infra/local/seeds/` - demo data
- [ ] Bootstrap script

### 5.2 Kubernetes
- [ ] `infra/kubernetes/base/` - namespaces, RBAC, etc.
- [ ] `infra/kubernetes/addons/` - NATS, cache, etc.
- [ ] Render manifests from platform model
- [ ] Overlay for dev/staging/prod

### 5.3 GitOps
- [ ] Flux bootstrap
- [ ] Application definitions
- [ ] Policy definitions

### 5.4 Operations
- [ ] Migration runner
- [ ] Runbooks
- [ ] Backup/restore scripts
- [ ] Health checks

### 5.5 Security
- [ ] SOPS setup
- [ ] Secret management
- [ ] Supply chain security

**Verification**:
```bash
just render-local
just render-k8s env=dev
just verify-single-vps
```

**Risks**:
- LOW-MEDIUM: Infrastructure as code
- Well-documented patterns available
- Can test locally first

**Handoff to Phase 6**:
- Infrastructure deployable
- Local dev working
- Production path defined

---

## Phase 6: Polish & Documentation

**Objective**: Complete documentation, add missing pieces, establish patterns

**Estimated Scope**: 30-50 files, LOW risk

**Entry Criteria**:
- [ ] Phase 5 complete (infrastructure working)
- [ ] System fully functional

**Exit Criteria**:
- [ ] ADRs written for key decisions
- [ ] Architecture diagrams
- [ ] Contract documentation
- [ ] Operations guide
- [ ] Generated docs complete
- [ ] Browser extension scaffolded
- [ ] Mobile app planned

**Tasks**:

### 6.1 Architecture Decision Records
- [ ] 001-platform-model-first.md
- [ ] 002-services-are-libraries-not-processes.md
- [ ] 003-runtime-abstraction-direct-plus-dapr.md
- [ ] 004-k3s-cilium-gateway-api-flux.md
- [ ] 005-authn-authz-zitadel-openfga.md
- [ ] 006-observability-vector-openobserve.md
- [ ] 007-workers-first-async-architecture.md
- [ ] 008-wasm-extension-plane.md

### 6.2 Architecture Diagrams
- [ ] Context diagrams
- [ ] Container diagrams
- [ ] Component diagrams
- [ ] Sequence diagrams
- [ ] Deployment diagrams
- [ ] Topology diagrams

### 6.3 Contract Documentation
- [ ] HTTP API docs
- [ ] Event schema docs
- [ ] RPC docs
- [ ] Error code catalog

### 6.4 Operations Guide
- [ ] Local dev guide
- [ ] Single VPS guide
- [ ] K3s cluster guide
- [ ] GitOps guide
- [ ] Secret management guide

### 6.5 Generated Documentation
- [ ] Service catalog
- [ ] Resource catalog
- [ ] Dependency graphs

### 6.6 Missing Apps
- [ ] Browser extension scaffold with real code
- [ ] Mobile app planning
- [ ] Feature parity matrix

**Verification**:
```bash
just gen-platform
# Verify docs generated
# Verify all ADRs present
```

**Risks**:
- LOW: Documentation work
- Can be done incrementally

**Handoff**:
- Refactoring complete
- Repository aligned with ARCHITECTURE.md
- Future work documented

---

## Parallel Execution Strategy

### Phases That Can Run in Parallel

After Phase 1 (foundation), some phases can parallelize:

**Track A: Structure**
- Phase 2: Package restructuring

**Track B: Implementation**
- Phase 3: Runtime & workers (can start on runtime ports while package structure being planned)

**Track C: Documentation**
- Phase 6: Can start writing ADRs for decisions already made

### Agent Assignment Strategy

```
Phase 1 → agent-foundation-fixes
Phase 2 → agent-package-structure
Phase 3 → agent-runtime-workers
Phase 4 → agent-services-integration
Phase 5 → agent-infrastructure
Phase 6 → agent-documentation
```

Each agent:
1. Reads `.refactoring-state.yaml` first
2. Reads `docs/refactoring/tasks/phase-N.md`
3. Reads handoff document from previous phase
4. Updates state file when starting
5. Creates handoff document when done
6. Runs verification commands before marking complete

---

## Monitoring & Recovery

### Progress Tracking
- Update `.refactoring-state.yaml` daily
- Review blockers in standup
- Adjust phases if needed

### Quality Gates
Each phase MUST pass:
```bash
cargo check --workspace
cargo clippy --workspace
just fmt
just test-unit
just validate-platform
just boundary-check
```

### Emergency Stop
If a phase introduces breaking changes:
```bash
git revert <phase-commit>
# Fix issues
# Reapply
```

### Communication Protocol
- Major decisions → update AGENTS.md or add ADR
- Blockers → update `.refactoring-state.yaml` with notes
- Completion → create handoff document

---

## Success Criteria (End State)

- [x] All CI/CD workflows passing
- [x] Package structure matches ARCHITECTURE.md
- [x] All services implemented and tested
- [x] All workers processing events
- [x] Runtime abstraction working
- [x] Infrastructure deployable
- [x] Documentation complete
- [x] Generated artifacts zero-drift
- [x ] Dependency directions enforced
- [x] E2E tests passing

---

## Appendix: Quick Reference

### Key Commands
```bash
# Before starting any phase
just doctor
just check

# During implementation
cargo check --workspace
cargo clippy --workspace

# Before marking phase complete
just verify
just test-unit
just validate-platform
just boundary-check
git diff --exit-code

# Generate artifacts
just gen-platform
just gen-contracts
just gen-sdk
```

### Important Files
- `.refactoring-state.yaml` - current progress
- `docs/refactoring/tasks/phase-N.md` - detailed task list
- `docs/refactoring/handoffs/handoff-N-to-N+1.md` - context transfer
- `docs/ARCHITECTURE.md` - the constitution
- `AGENTS.md` - working agreements

### Contacts
- Each phase agent ID tracked in state file
- Handoff documents link consecutive phases
- Questions → update task card with blockers
