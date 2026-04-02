# Phase 4 Research: 最小功能实现

**Researched:** 2026-04-02
**Phase:** 04-minimal-feature-implementation
**Requirements:** AUTH-01, COUNTER-01, ADMIN-01, AGENT-01

## Research Summary

This phase implements 4 features (Auth, Counter, Admin, Agent Chat) using the feature+adapter pattern established in Phase 3. All features share a common IPC dual-path architecture: Tauri `invoke()` for native, `fetch()` to Axum server for browser.

## Standard Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Frontend | SvelteKit 2 + Svelte 5 | runes mode ($state/$effect/$props) |
| Backend | Axum | 0.8.x |
| Native | Tauri v2 | 2.10.3 |
| IPC | Tauri invoke + HTTP fetch | dual-path runtime detection |
| Database | LibSQL (embedded) + SurrealDB (cloud) | via domain ports |
| Contracts | ts-rs + utoipa | single truth source |
| Build | moon + Bun | unified task graph |

## Architecture Patterns

### Feature Crate Pattern (from Phase 3)
```
packages/features/{name}/
├── Cargo.toml          # depends on domain + usecases (NOT adapters)
├── src/
│   ├── lib.rs          # usecases / services
│   └── ipc/            # Tauri command wrappers (optional)
```

### Adapter Pattern (from Phase 3)
```
packages/adapters/{domain}/{provider}/
├── Cargo.toml          # depends on domain ports
└── src/
    └── lib.rs          # provider-specific implementation
```

### IPC Dual-Path Pattern (D-06, D-14)
```typescript
// Frontend runtime detection
if (typeof window.__TAURI__ !== 'undefined') {
  return invoke('counter_get_value');
} else {
  return fetch('/api/counter/value').then(r => r.json());
}
```

## Key Migration Targets

### Auth Refactoring (AUTH-01)
**Source:** `packages/adapters/hosts/tauri/src/commands/auth.rs` (422 lines)
**Targets:**
- `packages/adapters/auth/google/` — New adapter crate with OAuth PKCE logic
- `packages/features/auth/` — AuthService trait + usecases
- Keep UserProfile/AuthSession in runtime_tauri (Tauri store types)
- Use TokenPair/UserSession from contracts_auth (API layer types)

**Key insight:** auth.rs already has a clean separation. Migration is primarily moving code to new crates with trait extraction, not rewriting.

### Counter (COUNTER-01)
**Source:** `apps/client/web/app/src/routes/(app)/counter/+page.svelte` (pure frontend, 48 lines)
**Targets:**
- `packages/features/counter/` — CounterService trait + LibSQL persistence
- counter table: `id INTEGER PRIMARY KEY, value INTEGER, updated_at TEXT`
- Tauri commands + HTTP endpoints for increment/decrement/reset/get_value
- Frontend IPC dual-path integration

### Admin (ADMIN-01)
**Source:** `apps/client/web/app/src/routes/(app)/admin/+page.svelte` (mock data, 56 lines)
**Targets:**
- `packages/features/admin/` — AdminService trait for dashboard data
- Real data from: tenant count, counter value, last login time, app version
- Uses existing TenantService from usecases + CounterService from feature-counter

### Agent Chat (AGENT-01)
**Source:** None (new feature)
**Targets:**
- `packages/features/agent/` — AgentService trait with chat + tools
- `packages/contracts/api/` — ChatMessage, ToolCall, AgentConfig DTOs
- LibSQL persistence: conversations + messages tables
- OpenAI-compatible API integration (user-provided key)
- Streaming: Axum SSE for browser, Tauri event channel for native

## Critical Decisions

### Agent Contracts Placement
Agent DTOs (ChatMessage, ToolCall, AgentConfig) should go in `packages/contracts/api/` since they're route-level shared types, not auth-specific. This keeps contracts_auth focused on auth tokens/sessions.

### IPC Module Placement (D-16)
Each feature owns its IPC module:
```
packages/features/auth/ipc/     # auth IPC
packages/features/counter/ipc/  # counter IPC  
packages/features/admin/ipc/    # admin IPC
packages/features/agent/ipc/    # agent IPC
```
NOT in shared `lib/ipc/`.

### Streaming Architecture
Two approaches evaluated:
1. **SSE via Axum:** Browser uses EventSource, native uses reqwest streaming — simpler, works for both paths
2. **Tauri event channel:** Native-only, needs separate SSE for browser — more complex

**Recommendation:** SSE via Axum for both paths. In Tauri environment, the frontend still calls `fetch('http://localhost:3000/api/agent/chat')` with SSE streaming. This eliminates the need for Tauri event channel complexity.

### Auth Adapter Placement (D-02)
`packages/adapters/auth/google/` — NOT `packages/adapters/auth/oauth/`. Each OAuth provider gets its own adapter crate.

## Architecture Patterns

### LibSQL Migration Pattern
Existing pattern from storage_libsql:
```rust
pub async fn run_tenant_migrations(db: &EmbeddedLibSql) -> Result<(), LibSqlError> {
    db.execute("CREATE TABLE IF NOT EXISTS ...", vec![]).await?;
    Ok(())
}
```

### Tauri Command Pattern
```rust
#[tauri::command]
pub async fn counter_increment(state: State<'_, AppState>) -> Result<i64, String> {
    // ... use AppState.db (EmbeddedLibSql)
}
```

### Axum Route Pattern
```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/counter/increment", post(increment_handler))
}
```

### Svelte 5 Page Pattern
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  let count = $state(0);
  $effect(() => { /* init logic */ });
</script>
```

## Don't Hand-Roll

- **Auth token refresh:** Existing `start_refresh_timer` in auth.rs handles this
- **LibSQL schema management:** Use `run_tenant_migrations` pattern
- **HTTP client:** AppState already has `reqwest::Client` with connection pooling
- **CORS:** Axum server already configured with tower-http CORS middleware
- **Tauri command structure:** Follow existing auth/config command pattern

## Common Pitfalls

1. **Feature crate depending on adapters** — Feature crates must depend on domain + usecases only. Adapter dependencies are added at the host level (runtime_tauri, native-tauri, servers/api)
2. **IPC path detection** — Use `window.__TAURI__` check, NOT `import.meta.env.TAURI` (env var not reliable at runtime)
3. **LibSQL in-memory** — Current setup uses `:memory:`, data doesn't persist across restarts. Counter needs persistent file path for real usage
4. **OpenAI API key storage** — In Tauri store (not LibSQL), since it's user config not app data
5. **Streaming responses** — Must handle connection drops gracefully, especially on mobile

## Validation Architecture

Phase 4 needs validation for:
- Each feature crate compiles independently
- Tauri commands are registered in generate_handler!
- Axum routes are mounted in server router
- Frontend pages load and interact correctly
- IPC dual-path works in both Tauri and browser modes

**Recommendation:** Run `cargo check --workspace` after each plan, plus targeted `cargo test -p feature-{name}` for each feature crate.

## Security Considerations

- **OAuth state:** CSRF protection via state parameter (already in auth.rs)
- **PKCE:** Code challenge/verifier flow (already in auth.rs)
- **API key:** Stored in Tauri store, NOT in LibSQL or exposed to Axum server
- **Agent tools:** Read-only operations only (get_counter_value, list_tenants, get_system_status)
- **Input validation:** Agent user input must be sanitized before LLM API call

## Requirements Mapping

| REQ-ID | Feature | Key Artifacts |
|--------|---------|--------------|
| AUTH-01 | Auth refactor | feature-auth, adapter-google, contracts_auth |
| COUNTER-01 | Counter | feature-counter, counter table, IPC dual-path |
| ADMIN-01 | Admin dashboard | feature-admin, real data from usecases |
| AGENT-01 | Agent chat | feature-agent, contracts_api (DTOs), streaming |

## Resource Needs

- **New crates:** 5 (feature-auth, feature-counter, feature-admin, feature-agent, adapter-google)
- **New contracts:** Agent DTOs in contracts_api
- **Modified files:** ~15 (Cargo.toml workspace, native-tauri lib.rs, routes/mod.rs, usecases/lib.rs, frontend pages)
- **New frontend:** Agent chat page, Settings page (API key input)

---

*Research: 2026-04-02*
*Phase: 04-minimal-feature-implementation*
