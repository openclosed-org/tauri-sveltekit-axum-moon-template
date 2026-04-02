# Phase 1: Package Foundation

**phase:** 1
**plan:** 01-package-foundation
**type:** auto
**autonomous:** true

## Objective

Configure all package dependencies and moon workspace for parallel lint/test execution.

**Requirements:** PKG-01, PKG-02, PKG-03, BUILD-03

## Context

- package.json already has SvelteKit + TailwindCSS v4 but missing bitsUI
- Cargo.toml (tauri) already declares all 3 core plugins — satisfied
- moon.yml root only has Rust tasks, no cross-package parallel aggregation
- Sub-packages (desktop-ui, domain, application, shared_contracts) each have their own moon.yml with lint/test tasks

## Tasks

### Task 1: Add bitsUI to package.json

**type:** auto

Add `bits-ui` as a dependency to `apps/desktop-ui/package.json`.

- `bits-ui` is the component library for Svelte 5 (SvelteKit UI primitive layer)
- Place in `dependencies` (not devDependencies) since it's runtime

**Done criteria:** package.json includes `"bits-ui": "^1.x"` in dependencies.

---

### Task 2: Add commented-out optional deps to package.json

**type:** auto

Add vitepress, lucide-animated, lottieplayer as commented-out entries in package.json, ready to enable when needed.

Format: use a `_commentedDependencies` section or inline JSON comments (JSON technically doesn't support comments, so use a convention block at the end).

**Done criteria:** package.json has clearly labeled commented-out entries for vitepress, lucide-animated, lottieplayer.

---

### Task 3: Verify Cargo.toml (tauri) plugin declarations

**type:** auto

Verify `apps/desktop-ui/src-tauri/Cargo.toml` and root `Cargo.toml` both correctly declare tauri-plugin-shell, tauri-plugin-dialog, tauri-plugin-store.

**Done criteria:** Both Cargo.toml files have the 3 plugins. No changes needed if already correct.

---

### Task 4: Add moon workspace parallel lint/test tasks

**type:** auto

Update root `moon.yml` to add aggregate `lint` and `test` tasks that depend on all sub-package lint/test tasks, enabling parallel execution.

Moon runs independent tasks in parallel by default. Adding `deps` on sub-project tasks creates the aggregation.

**Done criteria:** `moon run lint` triggers lint in all packages in parallel. `moon run test` triggers test in all packages in parallel.

## Verification

- `cat apps/desktop-ui/package.json` shows bits-ui in dependencies
- `cat apps/desktop-ui/package.json` shows commented-out optional deps
- `cat apps/desktop-ui/src-tauri/Cargo.toml` shows all 3 plugins
- `moon run lint --dry` shows parallel execution graph
