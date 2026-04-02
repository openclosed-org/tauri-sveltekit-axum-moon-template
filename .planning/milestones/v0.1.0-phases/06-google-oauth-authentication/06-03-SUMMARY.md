---
phase: 06-google-oauth-authentication
plan: 03
subsystem: auth
tags: [tokio, background-timer, token-refresh, tauri-events]

# Dependency graph
requires:
  - phase: 06-google-oauth-authentication
    provides: Auth commands with token storage (start_oauth, handle_oauth_callback)
provides:
  - Background token auto-refresh 5 min before expiry
  - auth:expired event emission on refresh failure
  - Frontend listener for auth:expired
affects:
  - 06-05 (setup hook already wired, root layout uses initAuthListeners)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Background async timer via tauri::async_runtime::spawn + tokio::time::sleep
    - Recursive refresh scheduling (chain next refresh after successful one)
    - Tauri event emission for cross-layer notification (Rust → Svelte)

key-files:
  created: []
  modified:
    - "apps/desktop-ui/src-tauri/src/commands/auth.rs"
    - "apps/desktop-ui/src-tauri/src/lib.rs"
    - "apps/desktop-ui/src-tauri/Cargo.toml"
    - "apps/desktop-ui/src/lib/stores/auth.ts"

key-decisions:
  - "Refresh 5 min (300s) before expiry — balances UX (no mid-action expiry) and security (short-lived tokens)"
  - "Failed refresh silently clears tokens + emits auth:expired — per D-07, no error popup"
  - "Recursive spawn for next refresh — avoids global timer state management"

patterns-established:
  - "Rust→Svelte events via app.emit() + listen() for cross-layer state sync"

requirements-completed: [AUTH-04]

# Metrics
duration: 5min
completed: 2026-03-29
---

# Phase 06 Plan 03: Token Auto-Refresh Summary

**Background token refresh timer that schedules refresh 5 minutes before expiry, with auth:expired event emission on failure and frontend listener to clear reactive state**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-29T05:32:00Z
- **Completed:** 2026-03-29T05:37:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added refresh_access_token, clear_session_and_notify, start_refresh_timer to auth.rs
- Added .setup() hook in lib.rs to start timer on app launch
- Added initAuthListeners to stores/auth.ts for auth:expired event handling

## Task Commits

1. **Task 1: Implement token refresh and background timer** + **Task 2: Add frontend listener** — single commit
   - feat(06-03): implement background token auto-refresh and auth:expired listener

## Files Created/Modified
- `apps/desktop-ui/src-tauri/src/commands/auth.rs` — Added refresh_access_token, clear_session_and_notify, start_refresh_timer
- `apps/desktop-ui/src-tauri/src/lib.rs` — Added .setup() hook calling start_refresh_timer
- `apps/desktop-ui/src-tauri/Cargo.toml` — Added tokio dep
- `apps/desktop-ui/src/lib/stores/auth.ts` — Added initAuthListeners with auth:expired event

## Decisions Made
- Refresh 5 min before expiry — balances UX and security
- Failed refresh silently clears + emits event — no error popup per D-07
- Recursive spawn pattern — simpler than global timer state

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Next Phase Readiness
- Refresh timer wired in setup hook — 06-05 doesn't need to add it
- initAuthListeners ready for root layout (06-05)

---
*Phase: 06-google-oauth-authentication*
*Completed: 2026-03-29*
