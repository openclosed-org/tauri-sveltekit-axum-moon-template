---
phase: 01-package-foundation
plan: 01-03
subsystem: tauri
tags: [tauri, plugins, cargo, workspace]
dependency_graph:
  requires:
    - PKG-03: Tauri plugin declarations
  provides:
    - All 7 Tauri plugins available to the Tauri app
  affects:
    - apps/desktop-ui/src-tauri/src/ (plugin registration in main.rs)
tech_stack:
  added:
    - tauri-plugin-fs (2)
    - tauri-plugin-deep-link (2)
    - tauri-plugin-window-state (2)
    - tauri-plugin-libsql (0.1.0)
  patterns:
    - Cargo workspace = true references
key_files:
  modified:
    - apps/desktop-ui/src-tauri/Cargo.toml
decisions:
  - "Preload all 7 plugins now — each will be used in later phases (fs in Phase 8, deep-link in Phase 6, libsql in Phase 5)"
metrics:
  duration: "~1 minute"
  completed: "2026-03-28T05:50:00Z"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 01 Plan 03: Tauri Plugin Registration Summary

## One-liner

Added 4 missing Tauri plugins to `apps/desktop-ui/src-tauri/Cargo.toml` — all 7 plugins now declared via workspace references.

## What Changed

Added to `[dependencies]`:
| Plugin | Version | Purpose | Phase |
|--------|---------|---------|-------|
| tauri-plugin-fs | workspace (2) | File system access | Phase 8 |
| tauri-plugin-deep-link | workspace (2) | OAuth callback handling | Phase 6 |
| tauri-plugin-window-state | workspace (2) | Window position persistence | Phase 8 |
| tauri-plugin-libsql | workspace (0.1.0) | Local embedded database | Phase 5 |

Pre-existing plugins (unchanged):
- tauri-plugin-shell, tauri-plugin-dialog, tauri-plugin-store

## Key Decisions

1. **Preload all plugins** — Declared now but used in later phases. Prevents needing to modify Cargo.toml mid-development. Each plugin will be registered in `main.rs` when its feature phase begins.

2. **workspace = true** — All plugins reference root Cargo.toml versions. Single source of truth for version management.

## Verification

- ✅ All 7 plugins present in src-tauri/Cargo.toml with `{ workspace = true }`
- ✅ All plugins declared in root Cargo.toml workspace.dependencies

## Known Limitations

- `cargo check` fails due to missing `cmake` in this environment (libsql-ffi requires cmake for native SQLite compilation). This is an environment prerequisite, not a configuration issue.

## Commits

| Hash | Message |
|------|---------|
| `8d36a6c` | `feat(01-03): add missing Tauri plugins to src-tauri/Cargo.toml` |

## Self-Check: PASSED

- [x] `apps/desktop-ui/src-tauri/Cargo.toml` exists and contains all 7 plugins
- [x] Commit `8d36a6c` exists in git log
- [x] All 7 plugins verified via automated check
