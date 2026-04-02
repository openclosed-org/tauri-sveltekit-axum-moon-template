---
phase: 07-multi-tenant-data-isolation
plan: 03
subsystem: api
tags: [tenant, surrealdb, axum, schema-migration, multi-tenancy]

# Dependency graph
requires:
  - phase: 07-02
    provides: "TenantAwareSurrealDb wrapper, tenant extraction middleware, TenantId newtype"
  - phase: 07-01
    provides: "TenantId newtype, SurrealDbPort trait, run_tenant_migrations()"
provides:
  - POST /api/tenant/init endpoint for auto-creating tenants on first login
  - AppState::new_dev() runs tenant schema migrations automatically
  - Tenant-scoped API routes with JWT middleware applied via route_layer
  - api_router() function separating public and authenticated route groups
affects:
  - phase: 08 (desktop native features — will use tenant-scoped routes)
  - phase: 06 (Google OAuth — login flow calls /api/tenant/init)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Admin-mode TenantAwareSurrealDb for cross-tenant queries"
    - "RecordId::parse_simple for SurrealDB record reference binding"
    - "route_layer pattern for selective middleware on API routes"
    - "record_id_to_string helper for RecordId → table:key formatting"

key-files:
  created: []
  modified:
    - crates/runtime_server/src/routes/tenant.rs
    - crates/runtime_server/src/routes/mod.rs
    - crates/runtime_server/src/lib.rs
    - crates/runtime_server/src/state.rs

key-decisions:
  - "Used RecordId::parse_simple + Value::RecordId for tenant_id binding (record<tenant> schema requires record type, not string)"
  - "Created record_id_to_string helper to format RecordId as table:key since surrealdb 3.x RecordId lacks Display trait"
  - "Used Value::String(String::clone()) instead of Value::from(&str) — surrealdb 3.x types don't implement From<&str>"
  - "Applied tenant middleware as route_layer on api_router() only, keeping health checks public"

patterns-established:
  - "Admin-mode DB for cross-tenant operations (TenantAwareSurrealDb::new_admin)"
  - "Separate public_routes and api_router groups in create_router()"
  - "Tenant schema migrations run in AppState::new_dev() for dev/test environments"

requirements-completed: [TENANT-03]

# Metrics
duration: 12min
completed: 2026-03-29
---

# Phase 07 Plan 03: Tenant Initialization API Summary

**POST /api/tenant/init endpoint that auto-creates tenant + user_tenant binding on first Google login, returns existing tenant_id on subsequent calls, with schema migrations running on AppState startup**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-29T11:48:53Z
- **Completed:** 2026-03-29T12:01:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- POST /api/tenant/init endpoint: creates tenant + user_tenant on first login, returns existing on subsequent calls
- AppState::new_dev() runs `run_tenant_migrations()` automatically on startup
- api_router() function separates tenant-scoped API routes from public health routes
- create_router() wires tenant middleware as route_layer on API routes only
- 13 tests passing (6 tenant-related + 7 existing), cargo check clean

## Task Commits

1. **Task 1: Create ensure_tenant API endpoint** - `abbdc0e` (feat)
2. **Task 2: Wire tenant module + run migrations on AppState init** - `a950bbf` (feat)

## Files Created/Modified
- `crates/runtime_server/src/routes/tenant.rs` - POST /api/tenant/init endpoint with init_tenant handler, request/response types, 3 unit tests
- `crates/runtime_server/src/routes/mod.rs` - Added api_router() for tenant-scoped routes
- `crates/runtime_server/src/lib.rs` - Wired tenant middleware as route_layer on api_router, separate public/api route groups
- `crates/runtime_server/src/state.rs` - Added run_tenant_migrations() call in new_dev()

## Decisions Made
- Used `RecordId::parse_simple` + `Value::RecordId` for tenant_id binding — SurrealDB schema defines `record<tenant>` type which requires a proper record reference, not a string
- Created `record_id_to_string` helper — surrealdb 3.x `RecordId` and `RecordIdKey` lack `Display` trait, so manual formatting extracts table name + key
- Used `Value::String(String::clone())` — surrealdb 3.x types don't implement `From<&str>`, must construct String variants directly
- Error mapping with `format!()` — `run_tenant_migrations` returns `Box<dyn Error + Send + Sync>` which can't convert to `Box<dyn Error>` via `?` operator

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] surrealdb 3.x API changes from plan template**
- **Found during:** Task 1 (tenant endpoint implementation)
- **Issue:** Plan template used `surrealdb::sql::Thing` and `surrealdb::sql::Value::from(&str)` — both invalid in surrealdb 3.x (`Thing` renamed to `RecordId`, no `From<&str>` for Value)
- **Fix:** Used `surrealdb::types::RecordId` + `RecordIdKey` for struct fields; `Value::String(String::clone())` for parameter binding; `RecordId::parse_simple` for record reference construction
- **Files modified:** crates/runtime_server/src/routes/tenant.rs
- **Verification:** cargo test passes, 6/6 tenant tests green
- **Committed in:** abbdc0e (Task 1 commit)

**2. [Rule 3 - Blocking] Error type mismatch in state.rs**
- **Found during:** Task 2 (AppState migration wiring)
- **Issue:** `run_tenant_migrations` returns `Box<dyn Error + Send + Sync>` but `new_dev()` returns `Result<_, Box<dyn Error>>` — `?` operator can't convert between them due to `Sized` constraint
- **Fix:** Used `.map_err(|e| format!("Tenant migration failed: {e}"))` to convert to String error
- **Files modified:** crates/runtime_server/src/state.rs
- **Verification:** cargo test passes, 13/13 tests green
- **Committed in:** a950bbf (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking — API compatibility)
**Impact on plan:** Both deviations required for surrealdb 3.x compatibility. No scope creep.

## Issues Encountered
- RecordId/RecordIdKey lack Display trait in surrealdb 3.x — resolved with helper function matching on enum variants
- TenantAwareSurrealDb::query trait method requires SurrealDbPort import — compiler error caught and fixed

## Next Phase Readiness
- Tenant initialization API is complete and tested
- Ready for Phase 06 Google OAuth integration (login flow → call /api/tenant/init)
- Phase 08 desktop features can use tenant-scoped routes

---

*Phase: 07-multi-tenant-data-isolation*
*Completed: 2026-03-29*
