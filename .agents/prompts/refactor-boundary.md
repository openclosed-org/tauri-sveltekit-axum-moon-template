# Refactor Boundary Prompt

## Purpose

Guide an agent through boundary refactoring — moving implementations to their correct architectural layer, eliminating cross-layer dependencies, and restoring hexagonal architecture compliance.

## When to Use

When code review or CI detects import violations, business logic leaked into wrong layers, or architectural drift after incremental changes.

---

## Prerequisites

- [ ] Read `.agents/rubrics/boundary-compliance.md` — this is the authoritative rule set
- [ ] Read `AGENTS.md` for execution protocol
- [ ] Run `just verify` to capture current baseline
- [ ] Identify the specific violation(s) triggering this refactor

---

## Steps

### 1. Identify Violation

Use grep or IDE to find the offending import:

```bash
# Check for cross-layer imports
rg "use (domain|usecases|adapters|contracts_|servers)" --type rust
```

Determine:
- **What** is being imported (trait, struct, function)
- **From where** (source layer)
- **Into where** (target layer — this is the violating layer)

### 2. Determine Correct Ownership

Per `.agents/rubrics/boundary-compliance.md`:

| Code belongs in | If it is... |
|---|---|
| `domain` | A port trait, value object, or error type |
| `usecases` | A service trait or business logic implementation |
| `contracts` | A cross-boundary DTO (no logic) |
| `adapters/storage` | A database/repository implementation |
| `adapters/hosts` | A Tauri command handler |
| `servers` | An HTTP route handler |

### 3. Plan the Migration

- [ ] Identify the interface that must remain stable (trait signature, function signature)
- [ ] Identify all consumers of the code being moved
- [ ] Plan import path updates for each consumer
- [ ] Plan dependency (Cargo.toml) changes if needed

### 4. Execute the Move

1. **Copy** the implementation to its correct location (don't delete yet)
2. **Update** the import paths in all consumers
3. **Update** `Cargo.toml` dependencies if the moved code is in a different crate
4. **Update** `mod.rs` files to export from new location
5. **Delete** the old implementation

### 5. Verify

```bash
# Full quality check
just verify

# Rust tests
cargo test

# Boundary compliance — review against rubric
```

---

## Verification Checklist

Per `.agents/rubrics/boundary-compliance.md`:

- [ ] No cross-layer imports violating the layer rules
- [ ] All trait implementations reference domain port traits
- [ ] usecases does NOT import contracts_api types
- [ ] Command handlers delegate to usecases
- [ ] No business logic in server route handlers
- [ ] `just verify` passes
- [ ] All tests pass

---

## Rollback

If the refactor breaks something:

```bash
# Uncommitted changes
git checkout -- .

# Already committed
git revert HEAD
```

---

## Key Principle

**Keep the interface stable, move only the implementation.** The trait signature, function signatures, and public API should not change — only the file location and import paths change.

---

## References

- `.agents/rubrics/boundary-compliance.md` — Authoritative layer rules
- `.agents/rubrics/code-review.md` — Code quality checklist
- `.agents/playbooks/create-feature.md` — Correct layer structure reference
