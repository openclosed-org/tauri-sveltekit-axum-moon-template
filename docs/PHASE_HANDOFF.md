# Phase Handoff Guide for Agent Teams

> **Purpose**: Enable fast, correct handoff between agents working on different phases of the refactoring plan  
> **How to use**: When starting a new phase, read this file + the relevant phase section in `docs/REFACTORING_PLAN.md`  
> **Last updated**: 2026-04-12

---

## Quick Start for Any Agent

When you're handed this project, read these files IN ORDER:

1. **This file** (`docs/PHASE_HANDOFF.md`) - Understand where we are
2. **`docs/REFACTORING_PLAN.md`** - Full plan details
3. **`docs/ARCHITECTURE.md`** - Absolute constitution (rules)
4. **`AGENTS.md`** - Working conventions
5. **Current git status** - `git status && git log --oneline -5`

Then execute the phase tasks.

---

## Current Project State

### What Exists ✅
- Root configs: `.mise.toml`, `Cargo.toml`, `moon.yml`, `Justfile`, `bun-workspace.yaml`
- Services: counter (golden), settings (golden), user, tenant, agent, event-bus
- Servers: api (Axum), bff/web-bff, gateway (Pingora)
- Apps: web (SvelteKit), desktop (Tauri v2)
- Packages: kernel, platform, contracts, features, adapters, shared, sdk, ui
- CI: `.github/workflows/`
- Infra scaffolding: docker compose, k3s, terraform

### What's Missing ❌
- **platform/** directory (model truth source)
- **workers/** directory (async execution units)
- **verification/** directory (cross-module testing)
- Complete implementations: chat-service, user HTTP routes, admin ports/infra

### Known Violations ⚠️
- `admin-service` depends on `tenant-service` and `counter-service` (violates service isolation)
- `user-service` depends on `axum` (violates layering)
- `servers/indexer/` should be `workers/indexer/`

---

## Phase Status Board

Update this table as phases complete:

| Phase | Status | Completion Report | Git Commit | Notes |
|-------|--------|-------------------|------------|-------|
| Phase 1: Fix deps | ✅ COMPLETE | `docs/refactoring/phase-1-completion.md` | `d25ce49` | Zero violations! |
| Phase 2: platform/ | ✅ COMPLETE | `docs/refactoring/phase-2-completion.md` | _pending_ | 32 models validated, catalog generated |
| Phase 3: workers/ | ⬜ Not Started | - | - | After Phase 2 |
| Phase 4: verification/ | ⬜ Not Started | - | - | After Phase 2 |
| Phase 5: servers/ | ⬜ Not Started | - | - | After Phase 2 |
| Phase 6: services | ⬜ Not Started | - | - | After Phase 1 |
| Phase 7: commands/CI | ⬜ Not Started | - | - | After Phase 2 |
| Phase 8: final | ⬜ Not Started | - | - | All above first |

---

## Handoff Templates

### When Completing a Phase

Create `docs/refactoring/phase-X-completion.md` with this structure:

```markdown
# Phase X Completion Report

**Status**: COMPLETE ✅  
**Completed by**: <agent/team>  
**Date**: <YYYY-MM-DD>  
**Git commit**: `<sha>` (run `git rev-parse HEAD`)

## What Was Done

### Tasks Completed
- [x] Task 1: Description
- [x] Task 2: Description
- [ ] Task 3: Description (if deferred)

### Files Created/Modified
- `path/to/file1.rs` - What changed and why
- `path/to/file2.yaml` - What changed and why

### Tests Added
- `path/to/test1.rs` - What it tests
- `path/to/test2.ts` - What it tests

## Verification

### Commands Run
```bash
just build              # ✅ Pass
just test               # ✅ Pass (XXX tests)
just validate-platform  # ✅ Pass (if applicable)
cargo test -p <package> # ✅ Pass
```

### Test Results
- Unit tests: XXX passing, 0 failing
- Integration tests: XXX passing, 0 failing
- E2E tests: XXX passing, 0 failing (if applicable)

## Known Issues

### Blocking Issues
- None (or describe if any)

### Non-Blocking Issues
- Issue #1: Description, impact, workaround if any
- Issue #2: Tracked in GitHub issue #XXX

## Technical Debt Created
- Description of any shortcuts taken
- Why it's acceptable for now
- Follow-up issue created: #XXX

## Next Phase Readiness

### Dependencies Delivered
- ✅ Dependency 1: What was delivered
- ✅ Dependency 2: What was delivered

### Documentation Updated
- ✅ `docs/REFACTORING_PLAN.md` - Phase status updated
- ✅ `docs/PHASE_HANDOFF.md` - This file updated
- ✅ `docs/ARCHITECTURE.md` - If any rule clarifications needed

### Next Phase Agent Brief
<3-5 sentences summarizing state for next agent>

## Review Checklist
- [ ] All acceptance criteria from REFACTORING_PLAN.md met
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Git commit message clear
- [ ] No unintended changes in commit
- [ ] This completion report reviewed for accuracy
```

---

## Phase-Specific Agent Briefs

Use these briefs to quickly orient new agents to their phase.

### Phase 1 Agent Brief

**Mission**: Fix dependency violations in existing services  
**Key files to read**:
- `services/admin-service/Cargo.toml` - Has cross-service deps to remove
- `services/user-service/Cargo.toml` - Has axum dep to remove
- `docs/REFACTORING_PLAN.md` §Phase 1 - Detailed tasks

**What to do**:
1. Read current service Cargo.toml files
2. Understand what they're using from cross-deps
3. Extract to contracts/ports/composition layer
4. Remove violations
5. Verify all services still compile and pass tests

**Verification**:
```bash
cargo test -p admin-service  # Must pass
cargo test -p user-service   # Must pass
cargo tree -p admin-service | grep -E "tenant|counter"  # Must be empty
rg "axum" services/user-service/src/  # Must be empty
```

**Risk**: HIGH - May break existing code. Test thoroughly before committing.

---

### Phase 2 Agent Brief

**Mission**: Create platform/ directory (model truth source)  
**Prerequisites**: Phase 1 complete  
**Key files to read**:
- `docs/ARCHITECTURE.md` §3.3 platform/ - Rules
- `docs/REFACTORING_PLAN.md` §Phase 2 - Detailed tasks
- Existing service structures to model

**What to do**:
1. Create `platform/` directory structure
2. Write JSON schemas for all platform concepts
3. Model existing services as YAML files
4. Build validator crate
5. Build generator crate
6. Generate `platform/catalog/`

**Verification**:
```bash
just validate-platform  # Must pass
just gen-platform       # Must produce reproducible output
```

**Risk**: MEDIUM - New structure, but defines truth source for everything else.

---

### Phase 3 Agent Brief

**Mission**: Create workers/ directory and migrate async operations  
**Prerequisites**: Phase 2 complete (workers need platform models)  
**Key files to read**:
- `docs/ARCHITECTURE.md` §3.7 workers/ - Rules
- `docs/REFACTORING_PLAN.md` §Phase 3 - Detailed tasks
- `services/event-bus/` - Existing outbox logic to migrate

**What to do**:
1. Create `workers/` directory structure
2. Implement outbox-relay worker (migrate from event-bus)
3. Implement indexer worker (move from servers/indexer)
4. Implement projector, scheduler, sync-reconciler workers
5. Add workers to Cargo workspace
6. Add worker startup scripts to Justfile

**Verification**:
```bash
cargo build -p outbox-relay-worker   # Must compile
cargo build -p indexer-worker        # Must compile
cargo test -p <worker>               # All must pass
```

**Risk**: MEDIUM - New functionality, must integrate with existing event bus.

---

### Phase 4 Agent Brief

**Mission**: Create verification/ directory structure  
**Prerequisites**: Phase 2 complete  
**Key files to read**:
- `docs/ARCHITECTURE.md` §3.11 verification/ - Rules
- `docs/REFACTORING_PLAN.md` §Phase 4 - Detailed tasks
- Existing test locations: `apps/web/tests/`, `apps/desktop/tests/`

**What to do**:
1. Create `verification/` directory structure
2. Migrate existing E2E tests
3. Add contract compatibility tests
4. Add topology verification tests
5. Add resilience tests
6. Create golden baseline

**Verification**:
```bash
just test-e2e-full   # Must pass
just verify-generated  # Must pass (zero drift)
```

**Risk**: LOW - Additive only, no breaking changes.

---

### Phase 5 Agent Brief

**Mission**: Restructure servers/ to match target architecture  
**Prerequisites**: Phase 2 complete  
**Key files to read**:
- `docs/ARCHITECTURE.md` §3.5 servers/ - Rules
- `docs/REFACTORING_PLAN.md` §Phase 5 - Detailed tasks
- Current `servers/api/` and `servers/bff/web-bff/` structures

**What to do**:
1. Clean up `servers/api/` (remove composition logic)
2. Complete `servers/bff/web-bff/`
3. Create `servers/bff/admin-bff/`
4. Restructure `servers/gateway/`
5. Ensure all servers have OpenAPI specs

**Verification**:
```bash
cargo build -p api-server        # Must compile
cargo build -p web-bff           # Must compile
cargo build -p admin-bff         # Must compile
# OpenAPI spec matches routes
```

**Risk**: MEDIUM - Server routing changes may break API.

---

### Phase 6 Agent Brief

**Mission**: Complete missing service implementations  
**Prerequisites**: Phase 1 complete (dependency violations fixed)  
**Key files to read**:
- `docs/ARCHITECTURE.md` §3.6 services/ - Rules
- `docs/REFACTORING_PLAN.md` §Phase 6 - Detailed tasks
- Counter service as golden example: `services/counter-service/`

**What to do**:
1. Complete user-service HTTP routes
2. Implement chat-service from scratch
3. Complete admin-service with ports/infrastructure
4. Audit all services for completeness

**Verification**:
```bash
cargo test -p user-service    # Must pass
cargo test -p chat-service    # Must pass
cargo test -p admin-service   # Must pass
```

**Risk**: MEDIUM - Completing stub implementations.

---

### Phase 7 Agent Brief

**Mission**: Add platform commands and CI validation  
**Prerequisites**: Phase 2 complete  
**Key files to read**:
- `docs/REFACTORING_PLAN.md` §Phase 7 - Detailed tasks
- Current `Justfile` and `justfiles/` structure
- Current `.github/workflows/` structure

**What to do**:
1. Add platform commands to Justfile
2. Create CI workflow for platform validation
3. Create platform validator crate
4. Create contract drift detector
5. Wire everything together

**Verification**:
```bash
just validate-platform   # Must work
just gen-platform        # Must work
just validate-deps       # Must work
just doctor              # Must work
```

**Risk**: LOW - Tooling only, no breaking changes.

---

### Phase 8 Agent Brief

**Mission**: Final verification and golden baseline  
**Prerequisites**: ALL previous phases complete  
**Key files to read**:
- `docs/REFACTORING_PLAN.md` §Phase 8 - Detailed tasks
- All previous phase completion reports

**What to do**:
1. Run full verification suite
2. Commit golden baseline
3. Document final architecture state
4. Create post-refactoring review

**Verification**:
```bash
just verify              # Must pass completely
just doctor              # Must pass
just boundary-check      # Must pass
just contracts-check     # Must pass
```

**Risk**: LOW - Verification only.

---

## Common Agent Mistakes to Avoid

### ❌ Don't Do This
1. **Don't read `node_modules/` or `target/`** - These are build artifacts (AGENTS.md §6)
2. **Don't guess at dependencies** - Use `cargo tree` or `grep_search`
3. **Don't modify generated files** - They'll be regenerated and overwritten
4. **Don't skip verification** - Every change must be tested
5. **Don't refactor working code** - Only fix violations of ARCHITECTURE.md rules

### ✅ Do This Instead
1. **Read before writing** - Always understand current state first
2. **Use evidence** - Logs, tests, `git diff` over assumptions
3. **Minimal changes** - Smallest possible diff to achieve goal
4. **Verify immediately** - Test after every logical unit
5. **Document decisions** - ADRs for non-obvious choices

---

## Emergency Procedures

### If Something Breaks

1. **Stop immediately** - Don't compound the problem
2. **Check git status** - `git status && git diff --stat`
3. **Run tests** - `just test` to see scope of breakage
4. **Revert if needed** - `git stash` or `git revert <commit>`
5. **Document what happened** - In `docs/refactoring/INCIDENTS.md`

### If You're Lost

1. **Read ARCHITECTURE.md again** - It's the constitution
2. **Check REFACTORING_PLAN.md** - You should have a phase assignment
3. **Look at golden examples** - `services/counter-service/` is the gold standard
4. **Ask for clarification** - Don't guess when stuck

---

## Tool Quick Reference

### Essential Commands

```bash
# Project structure
cargo metadata --format-version 1 | jq '.packages[].name'  # List all crates
cargo tree -p <crate>  # Show dependency tree
rg "pattern" --type rust  # Search Rust code

# Build and test
just build              # Build workspace
just test               # Run tests
just verify             # Full verification
just doctor             # Platform health check

# Git operations
git status              # Current state
git diff --stat         # What changed
git log --oneline -10   # Recent history
git stash               # Save work in progress
git revert <commit>     # Undo a commit

# File operations
rg "axum" services/     # Find axum usage in services
rg "services/" services/*/Cargo.toml  # Find cross-service deps
```

### Search Patterns for Common Tasks

```bash
# Find dependency violations
for svc in services/*/; do
  cargo tree -p $(basename $svc) | grep "services/"
done

# Find framework imports in business logic
rg "axum|tauri|hyper" services/*/src/domain/

# Find generated files that were hand-modified
git diff packages/contracts/generated/
git diff packages/sdk/typescript/

# Check service completeness
for svc in services/*/; do
  echo "=== $(basename $svc) ==="
  ls -d $svc/src/{domain,application,policies,ports,events,contracts} 2>&1
done
```

---

## File Location Cheat Sheet

### Where to Find Things

| Concept | Location |
|---------|----------|
| Service business logic | `services/<name>/src/domain/` + `application/` |
| Service abstractions | `services/<name>/src/ports/` |
| Service implementations | `services/<name>/src/infrastructure/` |
| HTTP routes | `servers/api/src/routes/` or `servers/bff/*/handlers/` |
| Shared types | `packages/contracts/` |
| Feature traits | `packages/features/<domain>/` |
| External adapters | `packages/adapters/` |
| Frontend pages | `apps/web/src/routes/` |
| Tauri commands | `packages/adapters/hosts/tauri/src/commands/` |
| Platform models (future) | `platform/model/services/*.yaml` |
| Workers (future) | `workers/<name>/` |

### Golden Examples

| Pattern | Example |
|---------|---------|
| Complete service | `services/counter-service/` |
| Complete settings service | `services/settings-service/` |
| Complete server route | `servers/api/src/routes/counter.rs` |
| Complete Tauri command | `packages/adapters/hosts/tauri/src/commands/counter.rs` |
| Complete frontend page | `apps/web/src/routes/(app)/counter/+page.svelte` |

When implementing anything new, **copy the pattern from the golden example**.

---

## Next Steps

1. **Update this file** with current phase status before starting work
2. **Read the phase brief** for your assigned phase
3. **Read the full phase details** from `docs/REFACTORING_PLAN.md`
4. **Execute tasks** one at a time, verifying each
5. **Write completion report** when done
6. **Update this file** for next agent

---

**Remember**: 
- ARCHITECTURE.md is the constitution (rules)
- REFACTORING_PLAN.md is the law (tasks)
- This file is the map (orientation)
- Counter service is the model (pattern)

**When in doubt**: Read more, guess less. Test early, test often.
