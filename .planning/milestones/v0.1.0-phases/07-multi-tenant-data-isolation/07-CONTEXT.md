# Phase 7: Multi-Tenant Data Isolation - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

所有 SurrealDB 数据访问自动按 tenant_id 隔离，防止跨租户数据泄漏。仅服务端 SurrealDB 需要隔离；本地 libsql 是 per-device 存储，无需 tenant_id。

**不包含：** libsql tenant 隔离（per-device，单用户）、复杂 RBAC（v2）、云端 session 管理（v2）。

</domain>

<decisions>
## Implementation Decisions

### Database Scope
- **D-01:** 仅 SurrealDB（服务端）需要 tenant_id 隔离。libsql 保持原样（per-device 本地存储，无需隔离）
- **D-02:** 所有 SurrealDB 表定义中包含 tenant_id 字段

### Tenant ID Propagation
- **D-03:** Axum 中间件从 `Authorization: Bearer <id_token>` 解码 JWT（使用已有的 `jsonwebtoken` crate），提取 `sub` 作为 tenant_id，注入 Axum request extensions（`TenantId(String)`）
- **D-04:** 不引入新依赖。复用 workspace 已声明的 `jsonwebtoken = "10.3.0"`
- **D-05:** 前端通过 Tauri IPC 调用 Axum API 时，在 HTTP 请求头中携带 id_token（从 tauri-plugin-store 读取）

### User-Tenant Binding
- **D-06:** Hybrid 模式 — 首次 Google 登录自动创建 tenant（1 user = 1 tenant），同时 schema 支持 invite 模式
- **D-07:** SurrealDB 表结构：
  - `tenant` 表：id, name, created_at
  - `user_tenant` 表：user_sub, tenant_id, role (owner/member), joined_at
- **D-08:** 首次登录自动分配 owner 角色。Invite 流程仅定义 schema，不实现端点

### Schema Strategy (Port Trait Wrapper)
- **D-09:** SurrealDbPort 实现增加 `tenant_id: Option<String>` 字段
  - `Some(tenant_id)`：所有查询自动注入 `WHERE tenant_id = $tenant_id`
  - `None`：无隔离（用于 admin/迁移操作）
- **D-10:** INSERT 操作自动在 record 中添加 tenant_id 字段
- **D-11:** 原始 `SurrealDbPort` trait 不变，隔离逻辑在实现层

### Cross-Tenant Isolation Behavior
- **D-12:** 静默过滤 — 查询自动添加 `WHERE tenant_id = $tenant_id`，跨租户查询返回空结果（不是错误）
- **D-13:** UPDATE/DELETE 同样受限于当前 tenant_id，防止修改/删除其他租户的数据

### Agent's Discretion
- JWT 解析失败的具体处理方式（过期 token、格式错误等）
- Tenant name 的默认值策略（从 Google profile name 取，还是用 email 前缀）
- SurrealDB 中 tenant_id 字段的索引策略
- 具体的 SurrealQL 模板格式（scope clause 注入方式）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 7 — Phase goal (data access scoped by tenant_id), success criteria (4 items), depends on Phase 6
- `.planning/REQUIREMENTS.md` §TENANT-01 through §TENANT-03 — tenant_id on all tables, query middleware auto-scopes, user-tenant binding on signup

### Architecture & Patterns
- `.planning/research/ARCHITECTURE.md` §System Overview — Clean Architecture layers, Tauri IPC boundary, runtime_server (Axum) vs runtime_tauri
- `.planning/research/ARCHITECTURE.md` §IPC Recommendation — Tauri IPC as primary channel, Axum for external access

### Prior Phase Context
- `.planning/phases/05-database-infrastructure/05-CONTEXT.md` — AppState (SurrealDB + Moka + reqwest), Domain Port traits (SurrealDbPort, LibSqlPort), tauri-plugin-libsql
- `.planning/phases/06-google-oauth-authentication/06-CONTEXT.md` — OAuth flow (PKCE + deep link), UserProfile { email, name, picture, sub }, session in tauri-plugin-store

### Existing Code
- `crates/domain/src/ports/surreal_db.rs` — SurrealDbPort trait definition (health_check, query<T>)
- `crates/domain/src/ports/lib_sql.rs` — LibSqlPort trait definition (health_check, execute, query<T>)
- `crates/runtime_server/src/state.rs` — AppState { db: Surreal<Any>, cache, http_client }
- `crates/runtime_server/src/lib.rs` — create_router() with middleware layers (Cors, Trace, Timeout)
- `crates/runtime_server/src/routes/mod.rs` — Route barrel, Router<AppState>
- `apps/desktop-ui/src-tauri/src/commands/auth.rs` — AuthSession, UserProfile, Google OAuth flow

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **jsonwebtoken crate** — workspace deps 已声明（10.3.0），Phase 6 用于解码 id_token，可直接用于 Axum 中间件
- **AppState** — 已有 db/cache/http_client，可扩展为携带 tenant_id 或从 request extensions 注入
- **SurrealDbPort trait** — query<T> 方法接受 BTreeMap<String, Value> vars，可在实现层自动注入 tenant_id
- **UserProfile.sub** — Google OAuth sub 字段作为 tenant 标识符（全局唯一、稳定）
- **tauri-plugin-store** — 存储 access_token/id_token，前端可读取并附加到 Axum API 请求头

### Established Patterns
- Domain Port traits — 实现层（runtime_server）负责具体逻辑，trait 保持抽象
- workspace.dependencies 统一管理 — 不直接声明 crate 版本
- Axum middleware layers — 在 create_router() 中注册，已有 Cors/Trace/Timeout
- AppState 注入 — Router::with_state() 使所有 handler 可通过 State<AppState> 访问

### Integration Points
- `crates/runtime_server/src/lib.rs` — create_router() 需要添加 tenant extraction middleware
- `crates/runtime_server/src/state.rs` — AppState 可能需要调整（或 middleware 注入 TenantId 到 extensions）
- `crates/domain/src/ports/surreal_db.rs` — trait 不变，但实现需要 tenant_id: Option<String>
- `crates/runtime_server/src/routes/mod.rs` — 新 API 路由注册点

</code_context>

<specifics>
## Specific Ideas

- "schema-ready, invite stubbed" — tenant 和 user_tenant 表结构定义完整，auto-create on first login 工作，invite 逻辑仅保留占位
- JWT 解码用 Google 的 JWKS endpoint 缓存公钥（或直接用 RS256 从 id_token 解码 payload，签名验证 v2）
- Tenant name 默认从 Google profile name 取，fallback 到 email 前缀

</specifics>

<deferred>
## Deferred Ideas

- libsql tenant 隔离 — 如果未来支持多用户共享设备，可在 libsql 加 tenant_id（v2）
- 完整 invite 流程 — create invite token, accept invite, admin role management（v2）
- RBAC（角色权限系统）— user_tenant.role 可扩展为细粒度权限（v2）
- 云端 session 管理 — 由 Auth Server 管理跨平台 session（v2）
- Server-side token 验证 — Axum 中间件调 Google tokeninfo endpoint 验证 token 有效性（v2）

</deferred>

---

*Phase: 07-multi-tenant-data-isolation*
*Context gathered: 2026-03-29*
