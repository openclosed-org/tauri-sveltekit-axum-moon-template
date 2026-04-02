---
phase: 07-multi-tenant-data-isolation
plan: 01
subsystem: database
tags: [surrealdb, tenant-isolation, rust, domain-types, schema-migration]

# Dependency graph
requires:
  - phase: 05-database-infrastructure
    provides: SurrealDB connection, AppState, SurrealDbPort trait
provides:
  - TenantId newtype in domain crate (importable from middleware)
  - TenantAwareSurrealDb wrapper with automatic SQL filter injection
  - SurrealDB tenant + user_tenant table schema migrations
affects:
  - 07-02 (tenant middleware wiring)
  - 07-03 (tenant initialization API)

# Tech tracking
tech-stack:
  added: [jsonwebtoken, chrono, async-trait]
  patterns:
    - Port trait pattern: SurrealDbPort stays unchanged, isolation in impl layer
    - SQL injection via string rewriting: WHERE/AND tenant_id=$tenant_id
    - serde_json::Value as SurrealDB→Rust bridge (avoids coupling trait to SurrealValue)

key-files:
  created:
    - crates/domain/src/ports/mod.rs
    - crates/runtime_server/src/ports/mod.rs
    - crates/runtime_server/src/ports/surreal_db.rs
  modified:
    - crates/domain/src/lib.rs
    - crates/domain/src/ports/surreal_db.rs
    - crates/runtime_server/src/lib.rs
    - crates/runtime_server/Cargo.toml

key-decisions:
  - "SurrealDbPort trait stays unchanged (D-11): isolation logic in TenantAwareSurrealDb impl layer"
  - "serde_json::Value as intermediate type: avoids coupling trait to surrealdb::types::SurrealValue"
  - "inject_tenant_filter uses string rewriting: SELECT→WHERE/AND, CREATE→SET append, UPDATE/DELETE→WHERE/AND"
  - "tenant_id field uses String type in Value (not Strand): surrealdb 3.x API compat"

patterns-established:
  - "SQL filter injection: case-insensitive WHERE detection, clause-aware insertion"
  - "Domain newtypes in ports/mod.rs: TenantId, future types here"
  - "Runtime ports in runtime_server/src/ports/: SurrealDB impl wrappers"

requirements-completed: [TENANT-01]

# Metrics
duration: 35min
completed: 2026-03-29
---

# Phase 07 Plan 01: Multi-Tenant Infrastructure Summary

**TenantId newtype in domain crate + TenantAwareSurrealDb wrapper with automatic SQL filter injection for multi-tenant data isolation**

## Performance

- **Duration:** 35 min
- **Started:** 2026-03-29T16:00:00Z
- **Completed:** 2026-03-29T16:35:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- TenantId newtype in domain::ports with Display + Clone + Eq + Hash
- TenantAwareSurrealDb struct implementing SurrealDbPort with automatic tenant_id injection
- SQL filter injection for SELECT/CREATE/UPDATE/DELETE with WHERE clause handling
- run_tenant_migrations function defining tenant + user_tenant tables with indexes
- 7 unit tests passing for SQL injection logic
- Fixed pre-existing surrealdb 3.x API compat issues (sql→types module, SurrealValue bound)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create TenantId newtype in domain crate** - `f4b30f1` (feat)
2. **Task 2: Create TenantAwareSurrealDb wrapper + schema migration** - `1ee5ca6` (feat)

## Files Created/Modified
- `crates/domain/src/ports/mod.rs` - TenantId newtype definition (created)
- `crates/domain/src/lib.rs` - Refactored to use ports module file (modified)
- `crates/domain/src/ports/surreal_db.rs` - Fixed surrealdb::sql→types, added SurrealValue bound (modified)
- `crates/runtime_server/src/ports/mod.rs` - Port implementations module (created)
- `crates/runtime_server/src/ports/surreal_db.rs` - TenantAwareSurrealDb + tests (created)
- `crates/runtime_server/src/lib.rs` - Added pub mod ports (modified)
- `crates/runtime_server/Cargo.toml` - Added jsonwebtoken, chrono, async-trait (modified)

## Decisions Made
- SurrealDbPort trait unchanged (D-11) — all isolation logic in TenantAwareSurrealDb impl layer
- serde_json::Value as intermediate type to avoid coupling trait to surrealdb::types::SurrealValue
- SQL filter injection via string rewriting (not SurrealDB SCOPE/PERMISSIONS)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed surrealdb::sql::Value → surrealdb::types::Value**
- **Found during:** Task 1
- **Issue:** Pre-existing code used `surrealdb::sql::Value` which doesn't exist in surrealdb 3.x
- **Fix:** Changed to `surrealdb::types::Value` in domain/src/ports/surreal_db.rs
- **Files modified:** crates/domain/src/ports/surreal_db.rs
- **Verification:** cargo check -p domain passes
- **Committed in:** f4b30f1 (Task 1 commit)

**2. [Rule 3 - Blocking] SurrealValue trait bound conflict with async_trait**
- **Found during:** Task 2
- **Issue:** surrealdb 3.x `take()` requires `SurrealValue` trait, but adding it to domain trait caused lifetime mismatch with async_trait
- **Fix:** Used `serde_json::Value` as intermediate type (implements SurrealValue), then serde_json::from_value to deserialize
- **Files modified:** crates/runtime_server/src/ports/surreal_db.rs
- **Verification:** cargo test -p runtime_server passes (7/7)
- **Committed in:** 1ee5ca6 (Task 2 commit)

**3. [Rule 1 - Bug] SQL filter injection: WHERE clause placement**
- **Found during:** Task 2
- **Issue:** Case-insensitive WHERE search produced wrong byte positions when uppercased string differs from original
- **Fix:** Search for WHERE in uppercase for case-insensitive match, use position to split original SQL at WHERE keyword boundary
- **Files modified:** crates/runtime_server/src/ports/surreal_db.rs
- **Verification:** 7 unit tests pass
- **Committed in:** 1ee5ca6 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All fixes essential for surrealdb 3.x compatibility. No scope creep.

## Issues Encountered
- surrealdb 3.x API changes: `sql` module removed, `SurrealValue` required for `take()`, `Value::from(String)` not implemented
- SQL string position mismatch between uppercase search and original string

## Next Phase Readiness
- TenantId importable from domain::ports — ready for middleware to inject
- TenantAwareSurrealDb ready for wiring in create_router()
- run_tenant_migrations available for startup schema setup
- Plan 07-02 (tenant middleware) can proceed

---
*Phase: 07-multi-tenant-data-isolation*
*Completed: 2026-03-29*
