# Handoff Protocol Template

> **Purpose**: Ensure seamless context transfer between agents working on different phases  
> **Location**: `docs/refactoring/handoffs/handoff-N-to-N+1.md`  
> **When to use**: When completing a phase and handing off to the next agent

---

## How to Use This Template

1. Copy this template to `docs/refactoring/handoffs/handoff-{from}-to-{to}.md`
2. Fill in ALL sections before marking a phase complete
3. Next agent MUST read this document before starting work
4. Update any outdated information as you discover it

---

```markdown
# Handoff: Phase {N} → Phase {N+1}

**From Agent**: {agent-id-or-name}
**To Agent**: {agent-id-or-name}
**Date**: {YYYY-MM-DD}
**Phase Duration**: {start-date} to {end-date}

---

## Executive Summary

{2-3 sentences summarizing what was accomplished and what's next}

---

## ✅ Completed Work

### What Was Accomplished
- [ ] {Specific deliverable 1}
- [ ] {Specific deliverable 2}
- [ ] {Specific deliverable 3}

### What Was Verified
- [ ] `{command that was run}` - PASSED
- [ ] `{command that was run}` - PASSED
- [ ] Manual review of {specific area} - CONFIRMED

### Tests Added/Modified
- [ ] {test file} - {what it tests}
- [ ] {test file} - {what it tests}

### Documentation Updated
- [ ] {doc file} - {what changed}
- [ ] {doc file} - {what changed}

---

## ⚠️ Partially Complete / Needs Follow-up

### In-Progress Items
- [ ] {item that's 80% done} - {what remains}
- [ ] {item that needs review} - {why it needs review}

### Known Gaps
- [ ] {area that wasn't fully addressed} - {reason}
- [ ] {area that needs future work} - {context}

### Technical Debt Incurred
- [ ] {shortcut taken} - {why} - {how to fix later}
- [ ] {temporary workaround} - {until what's ready}

---

## 🚧 Blockers & Decisions

### Decisions Made
1. **Decision**: {what was decided}
   - **Context**: {why this decision was needed}
   - **Rationale**: {why this option was chosen}
   - **Trade-offs**: {what was given up}
   - **Reversibility**: {easy/hard to undo}

### Decisions Deferred
1. **Decision**: {what needs to be decided}
   - **Context**: {why it matters}
   - **Options**: {option A} vs {option B}
   - **Recommendation**: {which you'd choose and why}
   - **Who to ask**: {if needs human input}

### Current Blockers
- [ ] {blocker} - {what's needed to unblock}
- [ ] {blocker} - {workaround if any}

### Resolved Blockers
- ~~{previous blocker}~~ - {how it was resolved}

---

## 📋 Next Agent Instructions

### Starting Point
**Exact state to begin from**:
- {specific commit hash or branch}
- {specific directory to start in}
- {specific files to read first}

### First Steps (Do These First)
1. ```bash
   # Run this to verify current state
   {command}
   ```
2. Read these files in order:
   - `docs/refactoring/PROGRESS-ASSESSMENT.md`
   - `docs/refactoring/REFACTORING-ROADMAP.md` (Phase {N+1} section)
   - `{specific file relevant to next phase}`
   - `{another specific file}`

### Verification Commands
Before marking any work complete, run:
```bash
# Essential checks
cargo check --workspace
cargo clippy --workspace

# Phase-specific checks
{specific commands for phase N+1}

# Generate and verify artifacts
just gen-platform
git diff --exit-code
```

### What to Read First (Context)
**Must-read before coding**:
1. `docs/ARCHITECTURE.md` - {specific sections: e.g., "Section 3.6 services"}
2. `AGENTS.md` - {specific rules that apply}
3. `docs/refactoring/tasks/phase-{N+1}.md` - {detailed task list}
4. {specific code files that define current interfaces}

### Files You'll Likely Touch
**High probability of modification**:
- `{file path}` - {why you'll need it}
- `{file path}` - {why you'll need it}
- `{file path}` - {why you'll need it}

### Files to Be Careful With
**High risk / sensitive**:
- `{file path}` - {why it's sensitive} - {what to watch for}
- `{file path}` - {why it's sensitive} - {what to watch for}

---

## 📁 Changed Files Inventory

### Complete List of Modified Files
```
{file path 1} - {brief description of change}
{file path 2} - {brief description of change}
{file path 3} - {brief description of change}
...
```

### New Files Created
```
{file path 1} - {purpose}
{file path 2} - {purpose}
...
```

### Files Deleted (if any)
```
{file path} - {why deleted}
...
```

---

## 🤔 Open Questions

### Questions Needing Answers
1. **Question**: {what needs clarification}
   - **Context**: {why it matters}
   - **Current assumption**: {what you're assuming until clarified}
   - **Impact if wrong**: {what breaks if assumption is wrong}

### Questions You Should Investigate
1. **Question**: {what you didn't have time to research}
   - **Why it matters**: {impact on phase N+1}
   - **Where to start**: {documentation, code, or person to ask}

---

## 🎯 Success Criteria for Phase {N+1}

Phase {N+1} is complete when:
- [ ] {specific criterion 1}
- [ ] {specific criterion 2}
- [ ] {specific criterion 3}
- [ ] All verification commands pass
- [ ] Handoff to Phase {N+2} created

---

## 💡 Lessons Learned

### What Worked Well
- {approach that was effective}
- {tool or pattern that helped}

### What Didn't Work
- {approach that failed} - {why}
- {dead end explored} - {what you learned}

### Tips for Next Agent
- {advice specific to this codebase}
- {gotcha to watch for}
- {pattern to follow}

---

## 🔗 References

### Relevant Documentation
- `{doc link or path}` - {what it covers}
- `{doc link or path}` - {what it covers}

### Relevant Code
- `{file path}` - {why reference it}
- `{file path}` - {why reference it}

### External Resources
- `{URL}` - {what you learned from it}
- `{URL}` - {what you learned from it}

---

## ✍️ Sign-off

**Phase {N} Status**: {COMPLETE | PARTIAL | BLOCKED}

**Confidence Level**: {HIGH | MEDIUM | LOW}

**Notes**: {any final thoughts}

**Next Steps**: {immediate actions for next agent}
```

---

## Handoff Checklist

Before creating handoff document, verify:

- [ ] All task items from phase task card marked complete or moved to "Partially Complete"
- [ ] All verification commands run and passing
- [ ] `.refactoring-state.yaml` updated
- [ ] Git committed with clear message
- [ ] All changed files documented
- [ ] All decisions recorded with rationale
- [ ] All blockers documented with workarounds if possible
- [ ] Open questions listed with current assumptions
- [ ] Next agent instructions specific and actionable
- [ ] Lessons learned captured

---

## Example: Handoff from Phase 1 to Phase 2

```markdown
# Handoff: Phase 1 → Phase 2

**From Agent**: agent-foundation-fixes
**To Agent**: agent-package-structure
**Date**: 2026-04-12

---

## Executive Summary

Phase 1 completed: All CI workflows fixed, missing root files added, platform validators implemented,
and generated artifacts verified. Repository ready for package structure migration.

---

## ✅ Completed Work

### What Was Accomplished
- [x] Fixed all 5 CI workflow files with correct paths
- [x] Added 4 missing root config files (typos.toml, .editorconfig, .cargo/audit.toml, .config/nextest.toml)
- [x] Implemented 6 platform validators in platform/validators/
- [x] Wired counter-service and user-service to web-bff
- [x] Added drift detection to CI pipeline

### What Was Verified
- `cargo check --workspace` - PASSED
- `cargo clippy --workspace` - PASSED (3 warnings in stub code, acceptable)
- `just validate-platform` - PASSED
- `just gen-contracts` - PASSED (zero diff)
- `just boundary-check` - PASSED
- All 5 GitHub workflows trigger on PR - CONFIRMED

### Tests Added/Modified
- `verification/contract/backward-compat/contract.test.ts` - Updated path references
- `verification/topology/single-vps/topology_test.rs` - New test for single VPS topology

### Documentation Updated
- `docs/refactoring/PROGRESS-ASSESSMENT.md` - Created
- `docs/refactoring/REFACTORING-ROADMAP.md` - Created
- `.refactoring-state.yaml` - Created and tracking
```
