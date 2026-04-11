# Agent Prompt — Add Module

## Steps

1. **Read constraints**: Load `agent/codemap.yml` and `agent/boundaries.md`
2. **Copy template**: Duplicate `agent/templates/module/` to `services/<module-name>/`
3. **Fill module structure**:
   - `src/domain/` — Pure domain logic (entities, value objects, domain events)
   - `src/application/` — Use case orchestration (pure functions, port-driven)
   - `src/ports/` — External dependency abstractions (traits)
   - `src/contracts/` — Stable contract definitions (DTOs, errors)
   - `src/sync/` — Sync strategy definition (SyncStrategy, ConflictResolver)
   - `src/lib.rs` — Module entry, re-exports
4. **Update workspace**: Add to root `Cargo.toml` `[workspace.members]`
5. **Update moon.yml**: Add build/test task for new module
6. **Update contracts**: Add DTOs to `packages/contracts/` if exposing externally
7. **Generate SDK**: Run `just gen-frontend-sdk` if frontend needs types
8. **Verify**:
   - `cargo build -p <module-name>` must succeed
   - `cargo test -p <module-name>` must pass
   - `just quality boundary` must have zero violations

## Checklist

- [ ] Module name follows convention (kebab-case)
- [ ] `Cargo.toml` has correct dependencies (no forbidden crates)
- [ ] All `SyncStrategy` declarations present
- [ ] Port traits defined in `src/ports/`
- [ ] No direct adapter imports
- [ ] Tests cover domain + application + sync
- [ ] Contract types added to `packages/contracts/`
- [ ] `agent/codemap.yml` updated if boundaries changed
