# ROADMAP: Tauri-SvelteKit-Axum Boilerplate

**Generated:** 2026-03-28
**Granularity:** fine
**Total v1 Requirements:** 29

## Phases

- [x] **Phase 1: Package Foundation** - Configure all package dependencies and moon workspace ✅
- [ ] **Phase 2: UI Styling Infrastructure** - TailwindCSS v4 theme and bitsUI component library
- [ ] **Phase 3: Application Pages** - Login, Counter, Admin dashboard with responsive layout
- [ ] **Phase 4: Backend Dependencies & Build** - Axum/Tauri cargo deps, release profile optimization
- [ ] **Phase 5: Docker Infrastructure** - Containerized local dev environment (Redis, nginx, libsql)
- [ ] **Phase 6: Google OAuth Authentication** - OAuth login, deep link callback, session management
- [ ] **Phase 7: Multi-Tenant Data Isolation** - tenant_id schema, query middleware, user-tenant binding
- [ ] **Phase 8: Desktop Native Features** - System tray, window state, single instance, error handling
- [ ] **Phase 9: Cross-Platform Build Pipeline** - Windows, macOS, Linux build verification
- [ ] **Phase 10: Test Suite** - Unit, component, and E2E tests for all core flows

## Phase Details

### Phase 1: Package Foundation
**Goal**: Project has all dependencies declared and moon workspace configured for parallel lint/test
**Depends on**: Nothing
**Requirements**: PKG-01, PKG-02, PKG-03, BUILD-03
**Success Criteria** (what must be TRUE):
  1. `package.json` declares SvelteKit, bitsUI, TailwindCSS v4, @pqoqubbw/icons, Lottie
  2. `package.json` declares VitePress, vitest, maestro, playwright as devDependencies
  3. `Cargo.toml` (tauri) declares all core plugins: tauri-plugin-shell, tauri-plugin-dialog, tauri-plugin-store, tauri-plugin-libsql
  4. `Cargo.toml` (workspace) declares Tauri 2.10.3, Axum 0.8.8, SurrealDB 3.0.5
  5. `moon.yml` workspace runs lint and test tasks in parallel across packages
 **Plans:** 4 plans

Plans:
- [x] 01-01-PLAN.md — Frontend package dependencies (package.json alignment) ✅ `ec2c5a7`
- [x] 01-02-PLAN.md — Rust workspace dependencies (root Cargo.toml) ✅ `04228c1`
- [x] 01-03-PLAN.md — Tauri plugin registration (src-tauri/Cargo.toml) ✅ `8d36a6c`
- [x] 01-04-PLAN.md — Verification gate (config audit passed, env deps noted)

### Phase 2: UI Styling Infrastructure
**Goal**: Frontend has a configured design system with reusable components ready for page construction
**Depends on**: Phase 1
**Requirements**: UI-03, UI-04
**Success Criteria** (what must be TRUE):
  1. TailwindCSS v4 compiles with a custom theme (colors, fonts, spacing tokens)
  2. bitsUI components (Button, Dialog, Input, Select) render correctly in the app
  3. Dark mode / light mode toggle works via TailwindCSS v4 theme switching
  4. Component library is importable from `$lib/components`
**Plans:** 3 plans

Plans:
- [ ] 02-01-PLAN.md — TailwindCSS v4 Vite plugin + global CSS with @theme tokens
- [ ] 02-02-PLAN.md — cn() utility + dark mode theme store
- [ ] 02-03-PLAN.md — Root layout + 9 component wrappers + barrel export

**UI hint**: yes

### Phase 3: Application Pages
**Goal**: Three core pages are functional and responsive on mobile and desktop viewports
**Depends on**: Phase 2
**Requirements**: UI-01, UI-02
**Success Criteria** (what must be TRUE):
  1. Login page renders with Google sign-in button and branded layout
  2. Counter page displays a reactive counter with increment/decrement controls (Svelte 5 runes)
  3. Admin dashboard page shows a placeholder layout with sidebar navigation
  4. All three pages adapt cleanly to mobile viewport (375px) and desktop viewport (1280px)
  5. SPA routing works between all three pages without full page reloads
**Plans**: 3 plans

Plans:
- [ ] 03-01-PLAN.md — Route groups + layouts + responsive navigation ✅
- [ ] 03-02-PLAN.md — Counter page with $state rune
- [ ] 03-03-PLAN.md — Admin dashboard placeholder

**UI hint**: yes

### Phase 4: Backend Dependencies & Build Optimization
**Goal**: Rust workspace compiles with optimized release profile producing binaries under 15MB
**Depends on**: Phase 1
**Requirements**: PKG-04, BUILD-01
**Success Criteria** (what must be TRUE):
  1. `Cargo.toml` (axum) has properly versioned dependencies for axum, tower, tokio
  2. `cargo build --release` produces a binary under 15MB (LTO enabled, codegen-units=1, opt-level="z", strip=true)
  3. Axum server starts on configured port and responds to health check
**Plans**: TBD

### Phase 5: Database & Infrastructure (双数据库架构)
**Goal**: SurrealDB(服务端) + libsql/turso(本地App) 双数据库架构就绪，HTTP客户端就绪
**Depends on**: Nothing (parallel track)
**Requirements**: INFRA-01, INFRA-02, INFRA-03, INFRA-04
**Success Criteria** (what must be TRUE):
  1. **服务端DB**: SurrealDB 服务端配置完成 (Axum连接)
  2. **本地DB**: libsql/turso 本地存储 via tauri-plugin-libsql
  3. **云同步**: Turso embedded-replica 同步配置 (可选)
  4. **HTTP Client**: `reqwest 0.13.2` 配置完成
  5. **Adapter Pattern**: 数据库抽象层实现 (支持多后端切换)
 **Plans**: TBD

**核心依赖**:
- 服务端: `surrealdb` - 独立部署通过 Axum 访问
- 本地: `tauri-plugin-libsql` + `libsql`
- 策略模式: DatabaseAdapter trait

### Phase 6: Google OAuth Authentication
**Goal**: User can sign in with Google, session persists across app restarts, and tokens auto-refresh
**Depends on**: Phase 3, Phase 4
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04
**Success Criteria** (what must be TRUE):
  1. Clicking "Sign in with Google" opens the OAuth consent screen in the system browser
  2. After Google consent, the app receives the callback via Tauri deep link and logs the user in
  3. Closing and reopening the app restores the session without re-prompting for login
  4. Session token auto-refreshes before expiry without user intervention
  5. Login page redirects to the Counter page when user is already authenticated
**Plans**: TBD
**UI hint**: yes

### Phase 7: Multi-Tenant Data Isolation
**Goal**: All data access is automatically scoped by tenant_id, preventing cross-tenant data leaks
**Depends on**: Phase 6
**Requirements**: TENANT-01, TENANT-02, TENANT-03
**Success Criteria** (what must be TRUE):
  1. Every database table includes a `tenant_id` column (visible in schema/migration files)
  2. Query middleware automatically filters all SELECT queries by the current user's tenant_id
  3. New user signup creates a tenant record and binds the user to it
  4. Attempting to query data with a mismatched tenant_id returns empty results (not errors)
**Plans**: TBD

### Phase 8: Desktop Native Features
**Goal**: App behaves as a polished desktop application with system tray, persistent window state, and user-friendly errors
**Depends on**: Phase 4
**Requirements**: DESKTOP-01, DESKTOP-02, DESKTOP-03, DESKTOP-04
**Success Criteria** (what must be TRUE):
  1. System tray icon appears with a menu offering Show/Hide/Quit options
  2. Resizing and moving the window, then restarting, restores the exact previous position and size
  3. Launching the app a second time focuses the existing window instead of opening a new one
  4. Unhandled errors display a user-friendly toast/message dialog instead of a blank screen
**Plans**: TBD

### Phase 9: Cross-Platform Build Pipeline
**Goal**: Project builds successfully on Windows, macOS, and Linux from a single configuration
**Depends on**: Phase 4, Phase 8
**Requirements**: BUILD-02
**Success Criteria** (what must be TRUE):
  1. `cargo build` completes without errors on Windows (with WebView2)
  2. `cargo build` completes without errors on macOS (with entitlements)
  3. `cargo build` completes without errors on Linux
  4. CI configuration (or moon task) exists to verify all three platform builds
**Plans**: TBD

### Phase 10: Test Suite
**Goal**: Core application flows are covered by passing unit, component, and E2E tests
**Depends on**: Phase 6, Phase 7, Phase 8
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. `cargo test` passes for all Rust service/middleware unit tests
  2. Vitest passes for Svelte component tests (Login page, Counter page interactions)
  3. Playwright E2E tests cover: login flow, counter increment/decrement, admin page navigation
  4. Test output shows green with no skipped or ignored tests for core flows
**Plans**: TBD
**UI hint**: yes

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Package Foundation | 4/4 | Completed | 2026-03-28 |
| 2. UI Styling Infrastructure | 0/3 | Not started | - |
| 3. Application Pages | 0/3 | Not started | - |
| 4. Backend Dependencies & Build | 0/3 | Not started | - |
| 5. Docker Infrastructure | 0/5 | Not started | - |
| 6. Google OAuth Authentication | 0/5 | Not started | - |
| 7. Multi-Tenant Data Isolation | 0/4 | Not started | - |
| 8. Desktop Native Features | 0/4 | Not started | - |
| 9. Cross-Platform Build Pipeline | 0/4 | Not started | - |
| 10. Test Suite | 0/4 | Not started | - |

## Coverage Map

| Requirement | Phase | Status |
|-------------|-------|--------|
| PKG-01 | Phase 1 | ✅ Complete |
| PKG-02 | Phase 1 | ✅ Complete |
| PKG-03 | Phase 1 | ✅ Complete |
| PKG-04 | Phase 4 | Pending |
| BUILD-03 | Phase 1 | ✅ Complete |
| UI-03 | Phase 2 | Pending |
| UI-04 | Phase 2 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 3 | Pending |
| BUILD-01 | Phase 4 | Pending |
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
| BUILD-02 | Phase 9 | Pending |
| TEST-01 | Phase 10 | Pending |
| TEST-02 | Phase 10 | Pending |
| TEST-03 | Phase 10 | Pending |

**Coverage: 29/29 v1 requirements mapped ✓**

## Phase Dependency Graph

```
Phase 1 (Packages) ──→ Phase 2 (UI Infra) ──→ Phase 3 (Pages) ──→ Phase 6 (Auth) ──→ Phase 7 (Tenancy)
       │                                        ↑                               │
       └──→ Phase 4 (Backend/Build) ────────────┘                               └──→ Phase 10 (Tests)
                │                │                                                       ↑
                └──→ Phase 8 (Desktop) ──→ Phase 9 (Cross-Platform)                    │
                                                                                        │
Phase 5 (Docker) ──────────────────────────────────────────────────────────────────────┘
(independent)
```

---

*Roadmap generated: 2026-03-28*
*Ready for phase planning: `/gsd-plan-phase 1`*
