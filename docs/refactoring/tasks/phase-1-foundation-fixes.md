# Phase 1 Task Card: Foundation Fixes

> **Objective**: Fix broken infrastructure, enable CI, establish validation baseline  
> **Risk Level**: LOW  
> **Estimated Scope**: 20-30 files  
> **Dependencies**: None (can start immediately)  
> **Read First**: `docs/refactoring/PROGRESS-ASSESSMENT.md`

---

## Pre-flight Checklist

Before starting:
- [ ] Read `docs/ARCHITECTURE.md` (the constitution)
- [ ] Read `AGENTS.md` (working agreements)
- [ ] Read `docs/refactoring/PROGRESS-ASSESSMENT.md` (current state)
- [ ] Update `.refactoring-state.yaml`:
  - Set `current_phase: 1`
  - Set `phase_status.1_foundation_fixes.status: in_progress`
  - Set `started_at` to today
  - Set `agent_id` to your identifier

---

## Task Checklist

### 1.1 Fix CI Workflows (Priority: CRITICAL)

**Why**: CI is broken and not validating anything

**Files to modify**:
- `.github/workflows/ci.yml`
- `.github/workflows/coverage.yml`
- `.github/workflows/e2e-tests.yml`
- `.github/workflows/platform-validation.yml`
- `.github/workflows/quality-gate.yml`

**What to do**:
- [ ] Search for all occurrences of `apps/client/` and replace with correct paths:
  - `apps/client/web/app` → `apps/web/src`
  - `apps/client/native/src-tauri/Cargo.toml` → `apps/desktop/src-tauri/Cargo.toml`
- [ ] Verify path triggers match actual directory structure
- [ ] Check for any other stale paths

**Verification**:
```bash
# Check syntax
actionlint .github/workflows/*.yml

# Or create a test PR to verify triggers work
```

**Risks**: Low - path fixes are straightforward

---

### 1.2 Add Missing Root Files (Priority: HIGH)

**Why**: Architecture constitution expects these files

**Files to create**:
- [ ] `typos.toml` - Spell checking configuration
- [ ] `.editorconfig` - Editor consistency across IDEs
- [ ] `.cargo/audit.toml` - Cargo audit configuration
- [ ] `.config/nextest.toml` - Nextest test runner configuration

**References**:
- Check similar projects for examples
- Keep configuration minimal and well-documented

**Verification**:
```bash
# Verify typos
typos

# Verify cargo audit
cargo audit

# Verify editorconfig
editorconfig-checker
```

**Risks**: Low - additive only

---

### 1.3 Implement Platform Validators (Priority: HIGH)

**Why**: Platform model needs validation enforcement

**Directories to implement**:
- `platform/validators/model-lint/`
- `platform/validators/dependency-graph/`
- `platform/validators/contract-drift/`
- `platform/validators/topology-check/`
- `platform/validators/security-check/`
- `platform/validators/observability-check/`

**Implementation approach**:
- [ ] Each validator should be a binary crate
- [ ] Read from `platform/model/` YAML files
- [ ] Apply validation rules
- [ ] Exit 0 on success, 1 on failure
- [ ] Add to Justfile with `just validate-*` commands

**Key validations**:

#### model-lint
- Validate all YAML against JSON schemas in `platform/schema/`
- Check required fields present
- Check naming conventions

#### dependency-graph
- Parse all `Cargo.toml` workspace members
- Verify dependency directions match ARCHITECTURE.md rules
- Flag violations

#### contract-drift
- Run `gen-contracts`
- Check `git diff --exit-code`
- Flag any drift

#### topology-check
- Verify all topologies reference valid deployables
- Verify all deployables have required resources

**Verification**:
```bash
cargo build -p model-lint
cargo run -p model-lint

cargo build -p dependency-graph
cargo run -p dependency-graph

just validate-platform
```

**Risks**: Medium - first time implementing validators

---

### 1.4 Wire Up Existing Services (Priority: MEDIUM)

**Why**: Verify services can actually be called

**Services to wire**:
- [ ] counter-service → web-bff
- [ ] user-service → web-bff

**Files to modify**:
- `servers/bff/web-bff/src/routes/`
- `servers/bff/web-bff/src/handlers/`
- Possibly `servers/bff/web-bff/Cargo.toml` for dependencies

**What to do**:
- [ ] Add route handlers that call service application layer
- [ ] Wire up dependency injection
- [ ] Test with curl/httpie
- [ ] Verify OpenAPI spec matches

**Verification**:
```bash
# Start API
cargo run -p web-bff

# Test counter endpoint
curl http://localhost:3000/api/counter

# Test user endpoint
curl http://localhost:3000/api/users
```

**Risks**: Medium - may expose interface mismatches

---

### 1.5 Fix Generated Artifacts (Priority: MEDIUM)

**Why**: Ensure generation pipeline works

**What to do**:
- [ ] Run `just gen-platform` - verify catalog regenerated
- [ ] Run `just gen-contracts` - verify zero diff
- [ ] Add drift detection to CI (fails if generated files differ)

**Verification**:
```bash
just gen-platform
just gen-contracts
git diff --exit-code  # Must be clean
```

**Risks**: Low - should already work

---

## Verification Suite

Run ALL of these before marking phase complete:

```bash
# Essential checks
cargo check --workspace
cargo clippy --workspace

# Formatting
cargo fmt --all -- --check
cargo fmt --all  # if fixing

# Platform validation
just validate-platform
just gen-platform
just gen-contracts

# Dependency directions
just boundary-check

# Tests
cargo test --workspace

# Git cleanliness
git diff --exit-code
git status  # should be clean after commits
```

---

## Known Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| CI paths still wrong after fix | Low | Medium | Test with actual PR |
| Validators too strict initially | Medium | Low | Start with warnings, escalate later |
| Service interface mismatch | Medium | Medium | May need to adjust interfaces |
| Generated artifacts drift | Low | High | CI will catch this |

---

## Exit Criteria

Phase 1 is complete when ALL of these are true:

- [x] Task checklist 100% complete
- [ ] All verification commands pass
- [ ] No blockers remaining
- [ ] `.refactoring-state.yaml` updated:
  - `status: completed`
  - `completed_at: <today>`
- [ ] Handoff document created: `docs/refactoring/handoffs/handoff-1-to-2.md`
- [ ] Git committed with clear message

---

## Rollback Plan

If something goes wrong:

```bash
# Check what changed
git status
git diff

# Revert if needed
git revert HEAD  # revert last commit
git reset --hard <known-good-commit>  # nuclear option

# Fix issues and retry
```

All changes in this phase are:
- Config file additions (safe)
- Path fixes in CI (safe)
- New validator binaries (additive)
- Service wiring (reversible)

No data loss risk.

---

## Notes & Discoveries

Use this section to capture learnings:

{Agent: Fill this in as you work}

---

## Completion Sign-off

When done, fill in:

- **Completion Date**: 
- **Time Spent**: 
- **Files Modified**: (count)
- **Files Created**: (count)
- **Issues Encountered**: 
- **Resolutions**: 
- **Confidence Level**: HIGH / MEDIUM / LOW
- **Ready for Phase 2**: YES / NO (explain)
