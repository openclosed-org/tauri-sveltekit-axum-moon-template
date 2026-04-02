---
phase: 04-minimal-feature-implementation
plan: '01'
subsystem: auth
tags: [auth, google-oauth, pkce, adapter, feature, hexagonal]
dependency_graph:
  requires: [contracts-auth, domain, runtime-tauri]
  provides: [adapter-google, feature-auth]
  affects: []
tech_stack:
  added: [adapter-google, feature-auth]
  patterns: [hexagonal-architecture, adapter-pattern, trait-based-abstraction]
key_files:
  created:
    - packages/adapters/auth/google/Cargo.toml
    - packages/adapters/auth/google/src/lib.rs
    - packages/features/auth/Cargo.toml
    - packages/features/auth/src/lib.rs
  modified:
    - Cargo.toml
decisions:
  - Remove #[tauri::command] from adapter (commands belong in host layer)
  - feature-auth defines UserProfile independently (not re-exported from adapter)
  - CounterServiceProxy stub added for future admin/agent plans
metrics:
  duration: ~3 minutes
  completed: 2026-04-02T11:25:00Z
---

# Phase 04 Plan 01: Google Auth Adapter & Feature Auth Summary

**One-liner:** Google OAuth PKCE adapter extracted from monolithic runtime_tauri commands, with AuthService trait defining the hexagonal boundary for auth features.

## What Changed

### adapter-google crate (`packages/adapters/auth/google/`)
- **Migrated all 422 lines** of OAuth logic from `runtime_tauri/commands/auth.rs`
- **GoogleAuthAdapter struct** with wrapped public API:
  - `start_login()` — PKCE flow with CSRF state
  - `handle_callback()` — code exchange + user profile decode
  - `get_session()` — session retrieval from Tauri store
  - `refresh_token()` — refresh token rotation
  - `clear_session()` — session cleanup + frontend notification
  - `start_timer()` — background refresh scheduling
- **AuthError enum** with Network, Config, InvalidCallback, TokenExchange, TokenExpired variants
- Preserved all original logic: PKCE generation, TCP listener callback, token exchange, JWT payload decode, recursive refresh timer
- Removed `#[tauri::command]` attributes — commands belong in the host layer, not the adapter

### feature-auth crate (`packages/features/auth/`)
- **AuthService trait** (`Send + Sync`) with async methods:
  - `start_login()` → `Result<(), AuthError>`
  - `handle_callback(url)` → `Result<AuthResult, AuthError>`
  - `get_session()` → `Result<Option<SessionInfo>, AuthError>`
  - `logout()` → `Result<(), AuthError>`
- **Supporting types**: UserProfile, AuthResult, SessionInfo, AuthError
- **Hexagonal boundary preserved**: depends on domain + usecases + contracts_auth, NOT on adapter-google
- **CounterServiceProxy stub** for future admin/agent use

### Workspace (root Cargo.toml)
- Added `packages/adapters/auth/google` and `packages/features/auth` to members
- Added `adapter-google` and `feature-auth` to workspace dependencies

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check -p adapter-google` | ✓ PASS |
| `cargo check -p feature-auth` | ✓ PASS |
| `cargo check --workspace` | ✓ PASS (4 pre-existing warnings in unrelated sync code) |
| Workspace member `adapters/auth/google` | ✓ PASS |
| Workspace member `features/auth` | ✓ PASS |
| `pub trait AuthService` defined | ✓ PASS |
| `pub struct GoogleAuthAdapter` defined | ✓ PASS |

## Known Stubs

- **CounterServiceProxy** (`packages/features/auth/src/lib.rs:97`) — empty struct, placeholder for future admin/agent plans. Will be wired when counter feature (Phase 04 Plan 03) is implemented.

## Commits

1. `feat(04-01): create adapter-google crate with OAuth PKCE` — 556 insertions, 3 files
2. `feat(04-01): create feature-auth crate with AuthService trait` — 96 insertions, 2 files

## Next Steps

- Phase 04 Plan 02: Counter feature crate (depends on feature-auth)
- Wire GoogleAuthAdapter to implement AuthService trait (adapter → feature bridge)
- Update runtime_tauri commands to delegate to adapter-google instead of inline logic

---

*Created: 2026-04-02 — Phase 04 Plan 01 execution complete*
