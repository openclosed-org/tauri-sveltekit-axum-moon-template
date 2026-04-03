# Phase 8: Agent 双路径 + Prompts + Phase 5 验证 - Context

**Gathered:** 2026-04-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Agent 页面支持 Tauri IPC 双路径（Tauri 环境走 Channel API streaming，浏览器走 HTTP SSE），补全 .agents/prompts/ 内容（用户亲自写），生成 Phase 5 VERIFICATION.md（只验证已有 playbooks + rubrics）。

**In scope:**
- `agent/+page.svelte` 改造：运行时检测 Tauri 环境，Tauri 走 invoke + Channel，浏览器走 fetch + SSE
- 新建 `lib/ipc/agent.ts` 封装双路径 agent client
- Tauri command 层新增 agent chat handlers（使用 async-openai crate）
- 创建 `.agents/prompts/` 模板（add-feature, add-host, refactor-boundary）— 用户亲自写
- 生成 Phase 5 VERIFICATION.md（验证 playbooks + rubrics 可用性）
- Desktop Mode E2E 完整对话验证

**NOT in scope:**
- Prompts 的具体内容（用户亲自写）
- 复杂多 agent 协作（V3 候选）
- Agent tool calling 的写操作（未来版本）

</domain>

<decisions>
## Implementation Decisions

### IPC 双路径 — Agent Chat

- **D-01:** Tauri 端 streaming 使用 **Tauri 2 Channel API** — 前端传 Channel 给 invoke，Rust 端通过 channel.send() 逐 token 推送。类型安全，官方推荐，避免 Event 全局冲突
- **D-02:** IPC 模块放在 `apps/client/web/app/src/lib/ipc/agent.ts`，与现有 `auth.ts` 并列。保持当前目录结构一致，最小改动
- **D-03:** Tauri 端 LLM 调用使用 **async-openai** crate（698K 下载/月，最成熟，原生支持 streaming）。与现有 `feature_agent` 的 `chat_stream` 模式一致，最小依赖
- **D-04:** `agent/+page.svelte` 改造：保留现有 HTTP SSE 路径不变，新增 Tauri 路径。运行时检测 `window.__TAURI__`，Tauri 环境走 `invoke('agent_chat', { channel })`，浏览器走现有 `fetch()` + SSE
- **D-05:** Tauri commands 新增到 `runtime_tauri/commands/agent.rs`，注册到 `commands/mod.rs` 和 `native-tauri` 的 handler list

### Prompts 模板

- **D-06:** `.agents/prompts/` 目录结构由用户亲自创建和编写，本阶段只确保目录存在（已有 .gitkeep）

### Phase 5 验证

- **D-07:** Phase 5 VERIFICATION.md 只验证已有 assets（playbooks + rubrics）的文件存在性、格式正确性、内容可执行性
- **D-08:** Skills 和 Prompts 标记为 deferred，不阻塞 Phase 5 通过（与 Phase 5 CONTEXT.md D-01/D-03 一致）

### Desktop Mode E2E

- **D-09:** Desktop Mode E2E 验证完整对话流程（非仅 IPC 通路），需要真实 API key 验证 streaming 显示
- **D-10:** E2E 验证链路：Tauri invoke → Rust command → async-openai → OpenAI API → Channel streaming → 前端逐 token 显示

### the agent's Discretion

- Tauri Channel API 的具体类型签名和错误处理
- `agent_chat` command 的参数设计（conversation_id, content, api_key, base_url, model, channel）
- async-openai 的 client 初始化和连接复用策略
- Phase 5 VERIFICATION.md 的具体验证步骤和通过标准
- E2E 测试的具体实现方式（Playwright Tauri 模式 vs 手动验证）

### Folded Todos

None — no pending todos matched this phase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 8 — Phase goal, success criteria, requirements
- `.planning/REQUIREMENTS.md` §AGENT-01, §AGENT-DEV-01 — Acceptance criteria

### Prior Phase Decisions (locked context)
- `.planning/phases/04-minimal-feature-implementation/04-CONTEXT.md` — D-06/D-08/D-09/D-10/D-11/D-14/D-16 双路径 IPC 策略、agent 对话架构
- `.planning/phases/05-agent-friendly/05-CONTEXT.md` — Phase 5 playbooks + rubrics 决策，skills/prompts 跳过决定

### Architecture & Blueprint
- `docs/blueprints/agent-native-starter-v1/` — 蓝图目录，定义整体架构方向
- `docs/ARCHITECTURE.md` — Layer boundaries, hexagonal 原则

### Existing Code (implementation targets)
- `apps/client/web/app/src/routes/(app)/agent/+page.svelte` — 现有 agent 页面（234 行），需改造双路径
- `apps/client/web/app/src/lib/ipc/auth.ts` — 现有 IPC 模式参考
- `packages/adapters/hosts/tauri/src/commands/counter.rs` — Tauri command 模式参考
- `packages/adapters/hosts/tauri/src/commands/mod.rs` — 需添加 agent 模块
- `apps/client/native/src-tauri/src/lib.rs` — 需注册 agent commands
- `packages/features/agent/src/lib.rs` — AgentService trait 定义
- `packages/core/usecases/src/agent_service.rs` — LibSqlAgentService 实现，chat_stream 接口
- `servers/api/src/routes/agent.rs` — Axum agent 路由（HTTP SSE 路径参考）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `agent/+page.svelte` — 已有完整聊天 UI（sidebar + conversation list + message area + input），已有 `loadSettings()` 检测 `window.__TAURI__` 读 Tauri store
- `lib/ipc/auth.ts` — Tauri invoke() wrapper 模式可参考
- `commands/counter.rs` — Tauri command 桥接 usecases 的标准模式
- `feature_agent::AgentService` trait — 已有 `chat_stream` 接口定义
- `usecases::agent_service::LibSqlAgentService` — 已有完整实现（OpenAI streaming + tool calling）
- `.agents/playbooks/create-feature.md` — 完整的 feature 创建流程规范
- `.agents/playbooks/update-contracts.md` — 完整的 contracts 修改流程规范
- `.agents/rubrics/boundary-compliance.md` — 六边形架构边界规则
- `.agents/rubrics/code-review.md` — 代码审查标准
- `.agents/rubrics/task-completion.md` — 任务完成标准

### Established Patterns
- Tauri command: `#[tauri::command]` async fn 接收 `AppHandle`，通过 `app.state::<EmbeddedLibSql>()` 获取 DB
- Svelte 5 runes: `$state()`, `$effect()` — agent 页面已使用
- IPC 双路径: `window.__TAURI__` 存在性检测（agent 页面 `loadSettings()` 已用此模式）
- SSE streaming: Axum 端已有 `chat_stream` → SSE response 实现
- Cargo workspace: feature crates + usecases + adapters 分层

### Integration Points
- `packages/adapters/hosts/tauri/src/commands/mod.rs` — 需添加 `pub mod agent;`
- `apps/client/native/src-tauri/src/lib.rs` — `tauri::generate_handler![...]` 需注册 agent commands
- `Cargo.toml` — 需添加 `async-openai` 依赖
- `apps/client/web/app/src/lib/ipc/` — 需新建 `agent.ts`

</code_context>

<specifics>
## Specific Ideas

- IPC 双路径: `window.__TAURI__` 存在性检测是最简单的运行时识别方式（agent 页面 loadSettings 已用）
- Tauri 2 Channel API: 前端创建 Channel 传给 invoke，Rust 端通过 channel.send() 逐 token 推送，比 Event 更安全（无全局冲突）
- async-openai: 与现有 usecases/agent_service.rs 的 OpenAI 调用逻辑一致，可以复用或共享 client 配置
- Phase 5 验证: 只验证 playbooks + rubrics 文件存在且格式正确，skills 和 prompts 标记为 deferred
- Desktop E2E: 需要真实 API key 验证完整对话，包括 streaming 逐 token 显示
- Prompts: 用户亲自写，本阶段不创建具体内容

</specifics>

<deferred>
## Deferred Ideas

- Prompts 具体内容（add-feature, add-host, refactor-boundary）— 用户亲自写
- Skills 具体内容（rust-core, tauri-host, sveltekit-ui, contracts-typegen, testing）— Phase 5 决定跳过
- Agent tool calling 的写操作（create_tenant, reset_counter）— 未来版本
- Agent 对话的多模态支持（图片、文件）— 未来

### Reviewed Todos (not folded)

None — no pending todos matched this phase.

</deferred>

---

*Phase: 08-agent-dualpath-prompts*
*Context gathered: 2026-04-02*
