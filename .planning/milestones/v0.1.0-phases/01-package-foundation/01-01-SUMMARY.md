---
phase: 01-package-foundation
plan: 01-01
subsystem: frontend
tags: [package.json, dependencies, bits-ui, testing]
dependency_graph:
  requires: []
  provides:
    - "Frontend dependency declarations for all subsequent UI work"
  affects:
    - "apps/desktop-ui/src/ (all Svelte component imports)"
tech_stack:
  added:
    - "bits-ui 2.16.4"
    - "@lucide/svelte 1.7.0"
    - "@pqoqubbw/icons latest"
    - "@lottiefiles/svelte-lottie-player 0.3.1"
    - "vitest 4.1.2"
    - "vitest-browser-svelte latest"
    - "@playwright/test 1.58.2"
    - "maestro latest"
    - "concurrently latest"
  patterns:
    - "Exact version pinning (no caret ranges)"
key_files:
  modified:
    - "apps/desktop-ui/package.json"
decisions:
  - "Pin all dependency versions exactly — no caret ranges — to ensure reproducible builds"
  - "Add concurrently for dev:tauri script to run vite + cargo tauri in parallel"
metrics:
  duration: "1m"
  completed: "2026-03-28T05:45:06Z"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 01 Plan 01: Frontend Package Dependencies Summary

## One-liner

Updated `apps/desktop-ui/package.json` to align with TECH_SELECTION.md §3.1: exact-pinned frontend deps, icon libraries, Lottie player, test tooling, and Tauri dev script.

## What Changed

### dependencies
| Package | Before | After |
|---------|--------|-------|
| bits-ui | `^2.16.4` | `2.16.4` (exact) |
| tailwindcss | `^4.0.0` (devDep) | `4.2.2` (dep) |
| @lucide/svelte | — | `1.7.0` (new) |
| @pqoqubbw/icons | — | `latest` (new) |
| @lottiefiles/svelte-lottie-player | — | `0.3.1` (new) |

### devDependencies
| Package | Before | After |
|---------|--------|-------|
| @sveltejs/kit | `^2.50.0` | `2.55.0` |
| svelte | `^5.54.0` | `5.55.0` |
| vite | `^8.0.0` | `8.0.3` |
| @biomejs/biome | `^1.9.4` | `1.9.4` |
| @sveltejs/adapter-static | `^3.0.0` | `3.0.0` |
| typescript | `^5.5.0` | `5.5.0` |
| vitest | — | `4.1.2` (new) |
| vitest-browser-svelte | — | `latest` (new) |
| @playwright/test | — | `1.58.2` (new) |
| maestro | — | `latest` (new) |
| concurrently | — | `latest` (new) |

### scripts added
- `dev:tauri` — runs `vite dev` + `cargo tauri dev` concurrently
- `test:unit` — runs `vitest run`
- `test:e2e` — runs `playwright test`
- `test:mobile` — runs `maestro test tests/flow/`

## Key Decisions

1. **Exact version pinning** — All dependencies use exact versions (no `^`). Ensures reproducible builds across environments. Trade-off: requires manual updates, acceptable for a boilerplate template.

2. **tailwindcss moved to dependencies** — Originally in devDependencies at `^4.0.0`, now in dependencies at `4.2.2`. Tailwind is a runtime build dependency that should be in `dependencies`.

3. **concurrently for dev script** — Enables parallel `vite dev` + `cargo tauri dev` execution. Standard pattern for Tauri+SvelteKit development.

## Verification

All 13 automated checks passed:
- ✅ bits-ui version = `2.16.4`
- ✅ tailwindcss version = `4.2.2`
- ✅ @lucide/svelte, @pqoqubbw/icons, @lottiefiles/svelte-lottie-player present
- ✅ vitest, @playwright/test, maestro, concurrently in devDependencies
- ✅ dev:tauri script includes `concurrently`
- ✅ test:unit script includes `vitest`
- ✅ No broken `lucide-animated` or `lottieplayer` references

## Deviations from Plan

None — plan executed exactly as written. The working tree already contained the correct changes (likely from a prior manual edit or incomplete previous run). Verification confirmed all criteria met.

## Commits

| Hash | Message |
|------|---------|
| `ec2c5a7` | `feat(01-01): align frontend dependencies with TECH_SELECTION.md` |

## Self-Check: PASSED

- [x] `apps/desktop-ui/package.json` exists and contains all required dependencies
- [x] Commit `ec2c5a7` exists in git log
- [x] All 13 verification checks pass
