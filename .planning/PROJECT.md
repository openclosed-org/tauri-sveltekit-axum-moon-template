# Tauri-SvelteKit-Axum Boilerplate

## What This Is

A production-ready boilerplate/template for building cross-platform desktop applications using Tauri 2 + SvelteKit + Axum + moon workflow. Target: developers who want a fully configured starting point with best practices for medium-scale projects.

## Core Value

Provide a runnable, tested, production-ready boilerplate with authentication (Google OAuth), multi-tenancy, backend infrastructure (containerized Redis/cache, database, reverse proxy), and full stack best practices — so developers can start building business logic immediately.

## Requirements

### Validated

- ✓ Tauri 2 desktop app scaffolding — existing
- ✓ SvelteKit frontend foundation — existing
- ✓ Axum backend server — existing
- ✓ moon build toolchain — existing
- ✓ Mobile-first responsive layout base — existing
- ✓ Frontend dependencies aligned with TECH_SELECTION.md — Validated in Phase 01: package-foundation
- ✓ Rust workspace dependencies pinned with release profile — Validated in Phase 01: package-foundation
- ✓ All 7 Tauri plugins registered — Validated in Phase 01: package-foundation
- ✓ moon parallel lint/test configured — Validated in Phase 01: package-foundation
- ✓ Database infrastructure (SurrealDB + libsql dual-DB) — Validated in Phase 05: database-infrastructure
- ✓ Multi-tenant data isolation (tenant_id scoping) — Validated in Phase 07: multi-tenant-data-isolation

### Active

- [ ] User can sign in with Google OAuth
- [ ] Backend containerized with docker-compose (Redis/cache, DB, nginx)
- [ ] Tests pass for core flows
- [ ] Local dev environment fully configured

### Out of Scope

- [Email/password auth] — Google OAuth sufficient for boilerplate
- [Complex RBAC] — Basic multi-tenancy only for v1

## Context

**Current state:** Phase 07 (multi-tenant-data-isolation) complete — TenantId domain type, tenant-aware query scoping middleware, and tenant initialization API are implemented and verified. Next up is Phase 08 (desktop-native-features), while earlier pending phases remain tracked in ROADMAP.

**Tech stack:**
- Frontend: SvelteKit + bitsUI + TailwindCSS v4 + VitePress + @pqoqubbw/icons + Lottie
- Desktop: Tauri 2.10.3
- Backend: Axum 0.8.8
- Database: SurrealDB (服务端) + libsql/turso (本地 App) - 双数据库架构
- Build: moon
- Testing: Maestro (移动端) + Playwright (Web E2E)

**MCP/Skills needed locally:**
- Code index MCP (for codebase search)
- Websearch MCP
- Research MCP
- Frontend skills (Svelte, Tailwind, bitsUI)
- Backend skills (Axum, Rust)
- Tauri skills
- Testing skills

**UI Requirements:**
- Mobile-first, responsive design
- Three pages: Login, Counter, Admin dashboard
- Platform-agnostic (desktop + mobile web)

**Task lists (template features to configure):**

For package.json: vitepress, lottieplayer, and other utilities to preload but comment out unused

For Cargo (tauri + axum): Deep dive into docs for plugins and dependencies, preload but comment out unused

**Date reference:** March 28, 2026 — verify all versions/dependencies are current

## Constraints

- **[Stack]**: Tauri2 + SvelteKit + Axum + moon — Full-stack Rust/WebView
- **[Timeline]**: Best effort for production-ready quality
- **[Scope]**: Desktop-first but web-accessible, mobile-responsive
- **[Testing]**: Must have passing tests for core flows before release
- **[Infra]**: Docker-compose for local backend (Redis-like cache, database, nginx reverse proxy)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Dual DB: SurrealDB + libsql/turso | SurrealDB(服务端) + libsql(本地App) 双架构 | ✓ Implemented (Phase 05) |
| Google OAuth only | Reduce boilerplate complexity | — Pending |
| Maestro + Playwright | 移动端用 Maestro, Web用 Playwright | — Pending |
| VitePress (静态) | 构建后纯 HTML, 不占服务器资源 | — Pending |
| release-plz + git-cliff | CI/CD 自动化,综合评估不纯追 Rust | — Pending |
| Fine granularity phases | Maximum flexibility for feature iteration | — Pending |

---

*Last updated: 2026-03-29 after Phase 07 completion (multi-tenant-data-isolation)*
