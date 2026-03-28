# Feature Research

**Domain:** Tauri 2 + SvelteKit + Axum Full-Stack Desktop Boilerplate
**Researched:** 2026-03-28
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Google OAuth login** | PROJECT.md explicitly requires it; desktop OAuth is standard practice in 2026 | MEDIUM | Use `tauri-plugin-google-auth` (v0.5.1) or deep-link + system browser pattern. Deep link callback via `tauri-plugin-deep-link` (v2.4.7, 1M+ downloads). |
| **Session persistence** | Users expect to stay logged in across app restarts | LOW | Use `tauri-plugin-store` (v2.x) to persist JWT tokens. Or `tauri-plugin-stronghold` for encrypted storage of sensitive tokens. |
| **Native file dialogs** | Every desktop app needs open/save dialogs | LOW | `tauri-plugin-dialog` (v2.x) — official plugin, supports file picker, save dialog, message boxes. Platform: desktop only. |
| **Window state persistence** | Users expect window to remember size/position | LOW | `tauri-plugin-window-state` (v2.x) — persists size, position, maximized state. One-line integration. |
| **Single instance lock** | Desktop apps shouldn't open multiple windows | LOW | `tauri-plugin-single-instance` (v2.x) — prevents duplicate launches, focuses existing window. |
| **Auto-updater** | Production apps need seamless updates | MEDIUM | `tauri-plugin-updater` (v2.x) — supports static JSON endpoint or update server. Needs signing keys configured. |
| **System tray icon** | Expected for apps that run in background | MEDIUM | Built-in Tauri 2 `TrayIconBuilder` API. Handle click events, context menu, show/hide main window. See ARCHITECTURE.md for patterns. |
| **Mobile-first responsive layout** | PROJECT.md requires platform-agnostic, mobile-responsive | MEDIUM | Tailwind CSS v4 breakpoints + SvelteKit `adapter-static` SPA mode. Already in template base. |
| **Error boundaries & loading states** | Production apps can't show blank screens | LOW | Svelte 5 `$state` + error boundaries. Toast notifications via `sonner-svelte` or similar. |
| **Structured logging** | Debugging production issues requires logs | LOW | `tracing` + `tracing-subscriber` on Rust side; `tauri-plugin-log` (v2.x) for JS↔Rust log bridging. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Embedded Axum server** | Full REST API inside desktop app — rare in Tauri templates. Real Reddit post (Mar 2026) shipping 18MB binary with 114 API routes. | HIGH | Axum runs in Tauri's tokio runtime. Frontend can use both `invoke()` for IPC and HTTP for complex API patterns. Unique selling point. |
| **Local-first database with Turso sync** | Offline-capable, syncs when connected. `tauri-plugin-libsql` (v0.1.0, Feb 2026) provides libsql + encryption + Turso embedded replica. | HIGH | Local SQLite file syncs bidirectionally with Turso cloud. Drizzle ORM via `sqlite-proxy` driver works in WebView. See database schema section below. |
| **Multi-tenant data isolation** | Boilerplate with `tenant_id` scoping ready to go — most templates skip this entirely | MEDIUM | Every table has `tenant_id` column. Query middleware auto-scopes. Row-level security pattern. |
| **Docker-compose dev infra** | One-command local environment: Redis, DB, nginx | LOW | Already in PROJECT.md requirements. Standard docker-compose.yml. |
| **Deep link protocol handler** | App responds to custom URLs (e.g., `myapp://callback`) | MEDIUM | `tauri-plugin-deep-link` (v2.4.7) — needed for OAuth callbacks. Also enables sharing links to your app. |
| **Cross-platform from single codebase** | Desktop + mobile web from same SvelteKit codebase | MEDIUM | Tauri 2 supports iOS/Android. `adapter-static` SPA mode works everywhere. Template demonstrates this. |
| **Type-safe IPC contract** | Shared Rust types via `shared_contracts` crate | MEDIUM | Template already has `shared_contracts/` crate. Frontend and backend share type definitions. Reduces bugs. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Real-time sync everywhere** | "Users want live updates" | CRDT complexity, battery drain, unnecessary for most boilerplate use cases | Use Turso's `sync()` on-demand (launch, manual refresh, network reconnect). Add CRDTs only for collaborative editing features. |
| **Complex RBAC/permissions** | "Enterprise needs role-based access" | Massive scope creep for a boilerplate; every app has different permission needs | Basic `tenant_id` isolation only for v1. Let consumers add their own RBAC layer. |
| **Email/password auth** | "Some users don't have Google" | Doubles auth surface area, password reset flows, email verification | Google OAuth only. Add email/password when a specific project needs it. |
| **Server-side rendering (SSR)** | "Better SEO, faster first paint" | Incompatible with Tauri's WebView model; `adapter-static` requires SPA | SPA mode with `fallback: 'index.html'`. SSR adds no value for desktop apps. |
| **Full ORM with migrations in JS** | "Developers prefer Drizzle/Prisma DX" | Drizzle's migrator uses Node `fs` which doesn't exist in WebView | Use Drizzle via `sqlite-proxy` driver + Vite `import.meta.glob` for inlined migrations. Or use sqlx directly in Rust. |
| **Electron-style full Node.js backend** | "Familiar Node ecosystem" | Defeats Tauri's size/performance advantage; adds Node dependency | Use Axum for backend logic. If you absolutely need Node, `tauri-plugin-js` provides kkrpc-based RPC to Bun/Node/Deno processes. |

## Feature Dependencies

```
[Google OAuth Login]
    └──requires──> [Deep Link Protocol Handler]
                       └──requires──> [tauri-plugin-deep-link registration]

[Session Persistence]
    └──requires──> [OAuth Login completed]
    └──requires──> [tauri-plugin-store or stronghold]

[Multi-Tenant Data Isolation]
    └──requires──> [Authentication (knows tenant_id)]
    └──requires──> [Database Schema with tenant_id columns]

[Local-First Database + Turso Sync]
    └──requires──> [libsql database setup]
    └──requires──> [Authentication (for Turso auth token)]
    └──enhances──> [Multi-Tenant Isolation]

[System Tray]
    └──enhances──> [Window Management (show/hide/focus)]

[Auto-Updater]
    └──requires──> [Signing keys configured]
    └──requires──> [Update server or static JSON endpoint]

[Embedded Axum Server]
    └──enhances──> [Local-First Database (REST API layer)]
    └──conflicts──> [Pure invoke()-only IPC (pick one pattern)]

[Native Dialogs] ──independent──> [can be used without auth]
[Window State Persistence] ──independent──> [no dependencies]
[Single Instance] ──independent──> [no dependencies]
```

### Dependency Notes

- **OAuth requires deep links:** Desktop OAuth flow opens system browser, redirects back via custom URI scheme (`myapp://callback`). `tauri-plugin-deep-link` handles this. Without it, you'd need a localhost redirect server (more complex).
- **Multi-tenancy requires auth:** Can't scope data to a tenant without knowing who the user is. Auth must come first.
- **Local-first enhances multi-tenancy:** Each tenant's data can live in a separate libsql database file (per-tenant SQLite pattern from Turso), or share a single DB with `tenant_id` column. Both patterns work.
- **Axum vs pure invoke():** Template has both. `invoke()` is simpler for CRUD. Axum shines for complex business logic, webhooks, or when you want a REST API that non-Tauri clients can also call. Recommendation: use `invoke()` for frontend↔Rust, Axum for any external-facing API.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [x] Tauri 2 + SvelteKit + Axum scaffolding — **exists**
- [x] moon build orchestration — **exists**
- [x] Responsive mobile-first layout base — **exists**
- [ ] Google OAuth login with deep link callback — **core requirement from PROJECT.md**
- [ ] Session persistence (JWT in tauri-plugin-store) — **required for auth to be useful**
- [ ] Multi-tenant data isolation (tenant_id in schema) — **core requirement from PROJECT.md**
- [ ] libsql database with basic schema — **required for any data persistence**
- [ ] Docker-compose for local dev (Redis, DB) — **core requirement from PROJECT.md**
- [ ] System tray with show/hide/quit — **expected desktop feature**
- [ ] Window state persistence — **expected desktop feature, trivial to add**
- [ ] Single instance lock — **expected desktop feature, trivial to add**
- [ ] Error boundaries + toast notifications — **required for production feel**

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] Auto-updater — **trigger: when ready to distribute to users**
- [ ] Turso cloud sync (embedded replica) — **trigger: when offline-first is validated as needed**
- [ ] Native notifications — **trigger: when background task alerts are needed**
- [ ] Deep link handler for custom protocols — **trigger: when OAuth is working, extend for other URLs**
- [ ] Global shortcuts — **trigger: when power-user features are added**
- [ ] Structured logging with file rotation — **trigger: when debugging production issues**

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Embedded Axum REST API — **why defer: adds complexity; invoke() suffices for boilerplate**
- [ ] Mobile (iOS/Android) support — **why defer: Tauri mobile is maturing; desktop-first is PROJECT.md scope**
- [ ] CRDT-based collaborative editing — **why defer: very complex, only needed for specific features**
- [ ] In-app purchase support — **why defer: monetization feature, not boilerplate concern**
- [ ] Biometric authentication — **why defer: mobile-only, outside desktop-first scope**

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Google OAuth login | HIGH | MEDIUM | P1 |
| Session persistence | HIGH | LOW | P1 |
| Multi-tenant isolation | HIGH | MEDIUM | P1 |
| libsql database | HIGH | MEDIUM | P1 |
| System tray | MEDIUM | MEDIUM | P1 |
| Window state persistence | MEDIUM | LOW | P1 |
| Single instance lock | MEDIUM | LOW | P1 |
| Docker-compose infra | MEDIUM | LOW | P1 |
| Error boundaries + toasts | HIGH | LOW | P1 |
| Auto-updater | MEDIUM | MEDIUM | P2 |
| Turso cloud sync | MEDIUM | HIGH | P2 |
| Native notifications | LOW | LOW | P2 |
| Global shortcuts | LOW | LOW | P3 |
| Embedded Axum API | LOW | HIGH | P3 |
| Mobile support | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Electron + Next.js | Tauri + Svelte (basic) | This Template |
|---------|-------------------|----------------------|---------------|
| Bundle size | 150MB+ | 8-15MB | 10-18MB (with Axum) |
| Auth boilerplate | Roll your own | Roll your own | Google OAuth built-in |
| Multi-tenancy | Not included | Not included | tenant_id scoping included |
| Backend server | Node.js (large) | None (invoke only) | Embedded Axum (optional) |
| Local database | better-sqlite3 | @tauri-apps/plugin-sql | libsql with encryption + sync |
| Build orchestration | npm scripts | npm scripts | moon (polyglot) |
| Desktop features | Limited native | Full Tauri plugins | Full Tauri plugins |
| Mobile support | N/A | Experimental | Tauri 2 ready |

## Desktop-Specific Feature Details

### System Tray (Required for v1)

Tauri 2 provides `TrayIconBuilder` API in Rust:

- **Left-click:** Show/focus main window (standard behavior)
- **Right-click:** Context menu with Show/Hide/Quit
- **Menu events:** Handled via `on_menu_event` callback
- **Tray icon events:** Click, DoubleClick, Enter, Move, Leave
- **Dynamic updates:** Change icon/text based on app state (e.g., badge for unread count)

Platform support: Windows, macOS, Linux. Not available on mobile.

### Native Dialogs

`tauri-plugin-dialog` (v2.x):

- File open dialog (single/multiple, filters)
- File save dialog
- Message boxes (info, warning, error, question)
- Desktop only (Android has no folder picker)

### Window Management

Tauri 2 built-in:

- Multi-window support with labeled windows
- Window menu (native OS menu bar)
- Custom titlebar (for frameless windows)
- Window events (resize, move, focus, close)
- `tauri-plugin-window-state` for persistence
- `tauri-plugin-positioner` for common positions

### Global Shortcuts

`tauri-plugin-global-shortcut` (v2.x):

- Register system-wide keyboard shortcuts
- Desktop only
- Use case: Quick capture, app activation

## Database Schema Design for Local-First

### Recommended Pattern: Shared DB with tenant_id

```sql
-- Every table includes tenant_id for multi-tenant isolation
CREATE TABLE users (
    id TEXT PRIMARY KEY,          -- UUID
    tenant_id TEXT NOT NULL,      -- Multi-tenant isolation
    email TEXT NOT NULL,
    google_id TEXT UNIQUE,
    display_name TEXT,
    avatar_url TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(tenant_id, email)
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    tenant_id TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Example business table
CREATE TABLE items (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    owner_id TEXT NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    content TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Indexes for tenant-scoped queries
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_sessions_tenant ON sessions(tenant_id);
CREATE INDEX idx_items_tenant ON items(tenant_id);
CREATE INDEX idx_items_owner ON items(owner_id);

-- Migration tracking (for Drizzle or manual migrations)
CREATE TABLE IF NOT EXISTS __drizzle_migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);
```

### Alternative: Per-Tenant SQLite Files

Turso's model allows each tenant to have their own database file:

```
~/.local/share/myapp/
  ├── tenants/
  │   ├── tenant_abc123.db
  │   └── tenant_def456.db
  └── global.db  (users, sessions — shared)
```

Pros: Stronger isolation, easier to sync individual tenants.
Cons: More complex connection management, harder cross-tenant queries.

**Recommendation for boilerplate:** Shared DB with `tenant_id` column. Simpler to implement, sufficient for most use cases. Per-tenant files can be added later if needed.

### Migration Strategy

Use Drizzle ORM's `sqlite-proxy` driver for type-safe queries + Vite's `import.meta.glob` for inlined migrations:

```typescript
// Migrations inlined at build time
const migrations = import.meta.glob<string>("./drizzle/*.sql", {
  eager: true,
  query: "?raw",
  import: "default",
});

// Run migrations via Tauri invoke
await migrate("sqlite:myapp.db", migrations);

// Drizzle queries via proxy (calls Rust through invoke())
const db = drizzle(createDrizzleProxy("sqlite:myapp.db"), { schema });
```

## Sources

- [Tauri 2 Plugin List](https://v2.tauri.app/plugin/) — Official plugin support table with platform compatibility
- [Tauri 2 System Tray Guide](https://v2.tauri.app/learn/system-tray/) — TrayIconBuilder API, event handling
- [Tauri 2 Window Menu](https://v2.tauri.app/learn/window-menu/) — Native menu creation
- [Tauri 2 Deep Linking](https://v2.tauri.app/plugin/deep-linking/) — Custom URI scheme registration
- [tauri-plugin-google-auth v0.5.1](https://crates.io/crates/tauri-plugin-google-auth) — Google OAuth plugin (Jan 2026)
- [tauri-plugin-libsql v0.1.0](https://crates.io/crates/tauri-plugin-libsql) — libsql + encryption + Turso sync (Feb 2026)
- [tauri-plugin-auth-session v0.2.2](https://crates.io/crates/tauri-plugin-auth-session) — In-app OAuth for mobile (Mar 2026)
- [Building Local-First Tauri App](https://dev.to/huakun/building-a-local-first-tauri-app-with-drizzle-orm-encryption-and-turso-sync-31pn) — Drizzle + libsql + Turso patterns (Feb 2026)
- [Shipped Tauri 2 + Svelte 5 + Axum](https://www.reddit.com/r/tauri/comments/1s4ah2f/) — Real production app: 18MB, 114 API routes (Mar 2026)
- [Local-First Design Patterns 2026](https://mongoose.cloud/design-patterns-local-first-2026/) — Journal + CRDT + optimistic UI patterns
- [Tauri Authentication Guide](https://www.reddit.com/r/tauri/comments/1ozm5go/) — OAuth patterns for GitLab, GitHub, Google, Apple (Nov 2025)
- [Supabase + Google OAuth in Tauri 2](https://medium.com/@nathancovey23/supabase-google-oauth-in-a-tauri-2-0-macos-app-with-deep-links-f8876375cb0a) — Deep link OAuth flow (Apr 2025)
- [Multi-Tenant SaaS Architecture 2026](https://dev.to/waqarhabib/building-a-multi-tenant-saas-app-with-react-and-nodejs-in-2026-31ih) — tenant_id isolation patterns (Mar 2026)
- [Turso: Give Each User Their Own SQLite](https://turso.tech/blog/give-each-of-your-users-their-own-sqlite-database-b74445f4) — Per-tenant SQLite pattern

---

*Feature research for: Tauri + SvelteKit + Axum full-stack desktop boilerplate*
*Researched: 2026-03-28*
