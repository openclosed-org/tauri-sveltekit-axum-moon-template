# Phase 1 Research: 仓库目录结构与工具链对齐

**Date:** 2026-04-01
**Status:** RESEARCH COMPLETE

## Current State Analysis

### Existing Directory Structure
```
apps/client/native/src-tauri/    ← Tauri desktop app
apps/client/web/app/             ← SvelteKit web app
apps/client/web/hosts/           ← exists (empty)
apps/client/browser-extension/   ← exists
servers/api/                     ← Axum API server
servers/workers/                 ← exists with atproto/chains/farcaster/nostr (empty)
packages/core/domain/            ← Rust domain crate
packages/core/usecases/          ← Rust usecases crate
packages/core/state/             ← exists (empty)
packages/contracts/api/          ← Rust contracts crate
packages/contracts/events/       ← exists
packages/contracts/protocols/    ← exists
packages/adapters/hosts/tauri/   ← Rust tauri adapter
packages/adapters/{chains,protocols,storage}/ ← exists with subdirs
packages/features/               ← 8 feature dirs exist (all empty)
packages/ui/{design-system,web}/ ← exists
packages/shared/{config,types,utils}/ ← exists
scripts/                         ← exists
.agents/skills/tailwindcss/     ← exists
```

### Missing from Blueprint (must create)
1. **Top-level `workers/`** — needs to be created; `servers/workers/` migrated to `workers/`
2. **`workers/jobs/`** — notifications/media/search/sync subdirs
3. **`tools/`** — scripts/, generators/, mcp/servers/, mcp/clients/, evals/datasets/, evals/graders/, evals/suites/
4. **`apps/ops/`** — docs-site/, storybook/
5. **`apps/client/web/hosts/`** — telegram-miniapp/, farcaster-miniapp/, base-app/
6. **`apps/client/desktop/`** — rename from native? (blueprint says desktop)
7. **`packages/contracts/`** — auth/, errors/, ui/, codegen/
8. **`packages/adapters/`** — auth/oauth/, auth/passkey/, auth/dpop/, telemetry/tracing/, telemetry/otel/, storage/sqlite/, storage/libsql/
9. **`packages/shared/`** — env/, testing/
10. **`packages/ui/`** — icons/, tokens/
11. **`.agents/`** — prompts/, playbooks/, rubrics/
12. **`servers/`** — gateway/, realtime/

### Existing Toolchain Config
- **moon.yml**: Only 8 cargo tasks (build, check, lint, test, format, format-fix, lint-all, check-all, test-all, bloat). No `repo:*` tasks.
- **Justfile**: 12 commands (dev, dev-web, dev-api, dev-tauri, dev-all, build-tauri, build-web, build-api, test-rust, test-web, test-e2e, check, clean). Old naming convention.
- **Cargo.toml**: 6 workspace members (apps/client/native/src-tauri, servers/api, packages/core/domain, packages/core/usecases, packages/contracts/api, packages/adapters/hosts/tauri)
- **.moon/workspace.yml**: 8 projects registered
- **No .prototools**: proto not configured yet
- **rust-toolchain.toml**: stable channel with rustfmt + clippy

## Gap Analysis

| Blueprint Target | Current State | Gap |
|-----------------|---------------|-----|
| 70+ directories | ~45 directories | ~25 missing dirs |
| workers/ at top-level | servers/workers/ | Migration needed |
| tools/ dir | Not exists | Create from scratch |
| apps/ops/ | Not exists | Create from scratch |
| 30+ repo:* moon tasks | 8 cargo tasks | Complete rewrite |
| Just as thin entry | Mixed responsibilities | Rewrite |
| .prototools | Missing | Create |

## Technical Decisions

### Directory Creation Strategy
- Create all missing directories with .gitkeep for empty ones
- `servers/workers/` → `workers/` migration is safe (all empty placeholder dirs)
- `apps/client/native` stays as-is (not renaming to desktop — too disruptive, not in context decisions)

### Moon Task Design
Per blueprint 03-toolchain-and-taskgraph.md:
- `repo:setup` — install deps + proto install
- `repo:bootstrap` — setup + doctor check
- `repo:doctor` — check toolchain, env vars, services
- `repo:dev-web` — moon run web app dev
- `repo:dev-desktop` — moon run tauri dev
- `repo:dev-api` — moon run api dev
- `repo:dev-fullstack` — combine web + api + desktop
- `repo:fmt` — cargo fmt + bun prettier
- `repo:lint` — cargo clippy + bun lint
- `repo:typecheck` — cargo check + tsc
- `repo:test-unit` — cargo test + bun test
- `repo:test-e2e` — playwright
- `repo:verify` — fmt + lint + typecheck + test
- `repo:typegen` — codegen placeholder

### Justfile Design
D-05/D-06: Just only exposes stable entry points, delegates to moon:
- `just setup` → moon run repo:setup
- `just dev` → moon run repo:dev-fullstack
- `just verify` → moon run repo:verify
- `just test` → moon run repo:test-unit
- `just typegen` → moon run repo:typegen

### proto Configuration
D-07/D-08: .prototools manages Bun + Node only; Rust stays in rust-toolchain.toml.

## Validation Architecture
No special validation needed — Phase 1 is structural (mkdir + config). Verified by:
1. `ls` checks for directory existence
2. `moon run repo:verify` (once moon tasks exist)
3. `just --list` shows expected commands
4. Cargo workspace builds after member updates
