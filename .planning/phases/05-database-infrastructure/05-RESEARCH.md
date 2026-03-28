# Phase 5 Research: Database & Infrastructure (双数据库架构)

**Researched:** 2026-03-29
**Confidence:** HIGH

## Research Questions
1. SurrealDB 3.x Rust client initialization and Axum integration
2. tauri-plugin-libsql registration and local DB setup
3. DatabaseAdapter trait pattern — trait-per-DB vs enum-based
4. Moka cache integration with Axum shared state
5. Quinn HTTP/3 transport layer setup
6. reqwest shared client configuration

## Findings

### 1. SurrealDB 3.x Server-Side Integration

**Key:** SurrealDB 3.0.5 is already declared in workspace `Cargo.toml`. The `surrealdb` crate provides an in-process engine (no separate server needed for dev).

**Initialization pattern:**
```rust
use surrealdb::{engine::any::Any, Surreal};

let db = Surreal::<Any>::init();
db.connect("memory").await?;  // or "rocksdb://path" for persistence
db.use_ns("app").use_db("main").await?;
```

**Axum integration:** Store `Surreal<Any>` as `Extension` or custom `State`:
```rust
let app = Router::new()
    .route("/api/...", ...)
    .layer(Extension(db.clone()));
```

**Caveat:** SurrealDB has its own query language (SurrealQL), not standard SQL. The `query()` method accepts SurrealQL strings. This is different from libsql's SQLite-compatible SQL — hence the need for separate Port traits, not a unified adapter.

### 2. tauri-plugin-libsql Local DB Setup

**Status:** `tauri-plugin-libsql = "0.1.0"` declared in workspace deps AND `apps/desktop-ui/src-tauri/Cargo.toml`. But NOT registered in Tauri builder (`src-tauri/src/lib.rs`).

**Registration:**
```rust
.plugin(tauri_plugin_libsql::Builder::default().build())
```

**Local DB initialization:** The plugin provides commands for the frontend to interact with a local SQLite database. For Rust-side access, the `libsql` crate is needed directly.

**Note:** `libsql` crate is currently commented out in workspace deps (Phase 5 future). Need to uncomment and add `rusqlite_migration` for embedded migrations.

### 3. DatabaseAdapter Pattern — Trait-per-DB

**CONTEXT.md Decision D-05/D-06:** Use trait-per-DB (not enum-based unified adapter). Each DB gets its own Port trait.

**Rationale:** SurrealDB and libsql have fundamentally different query languages (SurrealQL vs SQLite SQL). A unified `query(sql)` trait would be misleading. Separate traits make the abstraction honest.

**Pattern:**
```rust
// domain crate
#[async_trait]
pub trait SurrealDbPort: Send + Sync {
    async fn health_check(&self) -> Result<()>;
    async fn query<T: DeserializeOwned>(&self, sql: &str, vars: BTreeMap<String, Value>) -> Result<Vec<T>>;
}

#[async_trait]
pub trait LibSqlPort: Send + Sync {
    async fn health_check(&self) -> Result<()>;
    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64>;
    async fn query<T: DeserializeOwned>(&self, sql: &str, params: Vec<String>) -> Result<Vec<T>>;
}
```

**Dependency direction:** `domain` defines traits (zero deps besides serde). `application` uses traits. `runtime_server` / `runtime_tauri` provide implementations.

### 4. Moka Cache (替代 Redis)

**Pure Rust in-memory cache.** No external process needed.

**Axum integration:** Store `moka::future::Cache<K, V>` in shared state:
```rust
use moka::future::Cache;

let cache: Cache<String, String> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300))
    .build();

// In Axum state
#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
    cache: Cache<String, String>,
    http_client: reqwest::Client,
}
```

**Workspace dep needed:** `moka = { version = "0.12", features = ["future"] }`

### 5. Quinn HTTP/3 Transport

**For dev:** TCP (Axum default on :3001) is sufficient — debugging friendly.
**For prod:** h3 (HTTP/3 over QUIC) via `quinn` + `h3` crates.

**Setup complexity:** HIGH. Requires TLS certificates (self-signed for dev, ACME for prod). Recommend deferring the actual h3 listener to a later phase, but add the dependency scaffolding now.

**Workspace deps needed:**
- `quinn = "0.11"` — QUIC transport
- `h3 = "0.0.6"` — HTTP/3 layer
- `rustls-pemfile = "2"` — Certificate loading

**Recommendation:** Add deps to workspace, create a placeholder `h3_server.rs` module in runtime_server, but keep the TCP listener as the primary for Phase 5.

### 6. reqwest Shared Client

**Already in workspace deps:** `reqwest = { version = "0.13.2", features = ["rustls"] }`

**Pattern:** Single `reqwest::Client` instance (connection pool) shared via Axum state:
```rust
let http_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .build()?;
```

No additional deps needed. Just add `reqwest = { workspace = true }` to `runtime_server/Cargo.toml`.

## Validation Architecture

### Health Check Integration
- `/readyz` endpoint already has Phase 5 TODO — wire up DB connection checks
- SurrealDB: `db.health().await.is_ok()`
- libsql: connection ping query (`SELECT 1`)

### Cargo Check Gate
- `cargo check --workspace` must pass after all changes
- `cargo test --workspace` for any testable logic

## Recommendations

1. **Start with domain traits** — define `SurrealDbPort` and `LibSqlPort` before any implementations
2. **Axum AppState** — single struct holding all shared resources (SurrealDB, Moka cache, reqwest client)
3. **Keep h3 scaffolding minimal** — add deps + placeholder, don't implement full h3 listener yet
4. **Skip Docker entirely** — per CONTEXT.md, pure Rust stack
5. **Uncomment `libsql` in workspace deps** — it's already planned, just commented

## Sources
- SurrealDB Rust SDK docs (surrealdb 3.x)
- tauri-plugin-libsql GitHub
- Moka cache docs (moka 0.12)
- h3 + quinn crate docs
- ARCHITECTURE.md §Database Layer Architecture

---

*Research for Phase 5: Database & Infrastructure*
*2026-03-29*
