# Architecture Research

**Domain:** Tauri 2 + SvelteKit + Axum Full-Stack Desktop Application
**Researched:** 2026-03-28
**Confidence:** HIGH

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SvelteKit Frontend (WebView)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Routes      │  │ Components  │  │ Stores      │  │ IPC Client  │        │
│  │ (+page.sv)  │  │ (*.svelte)  │  │ ($state)    │  │ (invoke)    │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                │                │
├─────────┴────────────────┴────────────────┴────────────────┴────────────────┤
│                              Svelte 5 Runes                                  │
│         $state (reactive) │ $derived (computed) │ $effect (side-effects)     │
├───────────────────────────┴─────────────────────┴────────────────────────────┤
│                          Tauri IPC Boundary (JSON-RPC)                        │
│         invoke('command', { args }) → Result<T, E>                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                            Tauri Core (Rust)                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Commands    │  │ Plugins     │  │ State Mgmt  │  │ Events      │        │
│  │ (#[command])│  │ (Builder)   │  │ (app.manage)│  │ (emit/listen│        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                │                │
├─────────┴────────────────┴────────────────┴────────────────┴────────────────┤
│                          Application Layer                                    │
│  ┌──────────────────────────────────────────────────────────────────┐        │
│  │  application crate — Use case orchestration, service layer       │        │
│  └──────────────────────────────────────────────────────────────────┘        │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Domain Layer                                        │
│  ┌──────────────────────────────────────────────────────────────────┐        │
│  │  domain crate — Business rules, entities, value objects          │        │
│  └──────────────────────────────────────────────────────────────────┘        │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Infrastructure Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                      │
│  │ runtime_tauri│  │runtime_server│  │ libsql/turso │                      │
│  │ (Tauri glue) │  │ (Axum glue)  │  │ (database)   │                      │
│  └──────────────┘  └──────────────┘  └──────────────┘                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Shared Contracts                                    │
│  ┌──────────────────────────────────────────────────────────────────┐        │
│  │  shared_contracts crate — DTOs, schemas, type definitions        │        │
│  └──────────────────────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Communication |
|-----------|----------------|---------------|
| **SvelteKit Frontend** | UI rendering, user interaction, SPA routing | Tauri IPC (`invoke()`) to Rust backend |
| **Svelte 5 Runes** | Reactive state management within components | `$state`, `$derived`, `$effect` |
| **Tauri IPC** | JSON-RPC bridge between WebView and Rust | Typed `invoke()` calls, bidirectional events |
| **Tauri Commands** | Request handlers exposed to frontend | Receives IPC calls, returns `Result<T, E>` |
| **Tauri Plugins** | Modular capability extension | Lifecycle hooks, commands, state management |
| **Application Crate** | Use case orchestration, service logic | Calls domain, uses infrastructure |
| **Domain Crate** | Pure business rules, zero external deps | No framework dependencies |
| **Runtime Tauri** | Tauri-specific glue (commands, plugin setup) | Depends on domain + application |
| **Runtime Server** | Axum HTTP server glue (REST endpoints) | Depends on domain + application |
| **Shared Contracts** | DTOs shared between frontend and backend | TypeScript types generated from Rust |
| **libsql** | Embedded SQLite-compatible database | Direct file access, optional Turso sync |

---

## Tauri IPC vs REST API: When to Use Which

### Decision Matrix

| Use Case | Use Tauri IPC | Use REST API (Axum) |
|----------|:------------:|:------------------:|
| Frontend ↔ Backend within same app | ✅ | ❌ |
| External API consumers (mobile app, CLI) | ❌ | ✅ |
| OAuth callbacks / webhook receivers | ❌ | ✅ |
| Third-party integrations | ❌ | ✅ |
| Database CRUD operations | ✅ | ✅ (if external access needed) |
| File system operations | ✅ | ❌ |
| Native OS features (dialogs, tray) | ✅ | ❌ |
| Public API for other services | ❌ | ✅ |

### Performance Comparison

| Metric | Tauri IPC (`invoke()`) | HTTP to localhost |
|--------|:---------------------:|:-----------------:|
| Latency | **1-5ms** | 50-200ms |
| Type safety | ✅ Compile-time checked | ❌ Runtime only |
| Serialization | JSON-RPC (built-in) | JSON (manual) |
| Overhead | Minimal (direct function call) | Network stack + HTTP parsing |

### Recommendation for This Template

**Use Tauri IPC (`invoke()`) as the primary communication channel.** The frontend and backend live in the same process — adding HTTP for local communication is an anti-pattern.

**Use Axum REST API only when:**
1. External services need to call your app (webhooks, OAuth callbacks)
2. You're building a companion server that non-Tauri clients use
3. You need to expose a public API

**Implementation pattern:**
```typescript
// ✅ Correct: Tauri IPC for frontend ↔ Rust
import { invoke } from '@tauri-apps/api/core';

const data = await invoke<{ id: string; name: string }[]>('get_items', {
  tenantId: currentTenant
});

// ❌ Wrong: HTTP to local Axum
const response = await fetch('http://localhost:3000/api/items');
const data = await response.json();
```

```rust
// Tauri command handler
#[tauri::command]
async fn get_items(tenant_id: String) -> Result<Vec<Item>, String> {
    // Calls application layer, returns typed result
    app_service.list_items(&tenant_id).await.map_err(|e| e.to_string())
}
```

---

## Recommended Project Structure

```
tauri-sveltekit-axum-moon-template/
├── apps/
│   └── desktop-ui/                  # SvelteKit + Tauri desktop app
│       ├── src/
│       │   ├── routes/              # SvelteKit file-based routing
│       │   │   ├── +layout.svelte   # Root layout
│       │   │   ├── +layout.ts       # SPA config (ssr=false)
│       │   │   ├── +page.svelte     # Pages
│       │   │   └── (groups)/        # Route groups
│       │   ├── lib/
│       │   │   ├── components/      # Reusable Svelte components
│       │   │   ├── stores/          # Svelte 5 state ($state runes)
│       │   │   ├── ipc/             # Tauri IPC wrapper functions
│       │   │   │   ├── auth.ts      # Auth-related invoke calls
│       │   │   │   ├── data.ts      # Data CRUD invoke calls
│       │   │   │   └── system.ts    # System invoke calls
│       │   │   ├── types/           # TypeScript types (from shared_contracts)
│       │   │   └── utils/           # Utility functions
│       │   └── app.html             # HTML shell
│       ├── src-tauri/               # Tauri Rust project
│       │   ├── src/
│       │   │   ├── lib.rs           # Tauri builder, plugin registration
│       │   │   └── main.rs          # Entry point
│       │   ├── capabilities/        # Tauri 2 permissions
│       │   │   └── default.json     # Capability definitions
│       │   ├── icons/               # App icons
│       │   ├── Cargo.toml           # Rust dependencies
│       │   └── tauri.conf.json      # Tauri configuration
│       ├── package.json
│       ├── svelte.config.js         # adapter-static, SPA mode
│       └── vite.config.ts
├── crates/
│   ├── domain/                      # Pure business logic
│   │   ├── src/
│   │   │   ├── entities/            # Domain entities
│   │   │   ├── value_objects/       # Value objects
│   │   │   └── errors/              # Domain errors
│   │   └── Cargo.toml               # No external deps (except serde)
│   ├── application/                 # Use case orchestration
│   │   ├── src/
│   │   │   ├── services/            # Application services
│   │   │   ├── ports/               # Trait interfaces (Repository, etc.)
│   │   │   └── dto/                 # Internal DTOs
│   │   └── Cargo.toml               # Depends on domain
│   ├── runtime_tauri/               # Tauri integration
│   │   ├── src/
│   │   │   ├── commands/            # #[tauri::command] handlers
│   │   │   ├── plugins/             # Custom plugin definitions
│   │   │   └── state.rs             # App state management
│   │   └── Cargo.toml               # Depends on domain, application, tauri
│   ├── runtime_server/              # Axum integration
│   │   ├── src/
│   │   │   ├── routes/              # HTTP route handlers
│   │   │   ├── middleware/          # Tower middleware
│   │   │   └── extractors/          # Custom Axum extractors
│   │   └── Cargo.toml               # Depends on domain, application, axum
│   └── shared_contracts/            # Shared type definitions
│       ├── src/
│       │   └── lib.rs               # DTOs, schemas (TypeScript-exportable)
│       └── Cargo.toml               # Depends on serde, ts-rs (optional)
├── .moon/
│   └── workspace.yml                # Moon workspace config
├── moon.yml                         # Root-level tasks
├── Cargo.toml                       # Workspace definition
└── docker-compose.yml               # Dev infrastructure
```

### Structure Rationale

- **`domain/`**: Pure Rust with zero framework dependencies. Contains business entities, value objects, and domain errors. Testable in isolation.
- **`application/`**: Orchestration layer. Defines trait interfaces (ports) for infrastructure, implements use cases. Depends only on domain.
- **`runtime_tauri/`**: Glue layer for Tauri. Maps `#[tauri::command]` to application services. Manages Tauri state and plugin initialization.
- **`runtime_server/`**: Glue layer for Axum. Maps HTTP routes to application services. Tower middleware for CORS, auth, logging.
- **`shared_contracts/`**: DTOs used by both frontend (via ts-rs codegen) and backend. Single source of truth for types.
- **`desktop-ui/`**: SvelteKit SPA consumed by Tauri WebView. All IPC calls go through `lib/ipc/` wrapper functions.

---

## Architectural Patterns

### Pattern 1: Command-Query Separation via Tauri IPC

**What:** Separate read (query) and write (command) operations at the IPC boundary. Each Tauri command is either a query (returns data) or a command (mutates state).

**When to use:** Always in this template. Maps cleanly to application service methods.

**Trade-offs:** Slightly more boilerplate than generic CRUD, but clearer intent and easier testing.

**Example:**
```rust
// Query: returns data, no side effects
#[tauri::command]
async fn get_items(tenant_id: String) -> Result<Vec<ItemDto>, String> {
    let service = app_service();
    service.list_items(&tenant_id).await.map_err(|e| e.to_string())
}

// Command: mutates state, returns confirmation
#[tauri::command]
async fn create_item(tenant_id: String, title: String) -> Result<ItemDto, String> {
    let service = app_service();
    service.create_item(&tenant_id, &title).await.map_err(|e| e.to_string())
}
```

```typescript
// Frontend: type-safe wrappers in lib/ipc/data.ts
import { invoke } from '@tauri-apps/api/core';

export async function getItems(tenantId: string): Promise<Item[]> {
  return invoke('get_items', { tenantId });
}

export async function createItem(tenantId: string, title: string): Promise<Item> {
  return invoke('create_item', { tenantId, title });
}
```

### Pattern 2: Svelte 5 Runes for State Management

**What:** Use Svelte 5's `$state`, `$derived`, and `$effect` runes for reactive state. No external state management library needed.

**When to use:** All component-level and app-level state. Use `$state` for mutable data, `$derived` for computed values, `$effect` for side effects.

**Trade-offs:** Svelte 5 runes are stable but require understanding the proxy-based reactivity model. `$state.raw` for large datasets to avoid unnecessary proxying.

**Example:**
```svelte
<script lang="ts">
  import { getItems, createItem } from '$lib/ipc/data';

  // Reactive state
  let items = $state<Item[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Computed state
  let itemCount = $derived(items.length);
  let completedItems = $derived(items.filter(i => i.done));

  // Side effects
  $effect(() => {
    // Fetch items when tenant changes
    if (currentTenant) {
      loading = true;
      getItems(currentTenant)
        .then(data => items = data)
        .catch(e => error = e.message)
        .finally(() => loading = false);
    }
  });

  // Mutation
  async function addItem(title: string) {
    const newItem = await createItem(currentTenant, title);
    items.push(newItem); // Deep reactivity handles this
  }
</script>
```

**State hierarchy for this template:**
```
$state (local component)     → Component-scoped UI state
$state (module-level)        → Shared reactive stores
$derived (computed)          → Filtered/transformed views of state
$effect (side effects)       → IPC calls, localStorage sync, DOM updates
$effect.pre (pre-DOM)        → Scroll position, measurement
```

### Pattern 3: Tenant-Scoped Repository Pattern

**What:** All data access goes through repository traits that automatically scope queries by `tenant_id`. Application services accept tenant context and pass it through.

**When to use:** All data persistence. Prevents accidental cross-tenant data leaks.

**Trade-offs:** Requires threading tenant_id through the call stack. Mitigated by using a context/extractor pattern.

**Example:**
```rust
// Domain: Repository trait (port)
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_by_tenant(&self, tenant_id: &str) -> Result<Vec<Item>>;
    async fn create(&self, item: &Item) -> Result<()>;
}

// Application: Service uses repository
pub struct ItemService<R: ItemRepository> {
    repo: R,
}

impl<R: ItemRepository> ItemService<R> {
    pub async fn list_items(&self, tenant_id: &str) -> Result<Vec<Item>> {
        self.repo.find_by_tenant(tenant_id).await
    }
}

// Runtime Tauri: Command calls service
#[tauri::command]
async fn get_items(
    state: State<'_, AppState>,
    tenant_id: String,
) -> Result<Vec<ItemDto>, String> {
    let service = state.item_service.read().await;
    service.list_items(&tenant_id).await
        .map(|items| items.into_iter().map(Into::into).collect())
        .map_err(|e| e.to_string())
}
```

### Pattern 4: Axum Middleware Stack with Tower

**What:** Compose middleware layers using `tower::ServiceBuilder` for the Axum server. Applied top-to-bottom: tracing → compression → CORS → timeout.

**When to use:** When the embedded Axum server is used (webhook receivers, external API).

**Trade-offs:** Tower middleware is powerful but has a learning curve. For simple cases, Axum's built-in extractors suffice.

**Example:**
```rust
use axum::{routing::post, Router};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use std::time::Duration;

pub fn create_router() -> Router {
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    Router::new()
        .route("/webhook/oauth", post(handle_oauth_callback))
        .layer(middleware_stack)
}
```

### Pattern 5: Tauri 2 Plugin Architecture

**What:** Modular capability extension via the Tauri plugin system. Each plugin provides commands, lifecycle hooks, and optional mobile support.

**When to use:** When adding reusable capabilities (auth, file access, notifications). Both official plugins and custom plugins follow the same pattern.

**Trade-offs:** Plugin initialization order matters. State managed by plugins is global to the app.

**Plugin lifecycle hooks:**
```
setup()           → Plugin initialization, state registration
on_navigation()   → WebView navigation validation
on_webview_ready()→ Window creation, initialization scripts
on_event()        → Event loop events (window close, exit)
on_drop()         → Cleanup on plugin destruction
```

**Example custom plugin:**
```rust
// crates/runtime_tauri/src/plugins/auth.rs
use tauri::plugin::{Builder, TauriPlugin, Runtime};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("auth")
        .setup(|app, api| {
            // Register state, load config
            let config = api.config();
            app.manage(AuthState::new(config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::logout,
            commands::get_session,
        ])
        .on_event(|app, event| {
            // Handle exit, save state
        })
        .build()
}
```

**Official plugins used in this template:**
| Plugin | Purpose | Permissions Required |
|--------|---------|---------------------|
| `tauri-plugin-shell` | Execute system commands, open URLs | `shell:default`, `shell:allow-execute` |
| `tauri-plugin-dialog` | Native file/save/message dialogs | `dialog:default` |
| `tauri-plugin-store` | Persistent key-value storage | `store:default` |
| `tauri-plugin-fs` | File system access | `fs:default`, `fs:allow-read`, `fs:allow-write` |
| `tauri-plugin-window-state` | Persist window size/position | `window-state:default` |

---

## Data Flow

### Frontend → Backend (IPC)

```
User Action (click, form submit)
    ↓
Svelte Component ($state mutation or function call)
    ↓
IPC Wrapper (lib/ipc/*.ts) — type-safe invoke() call
    ↓
Tauri IPC Layer (JSON-RPC serialization)
    ↓
#[tauri::command] handler (runtime_tauri)
    ↓
Application Service (application crate)
    ↓
Domain Logic (domain crate)
    ↓
Repository/Infrastructure (libsql, fs, etc.)
    ↓
Result<T, E> — serialized back through IPC
    ↓
Svelte Component ($state update, $effect triggered)
```

### Backend → Frontend (Events)

```
Rust Background Task / Plugin
    ↓
app.emit("event-name", payload)
    ↓
Tauri Event System
    ↓
Frontend: listen("event-name", handler)
    ↓
$state update → UI re-renders
```

### State Management Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Module-Level Store                           │
│  export let currentUser = $state<User | null>(null);            │
│  export let currentTenant = $derived(currentUser?.tenantId);    │
└───────────────────────────┬─────────────────────────────────────┘
                            ↓ (import + reactive)
┌─────────────────────────────────────────────────────────────────┐
│                     Component State                              │
│  let items = $state<Item[]>([]);                                │
│  let filtered = $derived(items.filter(...));                    │
│  $effect(() => { getItems(currentTenant).then(...); });         │
└───────────────────────────┬─────────────────────────────────────┘
                            ↓ (reactive update)
┌─────────────────────────────────────────────────────────────────┐
│                     DOM (automatic)                              │
│  {#each filtered as item} ... {/each}                           │
└─────────────────────────────────────────────────────────────────┘
```

### Key Data Flows

1. **Authentication Flow:** User clicks Login → OAuth opens system browser → Deep link callback → `tauri-plugin-deep-link` captures → Tauri command processes token → `tauri-plugin-store` persists JWT → `$state` updates → UI reflects logged-in state.

2. **Data CRUD Flow:** User creates item → Svelte component calls `ipc.createItem()` → `invoke('create_item', ...)` → Tauri command → Application service → Domain validation → Repository writes to libsql → Result flows back → `$state` update → UI re-renders.

3. **Background Sync Flow:** Timer triggers → Rust background task → libsql sync with Turso → `app.emit("sync-complete", data)` → Frontend `listen()` → `$state` update → UI notification.

---

## Database Layer Architecture (libsql)

### Architecture Decision

**Use `libsql` crate for embedded SQLite-compatible database.** This is a pure Rust implementation with:
- SQLite wire compatibility (existing SQL knowledge applies)
- Concurrent read support (unlike rusqlite)
- Optional Turso cloud sync for offline-first
- Works embedded in desktop apps (no server needed)

### Database Layer Structure

```
┌──────────────────────────────────────────────┐
│              Application Service              │
│  (calls repository trait methods)             │
└───────────────────┬──────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│           Repository Trait (Port)             │
│  trait ItemRepository {                       │
│    async fn find_by_tenant(...) -> Result<>;  │
│    async fn create(...) -> Result<>;          │
│  }                                            │
└───────────────────┬──────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│        LibsqlRepository (Adapter)             │
│  - Holds libsql::Connection                   │
│  - Implements ItemRepository                  │
│  - Handles SQL queries + mapping              │
└───────────────────┬──────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│           libsql Database File                │
│  ~/.local/share/com.example.app/app.db        │
│  - Auto-migrated on startup                   │
│  - tenant_id scoped queries                   │
└──────────────────────────────────────────────┘
```

### Schema Pattern (Multi-Tenant)

```sql
-- Every data table includes tenant_id
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    email TEXT NOT NULL,
    google_id TEXT UNIQUE,
    display_name TEXT,
    avatar_url TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(tenant_id, email)
);

CREATE TABLE items (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    owner_id TEXT NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    content TEXT,
    done INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Indexes for tenant-scoped queries
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_items_tenant ON items(tenant_id);
CREATE INDEX idx_items_owner ON items(owner_id);
```

### Migration Strategy

**Use `rusqlite_migration` for embedded migrations.** Run on app startup.

```rust
use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up("CREATE TABLE IF NOT EXISTS users (...)"),
        M::up("CREATE TABLE IF NOT EXISTS items (...)"),
    ])
}

// In runtime_tauri setup:
fn setup_database(app: &tauri::App) -> Result<()> {
    let db_path = app.path().app_local_data_dir()?.join("app.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    
    let mut conn = libsql::Connection::open(&db_path)?;
    migrations().to_latest(&mut conn)?;
    
    app.manage(DatabaseConnection(conn));
    Ok(())
}
```

---

## Build & Release Workflows with moon

### moon Configuration Strategy

moon excels at polyglot monorepo orchestration. It provides:
- **Smart hashing** — Only rebuilds changed projects
- **Dependency graph** — Parallel execution where possible
- **Integrated toolchain** — Ensures consistent Rust/Node versions

### Recommended moon Task Structure

```yaml
# moon.yml (root)
tasks:
  # Rust workspace tasks
  rust-build:
    command: 'cargo build'
    inputs:
      - '@globs(sources)'
      - 'Cargo.toml'
    platform: 'system'

  rust-test:
    command: 'cargo test'
    inputs:
      - '@globs(sources)'
    platform: 'system'

  rust-check:
    command: 'cargo check'
    inputs:
      - '@globs(sources)'
      - 'Cargo.toml'
    platform: 'system'

  rust-lint:
    command: 'cargo clippy -- -D warnings'
    inputs:
      - '@globs(sources)'
    platform: 'system'

  rust-format:
    command: 'cargo fmt --all --check'
    inputs:
      - '@globs(sources)'
    platform: 'system'

  # Frontend tasks
  fe-build:
    command: 'bun run build'
    inputs:
      - 'apps/desktop-ui/src/**/*'
      - 'apps/desktop-ui/package.json'
    platform: 'system'

  fe-check:
    command: 'bun run check'
    inputs:
      - 'apps/desktop-ui/src/**/*'
    platform: 'system'

  fe-lint:
    command: 'bun run lint'
    inputs:
      - 'apps/desktop-ui/src/**/*'
    platform: 'system'

  # Tauri build
  tauri-build:
    command: 'bun run tauri build'
    deps:
      - '~:rust-check'
      - '~:fe-build'
    inputs:
      - 'apps/desktop-ui/src-tauri/**/*'
      - 'apps/desktop-ui/src/**/*'
    platform: 'system'

  # Full CI pipeline
  ci:
    deps:
      - '~:rust-lint'
      - '~:rust-test'
      - '~:fe-lint'
      - '~:fe-check'
    platform: 'system'
```

### Project-Level moon.yml

```yaml
# apps/desktop-ui/moon.yml
language: 'typescript'
type: 'application'

tasks:
  dev:
    command: 'bun run tauri dev'
    local: true

  build:
    command: 'bun run build'
    deps:
      - '^:build'  # Build dependencies first

  check:
    command: 'svelte-check --tsconfig ./tsconfig.json'
```

```yaml
# crates/domain/moon.yml
language: 'rust'
type: 'library'

tasks:
  build:
    command: 'cargo check'
    deps:
      - '^:build'

  test:
    command: 'cargo test'
```

### Build Pipeline Order

```
┌─────────────────────────────────────────────────────────────────┐
│                         moon ci                                  │
├─────────────────────────────────────────────────────────────────┤
│  rust-lint ──┐                                                  │
│  rust-test ──┤── (parallel)                                     │
│  fe-lint ────┤                                                  │
│  fe-check ───┘                                                  │
│       ↓                                                          │
│  If all pass → ready for tauri-build                            │
│       ↓                                                          │
│  tauri-build (depends on rust-check + fe-build)                 │
│       ↓                                                          │
│  Output: platform-specific binaries                             │
└─────────────────────────────────────────────────────────────────┘
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: HTTP to localhost for IPC

**What people do:** Use `fetch('http://localhost:3000/api/...')` to communicate with the embedded Axum server.

**Why it's wrong:** Adds 50-200ms latency per request. WebView network stack overhead. No type safety. Unnecessary complexity.

**Do this instead:** Use Tauri `invoke()` for all frontend↔Rust communication. Reserve Axum for external-facing endpoints only.

### Anti-Pattern 2: Business Logic in Commands

**What people do:** Put validation, business rules, and data access directly in `#[tauri::command]` functions.

**Why it's wrong:** Untestable without Tauri runtime. Tight coupling to Tauri framework. Can't reuse logic in Axum routes.

**Do this instead:** Commands call application services. Business logic lives in the domain crate (testable, framework-independent).

### Anti-Pattern 3: Global Mutable State in Svelte

**What people do:** Export mutable `$state` variables from module-level stores and mutate them directly from any component.

**Why it's wrong:** Hard to track mutations. Race conditions. No clear data flow.

**Do this instead:** Use functions that encapsulate state mutations. Prefer `$derived` for computed values over manual updates in `$effect`.

### Anti-Pattern 4: Skipping Tauri 2 Capabilities

**What people do:** Add plugins via `cargo add` but don't configure capabilities in `capabilities/default.json`.

**Why it's wrong:** Works in dev mode (relaxed permissions) but fails in production with cryptic "Permission denied" errors.

**Do this instead:** Always add required permissions to capability files when installing a plugin.

### Anti-Pattern 5: Hardcoded Database Paths

**What people do:** Use `"./app.db"` or relative paths for the database file.

**Why it's wrong:** Breaks when app is installed (not running from source directory). Different paths on Windows/macOS/Linux.

**Do this instead:** Use `app.path().app_local_data_dir()` to get the OS-appropriate data directory.

---

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| **Single user (desktop)** | Embedded libsql, Tauri IPC only, no Axum server needed |
| **Multi-user (shared desktop)** | Add `tenant_id` scoping, tenant-aware repositories |
| **Team deployment** | Add Turso cloud sync, embedded replica pattern |
| **Public API needed** | Enable Axum server, add REST endpoints, Tower middleware |
| **Mobile companion** | Same Tauri 2 codebase, responsive SvelteKit, platform detection |

### Scaling Priorities

1. **First bottleneck:** Database writes under concurrent use → Use libsql's WAL mode, batch writes
2. **Second bottleneck:** Large IPC payloads (>1MB) → Chunk data, use streaming, or file-based transfer
3. **Third bottleneck:** Frontend bundle size → Lazy load routes, code split at route boundaries

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Frontend ↔ Tauri Commands | `invoke()` IPC (JSON-RPC) | Type-safe, 1-5ms latency |
| Tauri Commands ↔ Application | Direct function calls | Rust trait-based |
| Application ↔ Domain | Direct function calls | No framework deps |
| Application ↔ Infrastructure | Trait implementations (ports/adapters) | Swappable |
| Tauri ↔ Axum | Shared application services | Same business logic, different entry points |
| Frontend ↔ External APIs | `tauri-plugin-http` or `fetch()` | OAuth, third-party services |

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Google OAuth | Deep link callback via `tauri-plugin-deep-link` | System browser → redirect → capture |
| Turso Cloud | libsql sync() on demand | Optional, for cloud backup |
| Update Server | `tauri-plugin-updater` + static JSON endpoint | For auto-updates |
| OS Features | Tauri plugins (shell, dialog, fs, clipboard) | Platform-native APIs |

---

## Sources

- [Tauri 2 IPC Documentation](https://v2.tauri.app/concept/inter-process-communication/) — Commands API, JSON-RPC protocol
- [Tauri 2 Plugin Development](https://v2.tauri.app/develop/plugins/) — Plugin lifecycle, commands, permissions, state
- [Tauri 2 Security Model](https://v2.tauri.app/security/) — Capabilities, permissions, CSP
- [Svelte 5 Runes](https://svelte.dev/docs/svelte/$state) — `$state`, `$derived`, `$effect` documentation
- [Axum Middleware](https://context7.com/tokio-rs/axum) — Tower integration, ServiceBuilder patterns
- [moon Documentation](https://moonrepo.dev/docs) — Task orchestration, polyglot support, Rust tier 3
- [libSQL GitHub](https://github.com/tursodatabase/libsql) — Embedded SQLite with Turso sync
- [Tauri 2 System Tray](https://v2.tauri.app/learn/system-tray/) — TrayIconBuilder API
- [Stack Research](./STACK.md) — Technology versions and alternatives
- [Pitfalls Research](./PITFALLS.md) — Anti-patterns and recovery strategies

---

*Architecture research for: Tauri + SvelteKit + Axum full-stack desktop boilerplate*
*Researched: 2026-03-28*
