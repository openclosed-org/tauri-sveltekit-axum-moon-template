---
phase: 04-backend-dependencies-build-optimization
plan: 01
subsystem: cargo-workspace
tags: [dependencies, cargo, workspace, release-profile]
dependency_graph:
  requires: []
  provides: [PKG-04, BUILD-01]
  affects: [04-02, 04-03]
tech_stack:
  added: [tower, tower-http, hyper, axum-extra, tracing, tracing-subscriber]
  patterns: [workspace-dependency-management, release-profile-optimization]
key_files:
  modified:
    - Cargo.toml
decisions: []
metrics:
  duration: ~5min
  completed: "2026-03-28"
---

# Phase 04 Plan 01: Root Workspace Dependencies Summary

## One-liner
Configured Axum middleware stack (tower/tower-http/hyper) as workspace dependencies with release profile panic="abort" optimization.

## Changes Made

### Cargo.toml Updates
1. **Axum middleware stack added:**
   - `tower = "0.5"` — middleware composition
   - `tower-http = { version = "0.6", features = ["cors", "trace", "timeout", "request-id"] }` — HTTP middleware
   - `hyper = { version = "1", features = ["full"] }` — HTTP foundation
   - `axum-extra = { version = "0.10", features = ["cookie"] }` — cookie/query support

2. **Axum json feature enabled:**
   - Changed `axum = "0.8.8"` → `axum = { version = "0.8.8", features = ["json"] }`

3. **Tracing dependencies added:**
   - `tracing = "0.1"`
   - `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`

4. **Release profile optimized:**
   - Added `panic = "abort"` to `[profile.release]`
   - Added `[profile.dev]` with `panic = "unwind"` for debugging

5. **Future-phase dependencies preloaded as comments:**
   - Phase 5: libsql, redis, rathole, vector
   - Phase 6: oauth2
   - Phase 8: tauri-plugin-updater

## Verification
- ✅ All 6 new workspace dependencies parse correctly
- ✅ panic profiles configured (abort for release, unwind for dev)
- ✅ 3 future-phase comment blocks present

## Commit
- `9d270cb`: chore(04-01): add Axum middleware stack to workspace dependencies and optimize release profile

## Self-Check: PASSED
