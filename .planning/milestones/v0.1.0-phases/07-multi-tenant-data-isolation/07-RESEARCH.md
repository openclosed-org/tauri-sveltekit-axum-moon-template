# Phase 7: Multi-Tenant Data Isolation - Research

**Researched:** 2026-03-29
**Domain:** SurrealDB tenant isolation, Axum middleware, JWT extraction
**Confidence:** HIGH

## Summary

Phase 7 实现多租户数据隔离：所有 SurrealDB 查询自动按 `tenant_id` 过滤，防止跨租户数据泄漏。核心机制是三层：

1. **Axum 中间件** — 从 `Authorization: Bearer <id_token>` 解码 JWT payload（不做签名验证，v2），提取 `sub` 作为 tenant_id，注入 `TenantId` request extension
2. **SurrealDbPort 实现包装** — 在实现层持有 `tenant_id: Option<String>`，所有 query 自动追加 `WHERE tenant_id = $tenant_id`，INSERT 自动添加 tenant_id 字段
3. **Tenant 表** — `tenant` + `user_tenant` 表定义，首次登录自动创建 tenant 并绑定 owner 角色

**关键设计决策：** 不修改 SurrealDbPort trait（D-11），所有隔离逻辑在实现层。使用 `jsonwebtoken` 的 `dangerous::insecure_decode` 做 payload-only 解码（不验证签名），避免引入 JWKS 缓存复杂度（v2 再做）。

**Primary recommendation:** 实现一个 `TenantAwareSurrealDb` 包装 struct，持有 `Surreal<Any>` + `Option<String>`（tenant_id），trait 方法自动注入 tenant_id，保持 SurrealDbPort trait 不变。

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** 仅 SurrealDB（服务端）需要 tenant_id 隔离。libsql 保持原样
- **D-02:** 所有 SurrealDB 表定义中包含 tenant_id 字段
- **D-03:** Axum 中间件从 `Authorization: Bearer <id_token>` 解码 JWT，提取 `sub` 作为 tenant_id，注入 Axum request extensions（`TenantId(String)`）
- **D-04:** 不引入新依赖。复用 workspace 已声明的 `jsonwebtoken = "10.3.0"`
- **D-05:** 前端通过 Tauri IPC 调用 Axum API 时，在 HTTP 请求头中携带 id_token（从 tauri-plugin-store 读取）
- **D-06:** Hybrid 模式 — 首次 Google 登录自动创建 tenant（1 user = 1 tenant），同时 schema 支持 invite 模式
- **D-07:** SurrealDB 表结构：`tenant` 表（id, name, created_at），`user_tenant` 表（user_sub, tenant_id, role, joined_at）
- **D-08:** 首次登录自动分配 owner 角色。Invite 流程仅定义 schema，不实现端点
- **D-09:** SurrealDbPort 实现增加 `tenant_id: Option<String>` 字段，Some(tenant_id) 时自动注入 WHERE 子句
- **D-10:** INSERT 操作自动在 record 中添加 tenant_id 字段
- **D-11:** 原始 SurrealDbPort trait 不变，隔离逻辑在实现层
- **D-12:** 静默过滤 — 查询自动添加 WHERE，跨租户查询返回空结果（不是错误）
- **D-13:** UPDATE/DELETE 同样受限于当前 tenant_id

### the agent's Discretion
- JWT 解析失败的具体处理方式（过期 token、格式错误等）
- Tenant name 的默认值策略（从 Google profile name 取，还是用 email 前缀）
- SurrealDB 中 tenant_id 字段的索引策略
- 具体的 SurrealQL 模板格式（scope clause 注入方式）

### Deferred Ideas (OUT OF SCOPE)
- libsql tenant 隔离 — 如果未来支持多用户共享设备，可在 libsql 加 tenant_id（v2）
- 完整 invite 流程 — create invite token, accept invite, admin role management（v2）
- RBAC（角色权限系统）— user_tenant.role 可扩展为细粒度权限（v2）
- 云端 session 管理 — 由 Auth Server 管理跨平台 session（v2）
- Server-side token 验证 — Axum 中间件调 Google tokeninfo endpoint 验证 token 有效性（v2）

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| jsonwebtoken | 10.3.0 | JWT decode (payload-only) | Workspace 已声明，Phase 6 使用 |
| surrealdb | 3.0.5 | Server-side database | Workspace 已声明，Phase 5 完成 |
| axum | 0.8.8 | HTTP middleware | Workspace 已声明，已有 Cors/Trace/Timeout |
| serde | 1.0.228 | Serialization | Workspace 已声明 |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 | Datetime for created_at/joined_at | Workspace 已声明 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `dangerous::insecure_decode` | Full JWKS verification | v1 deferred — 后续加 JWKS 缓存 |
| Middleware state injection | SurrealDB SCOPE/PERMISSIONS | SurrealDB 内置权限 — 更复杂，不适合 boilerplate |

**安装：** 无需新依赖 — jsonwebtoken、surrealdb、axum、serde、chrono 均已在 workspace 声明。

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TENANT-01 | Database schema includes tenant_id on all tables | SurrealDB DEFINE TABLE + DEFINE FIELD syntax (HIGH) |
| TENANT-02 | Query middleware automatically scopes by tenant_id | Axum middleware::from_fn + SurrealDbPort wrapper pattern (HIGH) |
| TENANT-03 | User belongs to exactly one tenant on signup | tenant + user_tenant table + auto-create logic (HIGH) |

## Architecture Patterns

### Recommended Project Structure

```
crates/
├── domain/
│   └── src/
│       └── ports/
│           ├── surreal_db.rs       # trait 不变
│           └── mod.rs              # 新增 TenantId newtype
├── runtime_server/
│   └── src/
│       ├── lib.rs                  # 添加 tenant middleware layer
│       ├── middleware/
│       │   └── tenant.rs           # JWT 解码 + TenantId 注入
│       ├── ports/
│       │   └── surreal_db.rs       # TenantAwareSurrealDb 实现
│       ├── routes/
│       │   └── tenant.rs           # tenant init API（首次登录）
│       └── state.rs                # 不变
```

### Pattern 1: Axum Tenant Extraction Middleware

**What:** 从 `Authorization: Bearer <id_token>` header 提取 JWT，用 `jsonwebtoken::dangerous::insecure_decode` 解码 payload，取 `sub` 作为 tenant_id，插入 request extensions。

**When to use:** 所有需要 tenant 隔离的 API 路由。

**Example:**
```rust
// Source: Context7 - tokio-rs/axum middleware from_fn pattern
// Source: Context7 - keats/jsonwebtoken dangerous::insecure_decode

use axum::{middleware::Next, extract::Request, http::StatusCode, response::Response};
use jsonwebtoken::dangerous::insecure_decode;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TenantId(pub String);

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
    // exp, email, name 等字段 — 仅需 sub
}

pub async fn tenant_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract Bearer token
    let token = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Decode JWT payload (no signature verification — v1)
    let token_data = insecure_decode::<IdTokenClaims>(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 3. Inject tenant_id into request extensions
    req.extensions_mut().insert(TenantId(token_data.claims.sub));

    Ok(next.run(req).await)
}
```

**注册方式：**
```rust
// crates/runtime_server/src/lib.rs
use axum::middleware;

pub fn create_router(state: AppState) -> Router {
    let api_routes = routes::router()
        .route_layer(middleware::from_fn(tenant_middleware));

    Router::new()
        .merge(health::router())  // health 不需要 tenant
        .merge(api_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(...))
}
```

### Pattern 2: TenantAwareSurrealDb Wrapper

**What:** 实现 SurrealDbPort trait，持有 `Surreal<Any>` + `Option<String>`（tenant_id）。query 方法自动在 SQL 后追加 `WHERE tenant_id = $tenant_id` 并绑定参数。

**When to use:** 所有带 tenant_id 的 SurrealDB 操作。

**Example:**
```rust
// SurrealQL 自动注入 tenant_id 示例
// 原始查询: "SELECT * FROM counter WHERE user = $user"
// 注入后:   "SELECT * FROM counter WHERE user = $user AND tenant_id = $tenant_id"

impl SurrealDbPort for TenantAwareSurrealDb {
    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        mut vars: BTreeMap<String, Value>,
    ) -> Result<Vec<T>, SurrealError> {
        // 1. 注入 tenant_id 到 vars
        if let Some(ref tid) = self.tenant_id {
            vars.insert("tenant_id".into(), surrealdb::sql::Value::from(tid.as_str()));
        }

        // 2. 重写 SQL — 在第一个 WHERE 子句后追加 AND tenant_id = $tenant_id
        //    或如果没有 WHERE，则添加 WHERE tenant_id = $tenant_id
        let scoped_sql = if let Some(ref tid) = self.tenant_id {
            Self::inject_tenant_filter(sql)
        } else {
            sql.to_string()
        };

        // 3. 执行查询
        let mut response = self.db.query(&scoped_sql).bind(vars).await?;
        // ...
    }
}
```

**SQL 注入策略（三种操作）：**

| 操作 | 注入方式 | 示例 |
|------|----------|------|
| SELECT | 追加 `AND tenant_id = $tenant_id` | `SELECT * FROM t WHERE x=1 AND tenant_id = $tenant_id` |
| INSERT | 在 content 中添加 tenant_id 字段 | `CREATE t SET x=1, tenant_id = $tenant_id` |
| UPDATE/DELETE | 追加 `WHERE tenant_id = $tenant_id` | `UPDATE t SET x=2 WHERE tenant_id = $tenant_id` |

### Pattern 3: Tenant Schema Definition

**What:** SurrealDB 表结构定义 — `tenant` 和 `user_tenant` 表，支持 1:1 自动创建 + 未来 invite 扩展。

**Example:**
```rust
// 启动时执行的 schema migration
db.query("
    DEFINE TABLE tenant SCHEMAFULL;
    DEFINE FIELD name ON TABLE tenant TYPE string;
    DEFINE FIELD created_at ON TABLE tenant TYPE datetime DEFAULT time::now();

    DEFINE TABLE user_tenant SCHEMAFULL;
    DEFINE FIELD user_sub ON TABLE user_tenant TYPE string;
    DEFINE FIELD tenant_id ON TABLE user_tenant TYPE record<tenant>;
    DEFINE FIELD role ON TABLE user_tenant TYPE string DEFAULT 'member';
    DEFINE FIELD joined_at ON TABLE user_tenant TYPE datetime DEFAULT time::now();

    -- 唯一索引：每个 user_sub 只能绑定一个 tenant
    DEFINE INDEX user_sub_unique ON TABLE user_tenant COLUMNS user_sub UNIQUE;

    -- 索引：tenant_id 查询优化
    DEFINE INDEX tenant_idx ON TABLE user_tenant COLUMNS tenant_id;
").await?.check()?;
```

### Anti-Patterns to Avoid
- **修改 SurrealDbPort trait 加 tenant_id 参数：** 违反 D-11，trait 应保持抽象
- **在 SurrealDB 内置 SCOPE/PERMISSIONS 做隔离：** 太复杂，不适合 boilerplate，且绕过 Rust 层会丢失类型安全
- **每次请求都查 DB 获取 tenant_id：** JWT sub 本身就是稳定的 tenant 标识符，无需额外查询
- **用 HTTP header 直传 tenant_id（不验证）：** 安全隐患，必须从 JWT 提取

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JWT 解码 | 手写 base64 解码 | `jsonwebtoken::dangerous::insecure_decode` | 处理 header/payload/格式校验，Phase 6 已经手动解码过一次 |
| JWT 签名验证 | 自建 JWKS 缓存 | v2 再做 — 当前用 insecure_decode | JWKS 缓存、轮换、并发控制是独立工程量 |
| SQL 注入防护 | 手拼 WHERE 字符串 | SurrealDB 参数绑定 `.bind()` | 参数化查询防止注入 |
| 时间戳 | 自算 Unix time | `chrono::Utc::now()` | 已在 workspace 声明 |

**Key insight:** `dangerous::insecure_decode` 是 v1 正确选择 — 它做格式校验但不做签名验证，正好匹配 "v1 不做 server-side token verification" 的决策。

## Common Pitfalls

### Pitfall 1: SQL 重写破坏原有 WHERE 子句
**What goes wrong:** 追加 `AND tenant_id = $tenant_id` 时，如果原 SQL 没有 WHERE（如 `SELECT * FROM table`），会生成 `SELECT * FROM table AND tenant_id = ...`（语法错误）。
**Why it happens:** 未区分 "有 WHERE" 和 "无 WHERE" 两种情况。
**How to avoid:** 检测 SQL 是否包含 `WHERE` 关键字，无则用 `WHERE tenant_id = $tenant_id`，有则用 `AND tenant_id = $tenant_id`。
**Warning signs:** 查询报 SurrealDB syntax error。

### Pitfall 2: INSERT/CREATE 未注入 tenant_id
**What goes wrong:** 只处理了 SELECT 的 tenant_id 注入，CREATE/INSERT 忘记，导致新记录没有 tenant_id。
**Why it happens:** SQL 重写只处理 SELECT 分支。
**How to avoid:** 对 CREATE 语句同样重写，在 SET 子句中追加 `tenant_id = $tenant_id`。
**Warning signs:** 新记录插入成功但后续查询找不到（因为 WHERE tenant_id 过滤掉了 NULL）。

### Pitfall 3: 中间件顺序错误
**What goes wrong:** tenant 中间件注册在 CORS/Trace 之前，导致 OPTIONS 预检请求也被拦截。
**Why it happens:** Axum 中间件是 LIFO（最后注册最先执行）。
**How to avoid:** 用 `route_layer` 而非 `layer` — route_layer 只对匹配的路由生效，且在 CORS/Trace 内层。
**Warning signs:** 浏览器 CORS preflight 请求返回 401。

### Pitfall 4: `dangerous::insecure_decode` 接受过期 token
**What goes wrong:** insecure_decode 不验证 exp claim，过期 token 仍可使用。
**Why it happens:** 这是 insecure_decode 的设计 — 它不做任何验证。
**How to avoid:** v1 接受此限制（与 Phase 6 一致），v2 做完整验证。但可以在中间件中手动检查 exp 字段（可选）。
**Warning signs:** 过期 token 仍能访问 API。

### Pitfall 5: tenant_id 索引缺失导致全表扫描
**What goes wrong:** 大量记录时，`WHERE tenant_id = $tenant_id` 无索引导致慢查询。
**Why it happens:** 未在 schema 定义中添加 tenant_id 索引。
**How to avoid:** 在 DEFINE SCHEMA 时对所有业务表的 tenant_id 字段添加索引。
**Warning signs:** 查询延迟随数据量线性增长。

## Code Examples

### 完整中间件注册（create_router）
```rust
// Source: Context7 - tokio-rs/axum middleware from_fn pattern
use axum::middleware;

pub fn create_router(state: AppState) -> Router {
    // Tenant-scoped routes — middleware applies tenant_id extraction
    let tenant_routes = routes::router()
        .route_layer(middleware::from_fn(tenant::tenant_middleware));

    // Public routes — health checks, no auth
    Router::new()
        .merge(routes::health::router())
        .merge(tenant_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
}
```

### SurrealDB 参数化查询
```rust
// Source: Context7 - surrealdb/surrealdb query with parameter binding
use surrealdb::sql::Value;

// SELECT with tenant_id injection
let mut response = db
    .query("SELECT * FROM counter WHERE tenant_id = $tenant_id AND user = $user")
    .bind(("tenant_id", "google-sub-123"))
    .bind(("user", "alice"))
    .await?;

// CREATE with tenant_id
let mut response = db
    .query("CREATE counter SET name = $name, count = 0, tenant_id = $tenant_id")
    .bind(("name", "my-counter"))
    .bind(("tenant_id", "google-sub-123"))
    .await?;

// UPDATE scoped by tenant_id
let mut response = db
    .query("UPDATE counter SET count += 1 WHERE id = $id AND tenant_id = $tenant_id")
    .bind(("id", thing("counter:abc")))
    .bind(("tenant_id", "google-sub-123"))
    .await?;
```

### 首次登录自动创建 Tenant
```rust
// Source: Context7 - surrealdb/surrealdb CREATE query
// 首次登录时：
// 1. 查 user_tenant WHERE user_sub = $sub
// 2. 如果不存在 → CREATE tenant, CREATE user_tenant (role: 'owner')
// 3. 返回 tenant_id

async fn ensure_tenant(db: &Surreal<Any>, user_sub: &str, user_name: &str) -> Result<String, Error> {
    // 1. Check existing binding
    let existing: Vec<UserTenant> = db
        .query("SELECT * FROM user_tenant WHERE user_sub = $sub")
        .bind(("sub", user_sub))
        .await?
        .take(0)?;

    if let Some(ut) = existing.first() {
        return Ok(ut.tenant_id.to_string());
    }

    // 2. Create tenant
    let tenant: Option<Tenant> = db
        .create("tenant")
        .content(Tenant { name: user_name.to_string() })
        .await?;

    let tenant_id = tenant.unwrap().id.to_string();

    // 3. Create user_tenant binding
    db.query("CREATE user_tenant SET user_sub = $sub, tenant_id = $tid, role = 'owner'")
        .bind(("sub", user_sub))
        .bind(("tid", thing(&tenant_id)))
        .await?;

    Ok(tenant_id)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 手动 base64 解码 JWT payload | `jsonwebtoken::dangerous::insecure_decode` | Phase 6 → Phase 7 | 更可靠，处理格式校验 |
| 无 tenant 隔离 | Middleware + Port wrapper 注入 | Phase 7 | 防止跨租户泄漏 |
| Redis 用于 session 缓存 | Moka in-memory | Phase 5 | 简化部署 |

**Deprecated/outdated:**
- Phase 6 的手动 JWT 解码（`URL_SAFE_NO_PAD.decode(id_token_parts[1])`）→ 用 `insecure_decode<Claims>` 替代
- 注意：Phase 6 在 Tauri 侧的解码可以保留（它是提取 profile 用的），Axum 侧使用标准 crate

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| jsonwebtoken | JWT decode | ✓ | 10.3.0 (workspace) | — |
| surrealdb | DB queries | ✓ | 3.0.5 (workspace) | — |
| axum | HTTP middleware | ✓ | 0.8.8 (workspace) | — |
| chrono | Datetime fields | ✓ | 0.4 (workspace) | — |

**无缺失依赖。** 所有需要的 crate 均已在 workspace Cargo.toml 声明。

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test + rstest |
| Config file | none — crate-level tests |
| Quick run command | `cargo test -p runtime_server` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TENANT-01 | Schema includes tenant_id on all tables | integration | `cargo test -p runtime_server -- tenant_schema` | ❌ Wave 0 |
| TENANT-02 | Query middleware auto-scopes by tenant_id | unit | `cargo test -p runtime_server -- tenant_middleware` | ❌ Wave 0 |
| TENANT-02 | TenantAwareSurrealDb auto-injects tenant_id | unit | `cargo test -p runtime_server -- tenant_aware_db` | ❌ Wave 0 |
| TENANT-03 | User belongs to exactly one tenant on signup | integration | `cargo test -p runtime_server -- ensure_tenant` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p runtime_server`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** `cargo test --workspace` green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/runtime_server/src/middleware/tenant.rs` — tenant extraction middleware + unit tests
- [ ] `crates/runtime_server/src/ports/surreal_db.rs` — TenantAwareSurrealDb 实现 + 注入测试
- [ ] `crates/runtime_server/src/routes/tenant.rs` — ensure_tenant API + integration test
- [ ] `crates/domain/src/ports/mod.rs` — TenantId newtype 定义

## Open Questions

1. **SQL 重写 vs raw query — 哪种注入策略更稳健？**
   - What we know: SurrealDB 的 `db.query()` 支持参数绑定，但不支持 "追加 WHERE 到已有查询"
   - What's unclear: SQL 字符串拼接是否安全（SurrealQL 和 SQL 不同）
   - Recommendation: **SQL 字符串重写** — 在 SELECT/UPDATE/DELETE 的已有 WHERE 后追加 `AND tenant_id = $tenant_id`，CREATE 的 SET 子句追加 `tenant_id = $tenant_id`。用正则或字符串检测 WHERE 关键字位置。风险可控因为：(a) 原始 SQL 来自 Rust 代码（非用户输入），(b) tenant_id 是参数绑定的

2. **健康检查路由是否需要 tenant 隔离？**
   - What we know: `/healthz` 和 `/readyz` 是公开端点
   - What's unclear: 无
   - Recommendation: 健康检查不走 tenant middleware — 已通过 `route_layer` 路由分离解决

3. **`dangerous::insecure_decode` 对畸形 JWT 的错误类型？**
   - What we know: 返回 `jsonwebtoken::errors::Error`，包含 Malformed、InvalidSignature 等变体
   - What's unclear: insecure_decode 是否也会因为签名无效而失败（理论不应该）
   - Recommendation: 中间件中统一 map 到 `StatusCode::UNAUTHORIZED`，不区分错误类型（v1 简化）

## Sources

### Primary (HIGH confidence)
- Context7 `/keats/jsonwebtoken` — `dangerous::insecure_decode`, `decode_header`, JWKS parsing
- Context7 `/surrealdb/surrealdb` — `query().bind()`, `DEFINE TABLE/FIELD`, `CREATE/DELETE` Rust SDK
- Context7 `/tokio-rs/axum` — `middleware::from_fn`, `request.extensions_mut().insert()`, `Extension` extractor

### Secondary (MEDIUM confidence)
- Phase 6 auth.rs — 现有 JWT 解码模式（手动 base64），可作为对照
- workspace Cargo.toml — jsonwebtoken 10.3.0, surrealdb 3.0.5 已声明

### Tertiary (LOW confidence)
- 无 — 所有关键信息均来自 Context7 官方文档

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — 所有依赖已在 workspace 声明，无需新 crate
- Architecture: HIGH — Axum middleware + Port wrapper 是标准模式，Context7 验证
- Pitfalls: HIGH — SQL 重写风险已识别并有缓解方案

**Research date:** 2026-03-29
**Valid until:** 2026-04-29 (30 days — stable technology)
