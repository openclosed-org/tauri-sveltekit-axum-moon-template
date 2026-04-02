# Phase 4: 最小功能实现 - Context

**Gathered:** 2026-04-02
**Status:** Ready for planning

<domain>
## Phase Boundary

用户可以通过 Google 登录、使用计数器、访问 Admin、与 Agent 对话。4 个功能通过 feature 模块实现，使用 contracts 类型，符合 Phase 3 建立的边界规则。

本阶段实现 4 个功能的完整闭环：feature crate（Rust）+ contracts + adapters + 前端页面（Svelte）+ IPC 层。

**NOT in scope:** 复杂多 agent 协作、完整 RBAC、email/password auth、实时协作。

</domain>

<decisions>
## Implementation Decisions

### Auth — Full Refactor into Feature+Adapter Pattern

- **D-01:** 将现有 auth 代码从 runtime_tauri/commands/auth.rs 完整重构到 feature-auth + adapter-auth 架构
- **D-02:** OAuth adapter 放在 `packages/adapters/auth/google/`（独立 crate，不是 `oauth/`），为其他 OAuth provider 预留 `packages/adapters/auth/github/` 等
- **D-03:** 保留两套类型定义：runtime_tauri 层使用 AuthSession/UserProfile（Tauri store 用），contracts_auth 使用 TokenPair/UserSession（API 层用）。两者服务于不同关注点，不强制统一
- **D-04:** feature-auth 定义 AuthService use case trait，方法包括：`start_login`, `handle_callback`, `get_session`, `refresh_token`, `logout`。Runtime_tauri command handler 实现此 trait

### Counter — LibSQL 持久化 + 双路径 IPC

- **D-05:** Counter 值持久化到嵌入式 LibSQL。创建 counter 表（id, value, updated_at）。通过现有 SyncEngine 支持可选的云端同步（TURSO_SYNC_URL 配置时）
- **D-06:** 双路径 IPC：前端运行时检测 — Tauri 环境用 `invoke()`，浏览器环境用 `fetch()` 到 Axum server。Both paths 调用同一套 usecases
- **D-07:** Counter operations: increment (+1), decrement (-1), reset (0), get_value。每个操作对应 Tauri command + HTTP endpoint

### Agent Chat — 对话 + 只读工具调用

- **D-08:** Agent 对话 + 固定工具集（只读）：`get_counter_value`, `list_tenants`, `get_system_status`。工具集写死，不允许用户扩展。证明 function calling 模式
- **D-09:** 用户在 Settings 页面输入 OpenAI 兼容 API key + base URL。存储在 Tauri store 中。首次使用时提示，可在 Settings 修改
- **D-10:** 对话历史持久化到 LibSQL。Conversations 表（id, title, created_at）+ Messages 表（id, conversation_id, role, content, tool_calls, created_at）
- **D-11:** 流式响应：Axum server 端调用 OpenAI API 时使用 SSE streaming，Tauri 环境通过 Tauri event streaming 传递到前端。逐 token 显示

### Admin — 真实数据 + Usecases 驱动

- **D-12:** Admin 展示真实数据库数据。4 个统计卡片：Tenant 数量、Counter 当前值、最近登录时间、App 版本号。数据随实际 app 状态变化
- **D-13:** 通过 feature-admin crate + usecases 获取数据。不直接读 adapter，走 hexagonal 边界

### IPC Architecture — 双路径 + Per-Feature 模块

- **D-14:** 默认 IPC 策略：每个 feature 的 IPC 模块检测运行时。Tauri: invoke()。Browser: fetch() 到 Axum server。Both paths 调用同一套 usecases
- **D-15:** Axum server 为 counter、admin、agent 提供 REST API 路由。Axum 不仅是外部 API gateway，也是 in-app HTTP 通道
- **D-16:** IPC 抽象层按 feature 模块组织（如 `packages/features/counter/ipc/`），不放在共享 `lib/ipc/` 目录。Auth 的 IPC 也迁移到 feature-auth

### the agent's Discretion

- feature crate 的具体命名（`feature-auth` vs `auth-feature`）
- AuthService trait 的具体方法签名
- LibSQL migration 的具体 schema 设计
- Agent chat UI 的具体布局（sidebar + chat area vs full-page chat）
- Streaming 的具体实现方式（SSE vs Tauri event channel）
- IPC 运行时检测的具体机制（window.__TAURI__ check vs import.meta.env.TAURI）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 4 — Phase goal, success criteria, requirements
- `.planning/REQUIREMENTS.md` §AUTH-01, §COUNTER-01, §ADMIN-01, §AGENT-01 — Acceptance criteria

### Prior Phase Decisions (locked context)
- `.planning/phases/01-repo-structure-toolchain/01-CONTEXT.md` — 目录结构、moon tasks、Cargo workspace
- `.planning/phases/02-contracts-typegen/02-CONTEXT.md` — contracts crate 结构、typegen 管线、DTO 命名
- `.planning/phases/03-runtime-boundary-convergence/03-CONTEXT.md` — feature crate pattern, runtime_tauri 职责, 边界规则, D-17~D-20 关于 features 的定义

### Architecture & Blueprint
- `docs/ARCHITECTURE.md` — Layer boundaries, hexagonal 原则, feature 组合模型, quality gates
- `docs/blueprints/agent-native-starter-v1/05-runtime-features-and-adapters.md` — Feature 骨架规范、adapter 分层、组合模型、易犯错误
- `docs/blueprints/agent-native-starter-v1/04-contracts-typegen-and-boundaries.md` — Contracts 单一真理源原则、DTO 与 domain 的关系、边界规则
- `docs/blueprints/agent-native-starter-v1/06-engineering-standards-rust-tauri-svelte.md` — 命名规范、代码风格

### Existing Code (migration/extension targets)
- `packages/adapters/hosts/tauri/src/commands/auth.rs` — 现有 Google OAuth 完整实现（422 行），待重构到 adapter-google
- `apps/client/web/app/src/lib/stores/auth.svelte.ts` — 前端 auth store，需迁移到 feature-auth
- `apps/client/web/app/src/lib/ipc/auth.ts` — 前端 auth IPC，需迁移到 feature-auth
- `apps/client/native/src-tauri/src/lib.rs` — native-tauri entry point，需注册新 commands
- `apps/client/web/app/src/routes/(app)/counter/+page.svelte` — 现有纯前端 counter，需加入 IPC
- `apps/client/web/app/src/routes/(app)/admin/+page.svelte` — 现有 mock admin，需接入真实数据
- `packages/contracts/auth/src/lib.rs` — contracts_auth 已有 TokenPair, OAuthCallback, UserSession
- `packages/contracts/api/src/lib.rs` — contracts_api 已有 HealthResponse, InitTenantRequest/Response

### Cargo Configuration
- `Cargo.toml` — workspace members 和 workspace.dependencies（需添加 feature crates + auth/google adapter）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `packages/adapters/hosts/tauri/src/commands/auth.rs` — 完整的 Google OAuth PKCE 实现（start_oauth, handle_oauth_callback, refresh_access_token, token refresh timer），可整体迁移到 adapter-google
- `apps/client/web/app/src/lib/stores/auth.svelte.ts` — 响应式 auth store（Svelte 5 runes），可迁移到 feature-auth
- `apps/client/web/app/src/lib/ipc/auth.ts` — Tauri invoke() wrapper，可作为 IPC 双路径的 Tauri 分支基础
- `packages/contracts/auth/src/lib.rs` — TokenPair, OAuthCallback, UserSession 已定义且通过 ts-rs 测试
- `apps/client/web/app/src/lib/components/ui/` — Button, Card, Badge, Input, Dialog, Toast 等 11 个 UI 组件可直接复用
- `packages/core/usecases/src/tenant_service.rs` — TenantService trait + LibSqlTenantService 实现，可作为 admin 数据源
- `apps/client/native/src-tauri/src/commands/sync.rs` — SyncState + SyncEngine 模式可参考用于 counter 同步

### Established Patterns
- Tauri command pattern: `#[tauri::command]` async fn 接收 `State<'_, AppState>` 或 `AppHandle`，返回 `Result<T, String>`
- Svelte 5 runes: `$state()`, `$effect()`, `$props()` — 前端所有组件使用 runes 模式
- Cargo workspace: path dependencies + workspace.dependencies 统一版本
- LibSQL: `EmbeddedLibSql::new(":memory:")` + `run_tenant_migrations` 模式
- Serde + utoipa + ts-rs derive 宏共存于同一 struct

### Integration Points
- `Cargo.toml` root — 需添加 feature-auth, feature-counter, feature-admin, feature-agent, adapters/auth/google 为 workspace members
- `apps/client/native/src-tauri/src/lib.rs` — `tauri::generate_handler![...]` 需注册新 counter/admin/agent commands
- `apps/client/web/app/src/routes/(app)/+layout.svelte` — 导航栏需添加 Agent Chat 入口
- `servers/api/src/routes/mod.rs` — 需添加 counter, admin, agent HTTP 路由
- `packages/core/usecases/src/lib.rs` — 需添加 counter_service, agent_service, admin_service 模块
- `packages/core/domain/src/ports/lib_sql.rs` — LibSqlPort trait 已定义，counter/admin 可直接使用

### Gaps
- feature-auth, feature-counter, feature-admin, feature-agent 均为 .gitkeep 空壳，需创建完整 Cargo.toml + src/lib.rs
- `packages/adapters/auth/google/` 目录不存在，需创建
- 无 agent 相关的 contracts 定义（需要 ChatMessage, ToolCall, AgentConfig 等 DTO）
- Axum server 无 counter/admin/agent 路由
- 无 IPC 运行时检测工具函数
- Settings 页面不存在（需创建）

</code_context>

<specifics>
## Specific Ideas

- Auth refactor: 保持现有 Google OAuth PKCE 逻辑不变，只重构代码组织（移动文件、提取 trait、创建 adapter crate）。功能行为不改变
- Counter: 类似 "hello world" 但验证完整 IPC 管线。Value 是单个 integer，不是多条记录
- Agent: 工具调用格式遵循 OpenAI function calling spec。System prompt 可以包含 product state 的简要描述
- Admin: 数据从 real sources 聚合，但 UI 保持当前的卡片+图表布局。图表可保持 CSS bar chart（不引入 chart.js）
- IPC 双路径: `window.__TAURI__` 存在性检测是最简单的运行时识别方式

</specifics>

<deferred>
## Deferred Ideas

- Agent tool calling 的写操作（create_tenant, reset_counter）— 后续版本
- Email/password auth — Out of scope per REQUIREMENTS.md
- Full RBAC for admin — Out of scope for v0.1
- Counter 的多计数器支持（目前只有一个 global counter）
- Agent 对话的多模态支持（图片、文件）— 未来
- Settings 页面的完整实现（theme, language, notifications 等）— 本阶段只实现 API key 配置

</deferred>

---

*Phase: 04-minimal-feature-implementation*
*Context gathered: 2026-04-02*
