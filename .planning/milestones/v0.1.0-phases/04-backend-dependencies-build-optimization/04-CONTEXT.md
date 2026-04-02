# Phase 04: Backend Dependencies & Build Optimization - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Rust workspace 的 Axum 后端依赖配置、release profile 优化、以及 Axum 服务器健康检查的基础路由。目标是 15MB 以下的 release binary。这是一次**配置和基础设施**阶段 — 不涉及业务逻辑，不涉及数据库集成，不涉及 OAuth。

**产出:**
- runtime_server crate 引入 workspace deps (axum, tower, tower-http, hyper) 并编译通过
- 一个可启动的 Axum HTTP 服务器，响应 /healthz 和 /readyz
- Release profile 最终调优（平台级 strip 差异，panic=abort）
- Cargo.toml 预加载未来阶段的注释依赖

</domain>

<decisions>
## Implementation Decisions

### Axum 中间件栈
- **D-01:** tower-http 0.6+ 全套中间件：CorsLayer::permissive() + TraceLayer + TimeoutLayer + 自定义 request-id 中间件
- **D-02:** 模块化路由组织 — 每个功能模块目录导出 `router()` 函数，顶层 `create_router()` 合并
  - 目录结构：`crates/runtime_server/src/routes/health.rs`, `api.rs`, `mod.rs`
  - 各模块内用 `axum::Router::new()` 定义子路由
- **D-03:** 分层健康检查：GET /healthz（存活检查，返回 `{"status":"ok"}`）+ GET /readyz（就绪检查，预留数据库连接状态等）

### Binary 优化
- **D-04:** Profile 分级 panic 策略：`[profile.release] panic = "abort"`, `[profile.dev] panic = "unwind"`
- **D-05:** moon task 集成 `cargo bloat --release --crates` 做二进制大小监控，定期检查依赖膨胀
- **D-06:** 平台级 strip 差异：
  - Windows: strip debug symbols, 保留 symbol table（调试需要）
  - macOS: strip all
  - Linux: strip all

### 依赖管理
- **D-07:** workspace.dependencies 新增：tower 0.5, tower-http 0.6, hyper 1.x
- **D-08:** 版本策略：workspace.dependencies 精确版本锁定，crate 内用 `workspace = true` 引用
- **D-09:** axum 启用 json 特性，额外引入 axum-extra（cookie/query 支持）

### 基础设施预加载
- **D-10:** Cargo.toml 中用 `#` 注释块预加载未来阶段依赖，格式：`# crate_name = "version"  # Phase X: 用途`
- **D-11:** 预加载范围覆盖 Phase 5-8 全部依赖：
  - Phase 5 (DB/Infra): surrealdb, libsql, reqwest (已声明), redis 驱动
  - Phase 5 (Tunnel): rathole / FerroTunnel
  - Phase 5 (Observability): vector, opentelemetry 相关
  - Phase 6 (Auth): oauth2, jsonwebtoken (已声明)
  - Phase 8 (Desktop): tauri-plugin-window-state (已声明), tray 相关

### Agent's Discretion
- tower-http TimeoutLayer 默认超时值（建议 30s）
- CorsLayer 具体 allow_origins 配置（permissive 开发环境，生产环境收紧）
- request-id 中间件实现细节（header 名称，生成逻辑）
- moon task 中 cargo-bloat 的具体命令行参数

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 4 — Phase goal, success criteria, dependencies
- `.planning/REQUIREMENTS.md` §PKG-04, §BUILD-01 — Acceptance criteria for axum deps versioning and binary under 15MB

### Stack & Architecture
- `.planning/PROJECT.md` §Tech stack — Backend stack overview (Axum 0.8.8, Tokio)
- `docs/toDev/TECH_SELECTION.md` §七 — Cargo 依赖版本已验证列表

### Workspace Configuration
- `Cargo.toml` — Root workspace dependencies and release profile (Phase 1 基础配置)
- `crates/runtime_server/Cargo.toml` — 当前仅声明 domain + shared_contracts，需要补充 axum stack

### Axum 0.8
- [Axum 0.8 migration guide](https://github.com/tokio-rs/axum/blob/main/axum/CHANGELOG.md) — 从 0.7 升级的关键变更
- tower-http 0.6 docs — middleware layers API (CorsLayer, TraceLayer, TimeoutLayer)

### Existing Code
- `crates/runtime_server/src/lib.rs` — Placeholder, 仅一行注释
- `apps/desktop-ui/src-tauri/src/lib.rs` — Tauri entry point, 已注册 3 个 plugin

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **workspace.dependencies** (`Cargo.toml`): 已声明 axum 0.8.8, tokio 1.50.0, reqwest 0.13.2, serde, serde_json, uuid, chrono — runtime_server 直接 workspace=true 引用即可
- **[profile.release]** (`Cargo.toml`): 已配置 lto=true, codegen-units=1, opt-level="z", strip=true — 仅需添加 panic="abort"
- **Clean Architecture crates**: domain, application, shared_contracts 各有 lib.rs placeholder — runtime_server 作为最外层引用它们

### Established Patterns
- workspace.dependencies 统一管理版本 — 新增依赖必须走此模式
- 所有 crate 用 edition = "2021"
- crate 内依赖 path 引用（crates 之间）+ workspace 引用（公共依赖）

### Integration Points
- `crates/runtime_server/Cargo.toml` — 需要从 workspace 引入 axum, tokio, tower, tower-http, hyper, serde, serde_json
- `crates/runtime_server/src/lib.rs` — 需要从 placeholder 改为实际服务器代码入口
- `moon.yml` — 需要添加 cargo-bloat 监控 task

</code_context>

<specifics>
## Specific Ideas

- Axum 服务器架构参考 Clean Architecture 分层：runtime_server 作为最外层 HTTP adapter
- /healthz 和 /readyz 是标准 K8s 健康检查端点命名 — 为未来容器化部署预留
- cargo-bloat 做依赖大小审计 — 比手动 `du` 更精确定位膨胀源

</specifics>

<deferred>
## Deferred Ideas

- Axum 实际业务路由（Phase 5+）
- 数据库连接池配置（Phase 5）
- JWT 鉴权中间件（Phase 6）
- 实际 CORS allow_origins 生产配置（Phase 9 部署时）

</deferred>

---

*Phase: 04-backend-dependencies-build-optimization*
*Context gathered: 2026-03-28*
