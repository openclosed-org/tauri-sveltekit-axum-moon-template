<!-- GSD:project-start source:PROJECT.md -->
## Project

**Tauri-SvelteKit-Axum Boilerplate**

A production-ready boilerplate/template for building cross-platform desktop applications using Tauri 2 + SvelteKit + Axum + moon workflow. Target: developers who want a fully configured starting point with best practices for medium-scale projects.

**Core Value:** Provide a runnable, tested, production-ready boilerplate with authentication (Google OAuth), multi-tenancy, backend infrastructure (containerized Redis/cache, database, reverse proxy), and full stack best practices — so developers can start building business logic immediately.

### Constraints

- **[Stack]**: Tauri2 + SvelteKit + Axum + moon — Full-stack Rust/WebView
- **[Timeline]**: Best effort for production-ready quality
- **[Scope]**: Desktop-first but web-accessible, mobile-responsive
- **[Testing]**: Must have passing tests for core flows before release
- **[Infra]**: Docker-compose for local backend (Redis-like cache, database, nginx reverse proxy)
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Tauri** | 2.10.x | Desktop app shell | Latest stable with mobile support, plugin ecosystem, and security model |
| **SvelteKit** | 2.x | Frontend framework | Svelte 5 runes, excellent DX, adapter-static for Tauri SPA |
| **Svelte** | 5.x | UI framework | Runes-based reactivity, compile-time optimization, smaller bundles |
| **Axum** | 0.8.x | HTTP server (embedded) | Tower ecosystem, ergonomic extractors, type-safe routing |
| **Vite** | 8.x | Build tool | Native ESM, HMR, optimal Tauri integration |
### Tauri 2 Official Plugins (Recommended for Desktop)
| Plugin | Crate Version | Purpose | Priority |
|--------|---------------|---------|----------|
| `tauri-plugin-shell` | 2.x | Execute system commands, open URLs | **Required** |
| `tauri-plugin-dialog` | 2.x | Native file/save/message dialogs | **Required** |
| `tauri-plugin-store` | 2.x | Persistent key-value storage | **Required** |
| `tauri-plugin-fs` | 2.x | File system access with permissions | High |
| `tauri-plugin-window-state` | 2.x | Persist window size/position | High |
| `tauri-plugin-updater` | 2.x | In-app updates | High (production) |
| `tauri-plugin-notification` | 2.x | Native OS notifications | Medium |
| `tauri-plugin-clipboard` | 2.x | System clipboard read/write | Medium |
| `tauri-plugin-http` | 2.x | HTTP client from Rust side | Medium |
| `tauri-plugin-single-instance` | 2.x | Prevent multiple instances | Medium |
| `tauri-plugin-log` | 2.x | Configurable logging | Medium |
| `tauri-plugin-opener` | 2.x | Open files/URLs in external apps | Low |
| `tauri-plugin-process` | 2.x | Process management | Low |
| `tauri-plugin-autostart` | 2.x | Launch at system startup | Low (optional) |
| `tauri-plugin-stronghold` | 2.x | Encrypted secure storage | Low (sensitive data) |
### Database & Persistence Stack
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **libsql** (via `libsql` crate) | latest | Embedded SQLite-compatible DB | Rust-native, Turso-compatible, concurrent reads |
| **sqlx** | 0.8.x | Async SQL toolkit | Compile-time checked queries, SQLite support |
| **sea-orm** | latest | Async ORM (optional) | Higher-level abstraction if needed |
- SQLite wire-compatible but with concurrent read support
- Can sync to Turso cloud if needed later
- Pure Rust implementation (no C dependency issues)
- Works embedded in desktop apps perfectly
- Alternative: `rusqlite` if you don't need async or cloud sync
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tower` | 0.5.x | Middleware framework | Always with Axum |
| `tower-http` | 0.6.x | HTTP middleware (CORS, compression, trace) | Always with Axum |
| `serde` + `serde_json` | 1.x | Serialization | Always |
| `tokio` | 1.x | Async runtime | Always |
| `tracing` + `tracing-subscriber` | latest | Structured logging | Always |
| `thiserror` | 2.x | Error types | Always |
| `anyhow` | 1.x | Error context | Application layer |
| `uuid` | 1.x | UUID generation | When needed |
| `chrono` | 0.4.x | Date/time | When needed |
| `jsonwebtoken` | 9.x | JWT auth | If auth needed |
| `argon2` | 0.5.x | Password hashing | If auth needed |
### Frontend Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/api` | 2.x | Tauri JS bindings | Always |
| `tailwindcss` | 4.x | Utility-first CSS | Already in template |
| `@tanstack/svelte-query` | latest | Server state management | If using API calls |
| `sveltekit-superforms` | 2.x | Form handling/validation | Complex forms |
| `zod` | latest | Schema validation | With superforms |
| `bits-ui` | latest | Headless UI components | If building design system |
| `vaul-svelte` | latest | Drawer/sheet component | Mobile-friendly UI |
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| **Biome** | Linting + formatting | Already in template, faster than ESLint+Prettier |
| **moon** | Build orchestration | Already in template (`.moon/`) |
| **bun** | Package manager | Already in template (1.3.x) |
| **rust-analyzer** | Rust IDE support | Essential for VS Code/Neovim |
| **svelte-check** | Svelte type checking | In template's `check` script |
## Installation
# === Rust (Cargo.toml workspace) ===
# Core Tauri
# Tauri Plugins (pick what you need)
# Axum stack
# Database
# === Frontend (package.json) ===
# Core
# Dev
## Alternatives Considered
| Category | Recommended | Alternative | When to Use Alternative |
|----------|-------------|-------------|-------------------------|
| Frontend | SvelteKit | Leptos (Rust SSR) | If you want full Rust stack, no JS |
| Frontend | SvelteKit | SolidJS | If you need larger ecosystem |
| CSS | Tailwind 4 | UnoCSS | If you want attributify mode |
| HTTP Server | Axum | Actix-Web | If you prefer actor model |
| Database | libsql | rusqlite | Simpler embedded, no async needed |
| Database | libsql | PostgreSQL (via sqlx) | Server mode, multi-user |
| ORM | sqlx (raw SQL) | SeaORM | Prefer ORM abstraction |
| ORM | sqlx (raw SQL) | Diesel | Synchronous, mature |
| Package Mgr | bun | pnpm | If bun compatibility issues |
| Linting | Biome | ESLint + Prettier | If need ESLint plugins |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Tauri 1.x | EOL, missing mobile, weaker security model | Tauri 2.x |
| Webpack | Slow, complex config | Vite 8 |
| node-sass | Deprecated, native dependency issues | Tailwind CSS or vanilla-express |
| `reqwest` (for IPC) | Wrong abstraction for Tauri IPC | `@tauri-apps/api` invoke |
| Raw `std::fs` in Tauri | No permission model | `tauri-plugin-fs` |
| `nodemon` / `ts-node` | ESM issues, slow | Vite dev server / bun --watch |
## Stack Patterns by Variant
### Desktop-Only (Current Template)
- Use `adapter-static` with `fallback: 'index.html'` for SPA mode
- Embed Axum server for local API (optional, for complex backend logic)
- Use Tauri `invoke()` for frontend↔Rust IPC
- Use `tauri-plugin-store` for simple persistence, libsql for structured data
### Desktop + Mobile (Future)
- Same Tauri 2 codebase supports iOS/Android
- Add platform-specific capabilities via Tauri plugins
- Consider responsive design from day one (Tailwind breakpoints)
### With Cloud Sync (Future)
- Add Turso for edge-replicated SQLite
- Use libsql's sync feature for offline-first with cloud backup
- JWT auth via `jsonwebtoken` + `argon2` for user accounts
## Version Compatibility
| Package | Compatible With | Notes |
|---------|-----------------|-------|
| tauri 2.10.x | tauri-plugin-* 2.x | All plugins match core major version |
| SvelteKit 2.x | Svelte 5.x | Runes are stable |
| SvelteKit 2.x | adapter-static 3.x | SPA mode with fallback |
| axum 0.8.x | tower 0.5.x, tower-http 0.6.x | Required compatibility |
| Vite 8.x | @sveltejs/vite-plugin-svelte 5.x | Svelte 5 support |
| Tailwind 4.x | @tailwindcss/vite 4.x | Vite plugin mode |
| bun 1.3.x | SvelteKit 2.x | Works well as of 2026 |
## Tauri 2 Permission Model (Important!)
## Testing Stack
| Layer | Framework | Purpose |
|-------|-----------|---------|
| Rust unit tests | `cargo test` + `rstest` | Domain logic, utilities |
| Rust integration | `cargo test` + `sqlx::test` | Database operations |
| Svelte unit | Vitest + `vitest-browser-svelte` | Component logic |
| Svelte E2E | Playwright | User flows |
| Tauri E2E | WebDriver (optional) | Full app testing |
- Unit test domain logic in Rust crates (no Tauri dependency)
- Use Vitest for Svelte component tests with browser mode
- Playwright for E2E (can test Tauri app via WebDriver, or test web version)
- Avoid testing Tauri IPC directly—test Rust logic and frontend separately
## Docker Compose (Backend Infra Development)
# docker-compose.yml
- You have a companion backend service
- You need Redis for caching
- You're testing against PostgreSQL before deploying a server mode
## Sources
- [Tauri 2 Release Page](https://v2.tauri.app/release/) — Version 2.10.3 confirmed (March 2026)
- [Tauri Plugins](https://v2.tauri.app/plugin/) — Official plugin list with platform support
- [Axum Releases](https://github.com/tokio-rs/axum/releases) — v0.8.8 latest stable
- [Context7: Axum docs](/tokio-rs/axum) — Middleware patterns, tower integration
- [Context7: SvelteKit docs](/sveltejs/kit) — adapter-static SPA configuration
- [libSQL GitHub](https://github.com/tursodatabase/libsql) — 16.5k stars, active development
- [Vitest + Svelte](https://fubits.dev/notes/2026-02-21-collection-vitest-svelte-sveltekit-testing/) — Testing patterns (Feb 2026)
- [Rust ORMs 2026](https://aarambhdevhub.medium.com/rust-orms-in-2026) — sqlx vs sea-orm vs diesel comparison
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
