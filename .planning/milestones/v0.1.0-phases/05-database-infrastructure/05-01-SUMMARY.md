---
phase: 05-database-infrastructure
plan: 01
subsystem: database
tags: [surrealdb, libsql, async-trait, port-pattern, clean-architecture]

requires:
  - phase: 04-backend-dependencies-build-optimization
    provides: Axum middleware stack, runtime_server skeleton, Cargo.toml workspace deps

provides:
  - SurrealDbPort trait definition (health_check + SurrealQL query)
  - LibSqlPort trait definition (health_check + execute + query)
  - Phase 5 workspace dependencies activated (libsql, moka, quinn, h3, rcgen, rusqlite_migration)
  - runtime_server updated with all Phase 5 deps (surrealdb, moka, reqwest, quinn, h3, rcgen, application)

affects:
  - 05-02-PLAN (AppState depends on these traits)
  - 05-03-PLAN (libsql plugin + h3 transport)
  - Phase 6 (auth will use these Port traits)

tech-stack:
  added: [async-trait, surrealdb (domain crate), libsql, rusqlite_migration, moka, quinn, h3, rcgen]
  patterns: [trait-per-DB port pattern, workspace dependency management]

key-files:
  created:
    - crates/domain/src/ports/surreal_db.rs
    - crates/domain/src/ports/lib_sql.rs
  modified:
    - crates/domain/src/lib.rs
    - crates/domain/Cargo.toml
    - Cargo.toml (workspace deps)
    - crates/runtime_server/Cargo.toml

key-decisions:
  - "D-05/D-06: trait-per-DB pattern over enum-based unified adapter — SurrealDB uses SurrealQL (graph queries, record IDs), libsql uses SQLite SQL, separate traits make the abstraction honest"
  - "Added surrealdb to domain crate deps — pragmatic choice to use surrealdb::sql::Value in SurrealDbPort trait signature"
  - "D-16: Removed redis, rathole, vector from Cargo.toml — pure Rust stack with Moka cache replaces Redis"

patterns-established:
  - "Port trait pattern: domain crate defines traits, runtime crates implement them"
  - "Workspace deps: all shared deps managed in root Cargo.toml, crates reference workspace = true"

requirements-completed: [INFRA-01, INFRA-02]

duration: 8min
completed: 2026-03-29
---

# Phase 05 Plan 01: Domain Port Traits & Workspace Dependencies Summary

**Trait-per-DB port layer with SurrealDbPort and LibSqlPort trait definitions, plus Phase 5 workspace dependency activation (libsql, moka, quinn, h3, rcgen)**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-03-29T01:52:10Z
- **Completed:** 2026-03-29T02:00:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Defined `SurrealDbPort` trait with `health_check()` and `query()` (SurrealQL vars) in `crates/domain/src/ports/surreal_db.rs`
- Defined `LibSqlPort` trait with `health_check()`, `execute()`, and `query()` in `crates/domain/src/ports/lib_sql.rs`
- Updated `crates/domain/Cargo.toml` with async-trait, serde, surrealdb dependencies
- Activated Phase 5 workspace deps: libsql, rusqlite_migration, quinn, h3, rcgen
- Removed deferred deps from Cargo.toml: redis, rathole, vector (per D-16)
- Updated `crates/runtime_server/Cargo.toml` with quinn, h3, rcgen, application deps

## Task Commits

1. **Task 1: Define SurrealDbPort and LibSqlPort traits** - `18caf60` (feat)
2. **Task 2: Activate Phase 5 workspace dependencies** - Already committed by 05-02 parallel agent (`01b9993`)

## Files Created/Modified
- `crates/domain/src/ports/surreal_db.rs` - SurrealDbPort trait (health_check + query with SurrealQL vars)
- `crates/domain/src/ports/lib_sql.rs` - LibSqlPort trait (health_check + execute + query with SQLite params)
- `crates/domain/src/lib.rs` - Port module declarations
- `crates/domain/Cargo.toml` - Added async-trait, serde, surrealdb deps
- `Cargo.toml` - Activated libsql, rusqlite_migration, quinn, h3, rcgen; removed redis/rathole/vector
- `crates/runtime_server/Cargo.toml` - Added quinn, h3, rcgen, application

## Decisions Made
- SurrealDbPort uses `surrealdb::sql::Value` in trait signature — pragmatic choice to keep type safety for SurrealQL variables, at cost of domain depending on surrealdb
- D-05/D-06 trait-per-DB pattern: separate traits for SurrealDB (SurrealQL) and libsql (SQLite SQL) — honest abstraction over fundamentally different query languages
- Moka replaces Redis (D-10/D-12): pure Rust in-memory cache, zero external dependencies for boilerplate

## Deviations from Plan

**Task 2 overlap with parallel agent:** The 05-02 plan executor committed the exact Cargo.toml changes this plan specified. When I attempted to commit Task 2, `git commit` reported "nothing to add" because the files were already committed. Verification confirmed all required deps are present and correct.

## Verification Results
- ✅ `cargo check -p domain` — Port traits compile
- ✅ `cargo check -p runtime_server` — All new deps resolve and compile
- ✅ `grep -c "workspace = true" crates/runtime_server/Cargo.toml` — 16 matches (≥ 12)
- ✅ `grep "redis|rathole|vector" Cargo.toml` — 0 matches (deferred deps removed)

## Issues Encountered
- `cargo check` timeout on first run (600s) due to heavy surrealdb dependency compilation — resolved on retry (dependencies were being compiled in background)
- Parallel agent overlap on Task 2 — no issue, changes verified identical

## Next Phase Readiness
- Domain Port traits ready for 05-02 (AppState + SurrealDB adapter implementation)
- All workspace deps available for 05-03 (libsql plugin + h3 transport)
- `application` crate dependency added to runtime_server for Clean Architecture wiring

## Self-Check: PASSED

- ✅ `crates/domain/src/ports/surreal_db.rs` exists
- ✅ `crates/domain/src/ports/lib_sql.rs` exists
- ✅ Commit `18caf60` (feat(05-01)) exists
- ✅ Commit `01b9993` (feat(05-02), includes Task 2 changes) exists
- ✅ `cargo check -p domain` passes
- ✅ `cargo check -p runtime_server` passes
- ✅ 16 `workspace = true` refs in runtime_server (≥ 12)
- ✅ redis/rathole/vector removed from root Cargo.toml

---

*Phase: 05-database-infrastructure*
*Completed: 2026-03-29*
