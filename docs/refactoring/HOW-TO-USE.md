# Refactoring Guide — How to Use This System

> **For**: Any agent (AI or human) working on this refactoring effort  
> **Read First**: This document, then your phase task card

---

## Quick Start

You've been assigned to refactor this repository to match `docs/ARCHITECTURE.md` (the constitution).

**Do this first**:

1. Read this guide (~5 minutes)
2. Read `docs/refactoring/PROGRESS-ASSESSMENT.md` (~10 minutes)
3. Check `.refactoring-state.yaml` to see current progress
4. Read your assigned phase task card in `docs/refactoring/tasks/phase-N.md`
5. Start working!

---

## Directory Structure

```
docs/refactoring/
├── PROGRESS-ASSESSMENT.md          # Current state analysis (read this first)
├── REFACTORING-ROADMAP.md          # 6-phase plan with details
├── HANDOFF-PROTOCOL.md             # How to hand off between phases
├── HOW-TO-USE.md                   # This file
├── tasks/
│   ├── phase-1-foundation-fixes.md # Detailed task checklist
│   ├── phase-2-package-structure.md
│   ├── phase-3-runtime-workers.md
│   ├── phase-4-services-integration.md
│   ├── phase-5-infrastructure.md
│   └── phase-6-documentation.md
└── handoffs/
    ├── handoff-1-to-2.md           # Created when Phase 1 completes
    ├── handoff-2-to-3.md
    └── ...

.refactoring-state.yaml             # Progress tracker (update this!)
```

---

## The System

### Three-Layer Handoff

1. **State File** (`.refactoring-state.yaml`)
   - Quick progress snapshot
   - MUST update before starting
   - MUST update when completing
   - MUST update when blocked

2. **Phase Task Card** (`docs/refactoring/tasks/phase-N.md`)
   - Detailed checklist for your phase
   - What files to touch
   - What commands to run
   - What to verify

3. **Handoff Document** (`docs/refactoring/handoffs/handoff-N-to-N+1.md`)
   - Created when you finish a phase
   - Context transfer to next agent
   - Lessons learned
   - Instructions for next agent

---

## Working on a Phase

### Step 1: Preparation

```bash
# 1. Read the current state
cat .refactoring-state.yaml

# 2. Read your phase task card
cat docs/refactoring/tasks/phase-N.md

# 3. Read handoff from previous phase (if any)
cat docs/refactoring/handoffs/handoff-N-1-to-N.md
```

### Step 2: Update State

Edit `.refactoring-state.yaml`:

```yaml
current_phase: N
phase_status:
  N_your_phase:
    status: in_progress  # Changed from pending
    started_at: 2026-04-12
    agent_id: your-agent-id
```

### Step 3: Work Through Tasks

- Follow the checklist in your phase task card
- Check off items as you complete them
- Update notes section with discoveries
- If blocked, update state file with blocker details

### Step 4: Verify

Run the verification suite from your task card:

```bash
cargo check --workspace
cargo clippy --workspace
just validate-platform
just boundary-check
# ... (specific to your phase)
```

### Step 5: Complete

1. Fill in completion section of task card
2. Update `.refactoring-state.yaml`:
   ```yaml
   phase_status:
     N_your_phase:
       status: completed
       completed_at: 2026-04-12
   ```
3. Create handoff document using template
4. Commit all changes

---

## Handoff Document Creation

Use the template in `docs/refactoring/HANDOFF-PROTOCOL.md`.

**Critical sections**:
1. ✅ What you completed
2. ⚠️ What needs follow-up
3. 🚧 Decisions made and why
4. 📋 Exact instructions for next agent
5. 📁 Complete file inventory
6. 💡 Lessons learned

**Example**:
```markdown
# Handoff: Phase 1 → Phase 2

## Executive Summary
Fixed all CI workflows, added missing config files, implemented validators.

## Completed
- Fixed 5 CI workflow files
- Added 4 root config files
- Implemented 6 validators

## Next Agent Instructions
1. Read docs/ARCHITECTURE.md Section 3.8 (packages)
2. Start with packages/kernel migration
3. Run cargo check after each package move
```

---

## Common Commands

### Daily Checks
```bash
# Verify build
cargo check --workspace

# Lint
cargo clippy --workspace

# Format check
cargo fmt --all -- --check

# Format fix
cargo fmt --all
```

### Platform Validation
```bash
# Validate platform models
just validate-platform

# Generate catalog
just gen-platform

# Generate contracts
just gen-contracts

# Check for drift
git diff --exit-code
```

### Testing
```bash
# Unit tests
cargo test --workspace

# With nextest
cargo nextest run --workspace

# Coverage
cargo llvm-cov --workspace --html
```

### Dependencies
```bash
# Check dependency directions
just boundary-check

# Audit for vulnerabilities
cargo audit
```

---

## Rules to Follow

### From AGENTS.md (Summary)
1. **Chinese communication** - but code/commands in English
2. **Read before modifying** - understand current state first
3. **Evidence before guessing** - get logs, repro steps
4. **Search before guessing** - check docs/issues
5. **Small changes first** - minimal viable change
6. **Explain before modifying** - understand what current code protects
7. **Don't fake success** - actually run verification
8. **Acknowledge uncertainty** - document what you don't know
9. **Don't bypass problems** - fix them properly

### From ARCHITECTURE.md (Key Rules)
1. **Services are libraries, not processes**
2. **Platform model is truth source**
3. **Contracts are truth source**
4. **Workers are first-class**
5. **Vendors only in adapters**
6. **Generated files are read-only**
7. **No cross-service dependencies**

---

## If You Get Stuck

### 1. Check State
```bash
cat .refactoring-state.yaml
```
Is someone else working? Are there known blockers?

### 2. Read Documentation
- `docs/ARCHITECTURE.md` - what should it look like?
- `AGENTS.md` - what are the working agreements?
- `docs/refactoring/tasks/phase-N.md` - what's your task?

### 3. Search Codebase
```bash
# Find references to X
grep_search "pattern"

# Find files
glob "**/pattern"

# Explore with agent
agent: "Find all implementations of X"
```

### 4. Update State with Blocker
If truly stuck, update `.refactoring-state.yaml`:

```yaml
phase_status:
  N_your_phase:
    status: blocked
    blockers:
      - "Description of blocker"
    notes: "What I've tried, what's needed"
```

---

## Quality Gates

### Before Committing
- [ ] Code compiles
- [ ] Tests pass
- [ ] Linting passes
- [ ] Formatting correct
- [ ] No debug code left in

### Before Completing Phase
- [ ] All task items done
- [ ] Verification suite passes
- [ ] State file updated
- [ ] Handoff document created
- [ ] Git committed cleanly

### Before Handing Off
- [ ] Handoff document complete
- [ ] Next agent instructions specific
- [ ] All decisions recorded with rationale
- [ ] All blockers documented
- [ ] File inventory accurate

---

## Tips for Success

### 1. Work Incrementally
- Small commits
- Verify after each
- Easy to revert if wrong

### 2. Document as You Go
- Don't wait until end
- Capture decisions immediately
- Notes section in task cards exists for this

### 3. Test Frequently
- Run verification often
- Catch problems early
- Don't batch up verification

### 4. Ask Questions
- If uncertain, ask user
- Better to clarify than assume
- Update docs with answers

### 5. Follow Patterns
- Look at existing code
- Match style and structure
- Don't introduce new patterns without reason

---

## Phase Overview

| Phase | Focus | Risk | Dependencies |
|-------|-------|------|--------------|
| 1 | Foundation Fixes | LOW | None |
| 2 | Package Structure | MEDIUM | Phase 1 |
| 3 | Runtime & Workers | MEDIUM-HIGH | Phase 2 |
| 4 | Services & Integration | MEDIUM | Phase 3 |
| 5 | Infrastructure | LOW-MEDIUM | Phase 4 |
| 6 | Documentation | LOW | Phase 5 (or Phase 2) |

---

## Emergency Procedures

### If You Break the Build
```bash
# Revert your last commit
git revert HEAD

# Or reset to known good state
git log --oneline  # find last good commit
git reset --hard <commit>
```

### If State File Corrupted
```bash
# Check git history
git log -- .refactoring-state.yaml
git show <commit>:.refactoring-state.yaml
```

### If Handoff Missing
- Read previous phase task card
- Infer from git history what was done
- Ask user for clarification
- Proceed with caution

---

## Contact & Communication

### Between Agents
- Use handoff documents
- Update state file
- Leave clear instructions

### With User
- Ask questions when uncertain
- Report blockers immediately
- Provide progress updates
- Clarify ambiguous requirements

---

## Success Metrics

Phase success = all exit criteria met + verification passing + handoff complete

Overall success = all 6 phases complete + repository matches ARCHITECTURE.md

---

## Revision History

| Date | Change | Author |
|------|--------|--------|
| 2026-04-12 | Initial creation | Initial assessment agent |
