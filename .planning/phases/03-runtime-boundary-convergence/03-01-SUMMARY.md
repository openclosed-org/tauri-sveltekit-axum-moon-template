---
phase: 03-runtime-boundary-convergence
plan: '01'
subsystem: architecture
tags: [hexagonal-ports, surrealdb, libsql, adapter-crate, clean-architecture]

requires:
  - phase: 02-contracts-typegen
    provides: domain port traits (SurrealDbPort, LibSqlPort)

provides:
  - Independent storage_surrealdb adapter crate with TenantAwareSurrealDb
  - Independent storage_libsql adapter crate with TursoDb + EmbeddedLibSql
  - Host crates (servers/api, native-tauri) depend on adapters, not vice versa
  - Old port code deleted from host crates

affects:
  - Phase 04 (minimal-feature-implementation) — counter and auth features will use adapter crates

tech-stack:
  added:
    - storage_surrealdb crate (packages/adapters/storage/surrealdb)
    - storage_libsql crate (packages/adapters/storage/libsql)
  patterns:
    - Adapter crates depend on domain traits, host crates depend on adapters
    - Tauri-dependent methods (new_app_data) excluded from adapter, kept in host
    - Workspace dependency declarations for async-trait

key-files:
  created:
    - packages/adapters/storage/surrealdb/Cargo.toml
    - packages/adapters/storage/surrealdb/src/lib.rs
    - packages/adapters/storage/libsql/Cargo.toml
    - packages/adapters/storage/libsql/src/lib.rs
    - packages/adapters/storage/libsql/src/remote.rs
    - packages/adapters/storage/libsql/src/embedded.rs
  modified:
    - Cargo.toml (workspace members + deps)
    - servers/api/Cargo.toml (adapter deps, removed libsql direct)
    - servers/api/src/lib.rs (removed mod ports)
    - servers/api/src/routes/tenant.rs (import from storage_surrealdb)
    - servers/api/src/state.rs (imports from adapter crates)
    - apps/client/native/src-tauri/Cargo.toml (storage_libsql dep, removed libsql)
    - apps/client/native/src-tauri/src/lib.rs (import from storage_libsql)
    - apps/client/native/src-tauri/src/sync/engine.rs (import from storage_libsql)
  deleted:
    - servers/api/src/ports/mod.rs
    - servers/api/src/ports/surreal_db.rs
    - servers/api/src/ports/turso_db.rs
    - apps/client/native/src-tauri/src/ports/mod.rs
    - apps/client/native/src-tauri/src/ports/lib_sql.rs

key-decisions:
  - "Adapter crates depend on domain traits, not the other way around — hexagonal architecture principle"
  - "new_app_data method excluded from EmbeddedLibSql adapter (depends on tauri::AppHandle) — stays host-specific"
  - "servers/api keeps surrealdb direct dep (state.rs uses Surreal<Any> type directly)"
  - "async-trait added to workspace dependencies (used by multiple crates)"
  - "storage_libsql split into remote.rs (TursoDb) and embedded.rs (EmbeddedLibSql) modules"

patterns-established:
  - "Adapter crate pattern: one crate per storage technology, multiple implementations per crate"
  - "Host-to-adapter import via workspace path dependencies"

requirements-completed:
  - RUNTIME-01
  - RUNTIME-02

duration: 15min
completed: 2026-04-02
---

# Phase 03 Plan 01: Runtime Boundary Convergence — Storage Adapter Migration Summary

**Migrated TenantAwareSurrealDb, TursoDb, and EmbeddedLibSql from host crates into two independent adapter crates (storage_surrealdb, storage_libsql), enabling hexagonal architecture where hosts depend on adapters.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-02T06:37:03Z
- **Completed:** 2026-04-02T06:52:00Z
- **Tasks:** 3
- **Files modified:** 15 (6 created, 9 modified, 5 deleted)

## Accomplishments
- Created `storage_surrealdb` adapter crate with TenantAwareSurrealDb (360 lines, 14 tests)
- Created `storage_libsql` adapter crate with TursoDb (129 lines, 1 test) + EmbeddedLibSql (256 lines, 7 tests)
- Updated both host crates (servers/api, native-tauri) to import from adapter crates
- Deleted all old port code from host crates
- Workspace compiles with 0 errors

## Task Commits

1. **Task 1: Create storage_surrealdb adapter crate** - `e46fa11` (feat)
2. **Task 2: Create storage_libsql adapter crate** - `7ace843` (feat)
3. **Task 3: Update host crates, delete old port code** - `76e0ffa` (refactor)

**Plan metadata:** Pending (this commit)

## Files Created/Modified
- `packages/adapters/storage/surrealdb/Cargo.toml` — SurrealDB adapter crate manifest
- `packages/adapters/storage/surrealdb/src/lib.rs` — TenantAwareSurrealDb + 14 tests (360 lines)
- `packages/adapters/storage/libsql/Cargo.toml` — LibSQL adapter crate manifest
- `packages/adapters/storage/libsql/src/lib.rs` — Module declarations + re-exports
- `packages/adapters/storage/libsql/src/remote.rs` — TursoDb + 1 test (129 lines)
- `packages/adapters/storage/libsql/src/embedded.rs` — EmbeddedLibSql + 7 tests (256 lines)
- `Cargo.toml` — Added 2 workspace members, 2 workspace deps, async-trait
- `servers/api/Cargo.toml` — Added adapter deps, removed direct libsql
- `servers/api/src/lib.rs` — Removed `pub mod ports`
- `servers/api/src/routes/tenant.rs` — Import from `storage_surrealdb`
- `servers/api/src/state.rs` — Imports from adapter crates
- `apps/client/native/src-tauri/Cargo.toml` — Replaced libsql with storage_libsql
- `apps/client/native/src-tauri/src/lib.rs` — Import from `storage_libsql`
- `apps/client/native/src-tauri/src/sync/engine.rs` — Import from `storage_libsql`

## Decisions Made
- Adapter crates depend on domain traits, host crates depend on adapters (hexagonal architecture)
- `new_app_data` excluded from EmbeddedLibSql adapter (tauri dependency stays host-specific)
- servers/api keeps `surrealdb` direct dep because `state.rs` uses `Surreal<Any>` type
- `async-trait` added to workspace dependencies (shared by multiple crates)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added async-trait to workspace dependencies**
- **Found during:** Task 1 (storage_surrealdb compilation)
- **Issue:** `async-trait` not in workspace dependencies, but Cargo.toml used `{ workspace = true }`
- **Fix:** Added `async-trait = "0.1"` to root Cargo.toml workspace.dependencies
- **Files modified:** Cargo.toml
- **Verification:** storage_surrealdb compiles after fix

**2. [Rule 3 - Blocking] Fixed sync/engine.rs import from deleted ports module**
- **Found during:** Task 3 (workspace compilation)
- **Issue:** `sync/engine.rs` still imported `crate::ports::lib_sql::EmbeddedLibSql` after ports directory deleted
- **Fix:** Changed import to `storage_libsql::EmbeddedLibSql`
- **Files modified:** apps/client/native/src-tauri/src/sync/engine.rs
- **Verification:** cargo check --workspace passes with 0 errors

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were import/dependency issues caught during compilation. No scope creep.

## Issues Encountered
None - all compilation issues resolved via deviation rules.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Adapter crates are independent and compile standalone
- Host crates import adapters via workspace path dependencies
- Ready for Phase 03 Plan 02 (next runtime boundary convergence plan)

---
*Phase: 03-runtime-boundary-convergence*
*Completed: 2026-04-02*
