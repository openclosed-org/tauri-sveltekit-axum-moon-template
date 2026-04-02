# Phase 05: Database & Infrastructure - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

数据库层基础设施就绪：双数据库连接配置（SurrealDB 服务端 + libsql 本地）、DatabaseAdapter 抽象层实现、HTTP 客户端配置、Moka 缓存层、Quinn HTTP/3 传输层。目标是 boilerplate 用户拿到项目即可开始写业务逻辑，不需要自己接数据库。

**不包含：** Docker/nginx、Cloudflare Tunnel、apalis job queue、R2 对象存储、Pingora — 这些属于部署/运维/独立功能阶段。

</domain>

<decisions>
## Implementation Decisions

### Database Architecture (覆盖 STATE.md 之前的 "libsql-only" 决定)
- **D-01:** 双数据库架构 — SurrealDB (服务端 Axum 连接) + libsql (本地 App 嵌入式存储)
- **D-02:** Turso 云端同步可选配置 — boilerplate 中预置配置但默认关闭，用户按需启用
- **D-03:** libsql 本地存储通过 `tauri-plugin-libsql` 实现（已在 Cargo.toml 声明，需注册到 Tauri builder）
- **D-04:** SurrealDB 独立部署，Axum 通过连接池访问

### DatabaseAdapter Pattern
- **D-05:** Trait-per-DB 模式 (Repository pattern) — 每个数据库独立 Port trait
  - `domain` crate 定义 trait：`SurrealDbPort`, `LibSqlPort`
  - `application` crate 通过 trait 依赖，不依赖具体实现
  - 具体 adapter 在 `runtime_server` / `runtime_tauri` 中实现并注入
- **D-06:** 不使用 enum-based unified adapter — Rust trait + DI 更 idiomatic，零成本抽象

### HTTP Transport
- **D-07:** HTTP/3 (Quinn QUIC) 作为主传输层
- **D-08:** HTTP/2 (Axum TCP) 作为 fallback 层
- **D-09:** 开发环境默认 TCP (:3001) 调试友好，生产可启用 h3

### Cache Layer
- **D-10:** Moka 替代 Redis — Rust-native 内存缓存，零外部依赖
- **D-11:** Moka 作为 Axum shared state 层注入，用于 session/查询缓存
- **D-12:** 不引入 Redis 依赖 — boilerplate 保持纯 Rust 栈

### HTTP Client
- **D-13:** reqwest 0.13.2 (rustls) 已在 workspace 声明，配置为 Axum 共享状态
- **D-14:** 统一的 HTTP 客户端实例（连接池复用），通过 DI 注入到需要外部 API 调用的服务

### Deferred Components
- **D-15:** 不使用 Docker/nginx — 纯 Rust 部署方案 (systemd + Cloudflare Tunnel，后续阶段)
- **D-16:** redis, rathole, vector — 从 Cargo.toml 注释预加载中移除或标记为 v2

### Agent's Discretion
- SurrealDB 连接池配置（连接数、超时）
- Moka cache 具体 eviction 策略和容量
- Quinn HTTP/3 证书配置（自签 vs ACME）
- libsql migration 文件组织和 rusqlite_migration 集成细节
- reqwest 客户端 middleware 配置（重试、超时）
- Turso 同步触发策略（手动/定时/事件驱动）

### Folded Todos
None.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 5 — Phase goal, success criteria, core dependencies list
- `.planning/REQUIREMENTS.md` §INFRA-01 through §INFRA-04 — Acceptance criteria for DB, HTTP client, tunnel, proxy

### Architecture & Design
- `.planning/research/ARCHITECTURE.md` §Database Layer Architecture (lines 508-607) — libsql layer structure, Repository trait pattern, multi-tenant schema, migration strategy
- `docs/toDev/TECH_SELECTION.md` §二 — 双数据库架构描述, DatabaseAdapter trait sketch, 依赖版本

### Prior Phase Context
- `.planning/phases/04-backend-dependencies-build-optimization/04-CONTEXT.md` — Axum 中间件栈, runtime_server 结构, Cargo.toml 预加载依赖列表

### Workspace Configuration
- `Cargo.toml` — Root workspace dependencies (surrealdb 3.0.5, reqwest 0.13.2, tauri-plugin-libsql 0.1.0 active; libsql, redis, rathole, vector commented)
- `crates/runtime_server/Cargo.toml` — Axum/Tower stack, 需要新增 DB + cache + h3 依赖
- `apps/desktop-ui/src-tauri/Cargo.toml` — tauri-plugin-libsql 已声明 workspace=true

### Existing Code
- `crates/runtime_server/src/lib.rs` — Axum router with middleware, no DB state yet
- `crates/runtime_server/src/routes/health.rs` — /readyz has Phase 5 TODO for DB health check
- `crates/domain/src/lib.rs` — Empty placeholder, awaiting Port trait definitions
- `crates/application/src/lib.rs` — Empty placeholder, awaiting use case orchestration
- `apps/desktop-ui/src-tauri/src/lib.rs` — Tauri builder registers 3 plugins, libsql missing

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **workspace.dependencies** (`Cargo.toml`): surrealdb 3.0.5, reqwest 0.13.2, tauri-plugin-libsql 0.1.0 已声明 — runtime_server 直接 workspace=true 引用
- **Axum server skeleton** (`crates/runtime_server/`): create_router() with CorsLayer, TraceLayer, TimeoutLayer 已就绪 — 加 with_state() 注入 DB 连接池
- **Clean Architecture crates**: domain, application, shared_contracts 各有空 lib.rs — Phase 5 定义 Port trait 和 DTO 类型
- **/readyz placeholder** (`routes/health.rs`): 已有 TODO Phase 5 — 直接接入 DB 健康检查

### Established Patterns
- workspace.dependencies 统一管理版本 — 新增依赖必须走此模式
- 所有 crate 用 edition = "2021"
- crate 内 path 引用（crates 之间）+ workspace 引用（公共依赖）
- Axum Router 模块化：每个功能模块导出 router()，顶层 create_router() 合并

### Integration Points
- `crates/runtime_server/Cargo.toml` — 需要新增 surrealdb, libsql, moka, quinn, reqwest workspace deps
- `crates/runtime_server/src/lib.rs` — 需要 with_state() 注入 DB 连接池 + Moka cache + reqwest client
- `crates/runtime_server/src/routes/health.rs` — /readyz 接入 DB pool health check
- `crates/domain/src/lib.rs` — 定义 SurrealDbPort, LibSqlPort traits
- `apps/desktop-ui/src-tauri/src/lib.rs` — 注册 tauri_plugin_libsql，初始化本地 DB

</code_context>

<specifics>
## Specific Ideas

- 纯 Rust 栈：零外部进程依赖（不装 Redis、不装 nginx、不装 Docker），cargo build 即可运行
- HTTP/3 primary + HTTP/2 fallback — 开发用 TCP 调试，生产启用 QUIC
- Moka 替代 Redis — 编译时确定，运行时零配置
- SurrealDB + libsql 都保留在 boilerplate 中 — 最终取舍由 boilerplate 用户决定
- Schema 设计遵循 ARCHITECTURE.md 中的 multi-tenant pattern (tenant_id on all tables)

</specifics>

<deferred>
## Deferred Ideas

- **Cloudflare Tunnel** — 部署/运维层，独立阶段
- **apalis job queue** — 后台任务处理，独立阶段
- **R2 object storage** — 对象存储，独立阶段
- **Pingora** — 反向代理，部署层，独立阶段
- **nginx** — 不使用，被 Pingora/Tunnel 方案替代
- **redis** — 不使用，被 Moka 替代
- **rathole/FerroTunnel** — 不使用，被 Cloudflare Tunnel 替代
- **vector observability** — 推迟到 v2 或按需启用

### Reviewed Todos (not folded)
None.

</deferred>

---

*Phase: 05-database-infrastructure*
*Context gathered: 2026-03-29*
