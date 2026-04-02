---
phase: 03-runtime-boundary-convergence
plan: '03'
subsystem: adapters
tags: [tauri, commands, runtime, adapter, boundary]

requires:
  - phase: 03-runtime-boundary-convergence
    provides: storage adapter crates (storage_libsql, storage_surrealdb)
  - phase: 03-runtime-boundary-convergence
    provides: host crate refactoring (native-tauri imports from adapters)
provides:
  - runtime_tauri with auth and config command modules
  - native-tauri as thin bootstrap importing commands from runtime_tauri
  - Clean host-adapter boundary: Tauri command handlers live in runtime_tauri

affects: [Phase 04 features that need Tauri commands]

tech-stack:
  added: []
  patterns: [host-adapter-command-bridge, thin-bootstrap-host]

key-files:
  created:
    - packages/adapters/hosts/tauri/src/commands/mod.rs
    - packages/adapters/hosts/tauri/src/commands/auth.rs
    - packages/adapters/hosts/tauri/src/commands/config.rs
  modified:
    - packages/adapters/hosts/tauri/Cargo.toml
    - packages/adapters/hosts/tauri/src/lib.rs
    - apps/client/native/src-tauri/src/lib.rs
    - apps/client/native/src-tauri/src/commands/mod.rs
    - apps/client/native/src-tauri/Cargo.toml
    - Cargo.toml (workspace deps)

key-decisions:
  - "tauri-plugin-opener added to workspace deps (was direct version in native-tauri only)"
  - "Sync commands stay in native-tauri until SyncEngine moves to core (Phase 4+)"
  - "runtime_tauri now depends on domain + usecases + tauri plugins (not contracts_api)"

patterns-established:
  - "Host adapter pattern: runtime_tauri bridges Tauri commands to usecases, host crate is pure bootstrap"
  - "Dependency direction: host → runtime_tauri → domain/usecases (no host deps in runtime_tauri)"

requirements-completed: [RUNTIME-02]

duration: 8min
completed: 2026-04-02
---

# Phase 03 Plan 03: Tauri Command Bridge Summary

**Moved Tauri command handlers (auth, config) from native-tauri to runtime_tauri adapter crate; native-tauri reduced to thin bootstrap that imports commands from runtime_tauri**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-02
- **Completed:** 2026-04-02
- **Tasks:** 2
- **Files modified:** 11 (5 created, 6 modified, 2 deleted)

## Accomplishments
- runtime_tauri now exports auth and config command modules (422 + 49 lines)
- native-tauri imports commands from runtime_tauri via `use runtime_tauri::commands::{auth, config}`
- native-tauri/src/commands/ reduced to sync.rs only (sync commands deferred to Phase 4+)
- Workspace deps updated: tauri-plugin-opener added, runtime_tauri added as internal crate
- `cargo check --workspace` passes cleanly

## Task Commits

1. **Task 1: Create command modules in runtime_tauri** - `1ebce43` (feat)
2. **Task 2: Refactor native-tauri to import commands from runtime_tauri** - `0d2af64` (feat)

**Plan metadata:** (pending)

## Files Created/Modified
- `packages/adapters/hosts/tauri/Cargo.toml` - Updated deps (removed contracts_api, added tauri plugins, reqwest, serde, etc.)
- `packages/adapters/hosts/tauri/src/lib.rs` - Exports commands module
- `packages/adapters/hosts/tauri/src/commands/mod.rs` - Declares auth and config modules
- `packages/adapters/hosts/tauri/src/commands/auth.rs` - Migrated from native-tauri (422 lines)
- `packages/adapters/hosts/tauri/src/commands/config.rs` - Migrated from native-tauri (49 lines)
- `apps/client/native/src-tauri/src/lib.rs` - Imports from runtime_tauri, updated generate_handler!
- `apps/client/native/src-tauri/src/commands/mod.rs` - Reduced to `pub mod sync;`
- `apps/client/native/src-tauri/Cargo.toml` - Added runtime_tauri dep
- `Cargo.toml` - Added tauri-plugin-opener and runtime_tauri to workspace deps

## Decisions Made
- Sync commands (sync_start, sync_stop, etc.) stay in native-tauri because SyncEngine is defined there — moving requires Phase 4+ core migration
- tauri-plugin-opener promoted to workspace dep (was direct `"2"` in native-tauri)
- runtime_tauri no longer depends on contracts_api (not needed for command bridge)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - both tasks compiled and verified on first attempt.

## Next Phase Readiness
- runtime_tauri command bridge complete — auth and config accessible from any host crate
- Sync commands remain in native-tauri until SyncEngine migrates to core
- Ready for Phase 03-04 or Phase 04 feature development

---
*Phase: 03-runtime-boundary-convergence*
*Completed: 2026-04-02*
