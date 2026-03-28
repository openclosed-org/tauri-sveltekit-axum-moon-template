# Requirements: Tauri-SvelteKit-Axum Boilerplate

**Defined:** 2026-03-28
**Core Value:** Provide a runnable, tested, production-ready boilerplate with authentication (Google OAuth), multi-tenancy, backend infrastructure, and full stack best practices — so developers can start building business logic immediately.

## v1 Requirements

### Authentication

- [ ] **AUTH-01**: User can sign in with Google OAuth
- [ ] **AUTH-02**: OAuth callback handled via Tauri deep link
- [ ] **AUTH-03**: Session persisted across app restarts (tauri-plugin-store)
- [ ] **AUTH-04**: Session auto-refresh before expiry

### Multi-Tenancy

- [ ] **TENANT-01**: Database schema includes tenant_id on all tables
- [ ] **TENANT-02**: Query middleware automatically scopes by tenant_id
- [ ] **TENANT-03**: User belongs to exactly one tenant on signup

### Infrastructure

- [ ] **INFRA-01**: Docker-compose with Redis/cache service
- [ ] **INFRA-02**: Docker-compose with nginx reverse proxy
- [ ] **INFRA-03**: Docker-compose with libsql database
- [ ] **INFRA-04**: Local dev environment runs via docker-compose up

### Desktop Features

- [ ] **DESKTOP-01**: System tray icon with show/hide toggle
- [ ] **DESKTOP-02**: Native window state persistence (position, size)
- [ ] **DESKTOP-03**: Single instance lock prevents duplicate app
- [ ] **DESKTOP-04**: Graceful error handling with user-friendly messages

### Build & Performance

- [ ] **BUILD-01**: Production bundle under 15MB (LTO enabled)
- [ ] **BUILD-02**: Build passes on Windows, macOS, Linux
- [ ] **BUILD-03**: moon workspace configured with lint/test parallelism

### UI/UX

- [ ] **UI-01**: Three pages functional: Login, Counter, Admin dashboard
- [ ] **UI-02**: Mobile-first responsive layout
- [ ] **UI-03**: bitsUI components integrated
- [ ] **UI-04**: TailwindCSS v4 configured with custom theme

### Testing

- [ ] **TEST-01**: Unit tests pass for core Rust services
- [ ] **TEST-02**: Svelte component tests via vitest-browser-svelte
- [ ] **TEST-03**: E2E tests via Playwright cover main flows

### Package Configuration

- [ ] **PKG-01**: package.json includes SvelteKit, bitsUI, Tailwind v4
- [ ] **PKG-02**: package.json includes vitepress, lucide-animated, lottieplayer (commented unused)
- [ ] **PKG-03**: cargo.toml tauri dependencies include all core plugins
- [ ] **PKG-04**: cargo.toml axum dependencies properly versioned

## v2 Requirements

### Advanced Auth

- **AUTH-05**: Email/password signup as fallback
- **AUTH-06**: Two-factor authentication
- **AUTH-07**: Session revocation from admin panel

### Cloud Sync

- **SYNC-01**: Turso embedded replica synchronization
- **SYNC-02**: Offline-first data conflict resolution

## Out of Scope

| Feature | Reason |
|---------|--------|
| SurrealDB | libsql/turso simpler for bootstrap, lower complexity |
| Complex RBAC | Basic multi-tenancy sufficient for v1, role-based deferred |
| SSR | Tauri apps no SSR needed |
| Real-time sync everywhere | Local-first, sync on-demand |
| Email/password OAuth | Google OAuth sufficient for demo/boilerplate |
| Push notifications | Defer to v2 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PKG-01 | Phase 1 | Pending |
| PKG-02 | Phase 1 | Pending |
| PKG-03 | Phase 1 | Pending |
| PKG-04 | Phase 4 | Pending |
| BUILD-01 | Phase 4 | Pending |
| BUILD-02 | Phase 9 | Pending |
| BUILD-03 | Phase 1 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 3 | Pending |
| UI-03 | Phase 2 | Pending |
| UI-04 | Phase 2 | Pending |
| INFRA-01 | Phase 5 | Pending |
| INFRA-02 | Phase 5 | Pending |
| INFRA-03 | Phase 5 | Pending |
| INFRA-04 | Phase 5 | Pending |
| AUTH-01 | Phase 6 | Pending |
| AUTH-02 | Phase 6 | Pending |
| AUTH-03 | Phase 6 | Pending |
| AUTH-04 | Phase 6 | Pending |
| TENANT-01 | Phase 7 | Pending |
| TENANT-02 | Phase 7 | Pending |
| TENANT-03 | Phase 7 | Pending |
| DESKTOP-01 | Phase 8 | Pending |
| DESKTOP-02 | Phase 8 | Pending |
| DESKTOP-03 | Phase 8 | Pending |
| DESKTOP-04 | Phase 8 | Pending |
| TEST-01 | Phase 10 | Pending |
| TEST-02 | Phase 10 | Pending |
| TEST-03 | Phase 10 | Pending |

**Coverage:**
- v1 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after roadmap creation (10 phases, fine granularity)*