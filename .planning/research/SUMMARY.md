# Project Research Summary

**Project:** Tauri + SvelteKit + Axum Full-Stack Desktop Boilerplate
**Domain:** Cross-platform desktop application with embedded backend
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

This project is a production-ready boilerplate for building cross-platform desktop applications using **Tauri 2** (Rust-based desktop shell), **SvelteKit** (frontend framework with Svelte 5 runes), and **Axum** (embedded HTTP server). The research confirms this is an established and well-supported stack as of March 2026, with a real-world precedent shipping an 18MB binary with 114 API routes. The template uses a **moon** polyglot build orchestrator and **bun** as package manager, both already integrated.

The recommended architecture follows **Clean Architecture** principles: a `domain` crate for pure business logic, an `application` crate for use case orchestration, `runtime_tauri` for Tauri IPC glue, `runtime_server` for Axum HTTP endpoints, and a `shared_contracts` crate for type-safe DTOs across the Rust/TypeScript boundary. The primary IPC mechanism should be **Tauri `invoke()` calls** (1-5ms latency, type-safe), not HTTP to localhost (50-200ms, no type safety). Axum is reserved for external-facing endpoints like OAuth callbacks and webhooks.

Key risks center on **Tauri 2's permission system** (deny-by-default, must configure capabilities before any feature development), **bundle size optimization** (must configure LTO/release profile from the start), and **cross-platform build complexity** (WebView2 on Windows, platform-specific entitlements on macOS). The database strategy uses **libsql** (SQLite-compatible, pure Rust, concurrent reads, optional Turso cloud sync) with `rusqlite_migration` for embedded migrations. Google OAuth via deep link callback is the authentication requirement; multi-tenant isolation via `tenant_id` column scoping is a core feature.

## Key Findings

### Recommended Stack

See [STACK.md](./STACK.md) for full details.

**Core technologies:**
- **Tauri 2.10.x**: Desktop app shell — latest stable with mobile support, capability-based security, plugin ecosystem
- **SvelteKit 2.x + Svelte 5.x**: Frontend framework — runes-based reactivity (`$state`, `$derived`, `$effect`), adapter-static SPA mode for Tauri
- **Axum 0.8.x**: Embedded HTTP server — Tower ecosystem, ergonomic extractors, reserved for external API endpoints only
- **libsql**: Embedded SQLite-compatible database — pure Rust, concurrent reads, Turso sync path for cloud backup
- **Vite 8.x**: Build tool — native ESM, HMR, optimal Tauri integration
- **moon + bun**: Build orchestration + package manager — already in template

**Required Tauri 2 plugins:** `tauri-plugin-shell`, `tauri-plugin-dialog`, `tauri-plugin-store` (all v2.x). High priority: `tauri-plugin-fs`, `tauri-plugin-window-state`, `tauri-plugin-updater`. All plugins follow capability-based permissions — each must be declared in `capabilities/default.json`.

**Testing stack:** `cargo test` + `rstest` for Rust, Vitest + `vitest-browser-svelte` for Svelte components, Playwright for E2E.

### Expected Features

See [FEATURES.md](./FEATURES.md) for full details.

**Must have (v1 — table stakes):**
- Google OAuth login with deep link callback — core PROJECT.md requirement
- Session persistence (JWT in tauri-plugin-store) — required for auth to be useful
- Multi-tenant data isolation (`tenant_id` in schema) — core PROJECT.md requirement
- libsql database with basic schema and migrations — required for any data persistence
- System tray with show/hide/quit — expected desktop feature
- Window state persistence — trivial to add, high user expectation
- Single instance lock — prevents duplicate launches
- Docker-compose for local dev infrastructure — core PROJECT.md requirement
- Error boundaries + toast notifications — required for production feel

**Should have (v1.x — competitive differentiators):**
- Embedded Axum server (full REST API inside desktop app — rare in Tauri templates)
- Local-first database with Turso sync (offline-capable, syncs when connected)
- Auto-updater (when ready to distribute)
- Deep link protocol handler (extending OAuth pattern for custom URLs)

**Defer (v2+):**
- Mobile (iOS/Android) support — Tauri mobile is maturing, desktop-first is scope
- CRDT-based collaborative editing — very complex, feature-specific
- Complex RBAC/permissions — massive scope creep for boilerplate
- Email/password auth — doubles auth surface, add when specific project needs it
- Server-side rendering — incompatible with Tauri WebView, SPA mode is correct

### Architecture Approach

See [ARCHITECTURE.md](./ARCHITECTURE.md) for full details.

The architecture is a **layered Clean Architecture** with clear separation of concerns and two runtime entry points (Tauri and Axum) sharing the same domain and application layers.

**Major components:**
1. **SvelteKit SPA Frontend** (`apps/desktop-ui/`) — UI rendering, SPA routing, communicates via Tauri `invoke()` through typed IPC wrapper functions in `lib/ipc/`
2. **Domain Crate** (`crates/domain/`) — Pure business rules, zero external dependencies, entities and value objects
3. **Application Crate** (`crates/application/`) — Use case orchestration, defines trait interfaces (ports) for infrastructure
4. **Runtime Tauri** (`crates/runtime_tauri/`) — `#[tauri::command]` handlers, plugin registration, app state management
5. **Runtime Server** (`crates/runtime_server/`) — Axum HTTP routes, Tower middleware (CORS, compression, tracing)
6. **Shared Contracts** (`crates/shared_contracts/`) — DTOs shared between frontend and backend, single source of truth for types

**Key architectural decisions:**
- **IPC over HTTP for local communication** — `invoke()` is 20-100x faster than HTTP to localhost and provides compile-time type safety
- **Svelte 5 runes** for all state management — no external state library needed
- **Tenant-scoped repository pattern** — all data access auto-scoped by `tenant_id` to prevent cross-tenant leaks
- **Axum reserved for external endpoints** — OAuth callbacks, webhooks, future public API

### Critical Pitfalls

See [PITFALLS.md](./PITFALLS.md) for full details.

1. **Tauri 2 Permission System Misconfiguration** — Deny-by-default security model; plugins fail silently without capability declarations. Must configure `capabilities/default.json` before any feature development. *Prevention: add permissions for every plugin at install time, test in production build.*

2. **Bundle Size Bloat** — Default Cargo settings produce 50MB+ binaries when 5-10MB is achievable. *Prevention: configure `[profile.release]` with `lto=true`, `codegen-units=1`, `opt-level="z"`, `strip=true` from the start.*

3. **IPC vs HTTP Performance Mismatch** — Using `fetch('http://localhost:...')` for local communication adds 50-200ms latency vs 1-5ms for `invoke()`. *Prevention: use Tauri IPC for all frontend↔Rust communication; HTTP only for external services.*

4. **Database Migration Failures in Production** — Hardcoded paths and missing migration runners cause crashes on first production run. *Prevention: use `rusqlite_migration`, store DB in OS-appropriate data directory via `app.path().app_local_data_dir()`, run migrations on every startup.*

5. **Cross-Platform Build Complexity** — WebView2 missing on Windows, platform-specific entitlements, path separator issues. *Prevention: CI/CD with target platform toolchains, use `std::path::Path` everywhere, bundle WebView2 installer for Windows.*

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation & Configuration
**Rationale:** Must come first — Tauri 2's permission system and build optimization are prerequisites for everything else. Skipping this causes cryptic production failures and bloated binaries.
**Delivers:** Working Tauri 2 + SvelteKit + Axum scaffold with correct capabilities, optimized build profile, moon task configuration
**Addresses:** Window state persistence, single instance lock (trivially achievable in this phase)
**Avoids:** Pitfall 1 (permission misconfiguration), Pitfall 2 (bundle size bloat), Pitfall 6 (cross-platform build issues)
**Stack:** Tauri 2.10.x, SvelteKit 2.x, Axum 0.8.x, Vite 8.x, moon, bun
**Research needed:** No — well-documented setup patterns

### Phase 2: Authentication & Session Management
**Rationale:** Auth is a dependency for multi-tenancy and data isolation. Must implement before any data layer work.
**Delivers:** Google OAuth login via deep link callback, JWT session persistence, user entity
**Addresses:** Google OAuth login (P1), Session persistence (P1), Deep link protocol handler
**Avoids:** Pitfall 5 (security model gaps — CSP, input validation)
**Stack:** `tauri-plugin-deep-link`, `tauri-plugin-store`, `tauri-plugin-google-auth` or manual OAuth flow
**Research needed:** YES — OAuth flow complexity, deep link registration per platform

### Phase 3: Data Layer & Multi-Tenancy
**Rationale:** Depends on auth (knows tenant_id). Implements the core persistence and isolation features.
**Delivers:** libsql database setup, schema with `tenant_id` columns, migration system, tenant-scoped repository pattern
**Addresses:** libsql database (P1), Multi-tenant data isolation (P1), Docker-compose dev infrastructure
**Avoids:** Pitfall 4 (database migration failures — use `rusqlite_migration`, proper path handling)
**Stack:** libsql crate, rusqlite_migration, docker-compose
**Research needed:** Partially — libsql integration patterns, but SQLite schema design is well-understood

### Phase 4: Desktop Features & Polish
**Rationale:** Independent of auth/data — can be developed in parallel but logically follows once core app works.
**Delivers:** System tray, native dialogs, error boundaries, toast notifications, loading states
**Addresses:** System tray (P1), Native file dialogs, Error boundaries + toasts (P1)
**Avoids:** UX pitfalls (no loading states, blank windows, unhelpful error messages)
**Stack:** Tauri built-in TrayIconBuilder, `tauri-plugin-dialog`, `tauri-plugin-notification`
**Research needed:** No — well-documented Tauri plugin APIs

### Phase 5: Distribution & Updates
**Rationale:** Final phase — needed when ready to ship to users. Depends on all previous phases being stable.
**Delivers:** Auto-updater, signing key configuration, platform-specific installers (MSI/DMG/AppImage)
**Addresses:** Auto-updater (P2), Cross-platform build pipeline
**Avoids:** Pitfall 6 (cross-platform build complexity — CI/CD with all target platforms)
**Stack:** `tauri-plugin-updater`, GitHub Actions or similar CI for multi-platform builds
**Research needed:** Partially — signing and notarization per platform

### Phase 6: Advanced Features (v1.x)
**Rationale:** Post-launch enhancements based on validated needs.
**Delivers:** Turso cloud sync, embedded Axum REST API (for external consumers), native notifications, global shortcuts
**Addresses:** Turso cloud sync (P2), Embedded Axum API (P3), Native notifications (P2)
**Stack:** `tauri-plugin-libsql`, Axum routes with Tower middleware
**Research needed:** YES — Turso sync patterns, embedded replica architecture

### Phase Ordering Rationale

- **Foundation first** (Phase 1): Tauri 2 capabilities and build config are prerequisites; mistakes here cause production-only failures that are hard to debug
- **Auth before data** (Phase 2 → 3): Multi-tenancy requires knowing the tenant; auth must exist before tenant-scoped data access
- **Features after core** (Phase 4): Desktop polish is independent but logically follows once the app has functional auth and data
- **Distribution last** (Phase 5): Signing keys, installers, and update servers are only needed when ready to ship
- **Advanced features deferred** (Phase 6): Turso sync and Axum REST API add complexity; validate core product first

### Research Flags

Phases needing deeper research during planning:
- **Phase 2 (Auth):** OAuth deep link flow varies by platform; `tauri-plugin-google-auth` is relatively new (v0.5.1). Needs validation of the plugin vs manual OAuth implementation trade-off.
- **Phase 6 (Advanced):** Turso embedded replica patterns and `tauri-plugin-libsql` (v0.1.0) are bleeding edge. Needs proof-of-concept.

Phases with standard patterns (skip research):
- **Phase 1 (Foundation):** Well-documented Tauri 2 setup, moon configuration patterns exist in template
- **Phase 3 (Data Layer):** SQLite schema design and migrations are mature; libsql is wire-compatible
- **Phase 4 (Desktop Features):** Tauri plugin APIs are extensively documented

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Official docs, version compatibility verified, real-world production precedent (Reddit Mar 2026) |
| Features | HIGH | Feature prioritization grounded in PROJECT.md requirements and Tauri 2 plugin ecosystem |
| Architecture | HIGH | Clean Architecture is well-established; IPC vs HTTP decision clear from performance data |
| Pitfalls | MEDIUM-HIGH | Based on Tauri GitHub issues, community reports, and documented anti-patterns; some platform-specific edge cases may surface during implementation |

**Overall confidence:** HIGH

### Gaps to Address

- **Google OAuth plugin maturity:** `tauri-plugin-google-auth` is v0.5.1 — evaluate whether to use it or implement manual OAuth with `tauri-plugin-deep-link`. Validate during Phase 2 planning.
- **Tauri mobile readiness:** PROJECT.md mentions mobile-responsiveness; Tauri 2 mobile support is functional but less mature than desktop. Defer mobile to v2+.
- **Drizzle ORM in WebView:** Using Drizzle via `sqlite-proxy` driver is documented but not mainstream. May need fallback to sqlx (Rust-side) for complex queries.
- **Cross-platform CI/CD:** Specific GitHub Actions workflow configuration for Windows/macOS/Linux builds not researched in detail. Address during Phase 5 planning.
- **Turso production readiness:** `tauri-plugin-libsql` is v0.1.0 — very early. Cloud sync features should be gated behind feature flag when implemented.

## Sources

### Primary (HIGH confidence)
- [Tauri 2 Documentation](https://v2.tauri.app/) — plugin list, security model, IPC, system tray, capabilities
- [Context7: Axum docs](/tokio-rs/axum) — middleware patterns, Tower integration
- [Context7: SvelteKit docs](/sveltejs/kit) — adapter-static SPA configuration
- [Svelte 5 Runes](https://svelte.dev/docs/svelte/$state) — $state, $derived, $effect
- [libSQL GitHub](https://github.com/tursodatabase/libsql) — 16.5k stars, embedded SQLite with Turso sync

### Secondary (MEDIUM confidence)
- [Reddit: Shipped Tauri 2 + Svelte 5 + Axum](https://www.reddit.com/r/tauri/comments/1s4ah2f/) — 18MB binary, 114 API routes (Mar 2026)
- [Vitest + Svelte Testing Patterns](https://fubits.dev/notes/2026-02-21-collection-vitest-svelte-sveltekit-testing/) — Feb 2026
- [Rust ORMs 2026 Comparison](https://aarambhdevhub.medium.com/rust-orms-in-2026) — sqlx vs sea-orm vs diesel
- [Building Local-First Tauri App](https://dev.to/huakun/building-a-local-first-tauri-app-with-drizzle-orm-encryption-and-turso-sync-31pn) — Drizzle + libsql + Turso patterns
- [Multi-Tenant SaaS Architecture 2026](https://dev.to/waqarhabib/building-a-multi-tenant-saas-app-with-react-and-nodejs-in-2026-31ih) — tenant_id isolation

### Tertiary (LOW confidence)
- [tauri-plugin-google-auth v0.5.1](https://crates.io/crates/tauri-plugin-google-auth) — newer plugin, needs validation
- [tauri-plugin-libsql v0.1.0](https://crates.io/crates/tauri-plugin-libsql) — very early stage (Feb 2026)
- [Tauri GitHub Issues](https://github.com/tauri-apps/tauri/issues) — #14259 (fs permissions), #12312 (cross-platform compilation)

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
