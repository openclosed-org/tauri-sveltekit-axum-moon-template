# Phase 9: 功能正确性基线修复 - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

<domain>
## Phase Boundary

修复既有认证、计数器、Agent 关键交互路径中的正确性缺陷，使用户在桌面与浏览器路径上都能稳定完成：退出登录、计数器变更、New Chat 新会话与连接性验证，并形成可回归的行为基线。

本阶段不新增业务能力域，只澄清并锁定既有能力的实现与验收口径。

</domain>

<decisions>
## Implementation Decisions

### 退出入口与会话清理
- **D-01:** 退出动作入口放在 Settings 页面，不在主侧栏新增入口。
- **D-02:** 退出执行采用“双端失效 + 本地兜底”：先尝试服务端/会话失效，再执行本地凭据清理；即使前者失败也必须完成本地退出。
- **D-03:** 本地退出必须清理可复用会话凭据，确保 desktop 与 browser 路径都不会复用上一会话。
- **D-04:** 退出后优先返回上一个公开页面；若无可用公开页面则回到 `/login`。

### 计数器一致性与错误反馈
- **D-05:** 计数器操作失败时显示页面内错误条，并保留上一次成功值，不做静默失败。
- **D-06:** 一致性规则锁定为“进入即拉取 + 命令返回值为准”：页面进入时读取持久值；每次增减/重置后以后端返回值更新展示值。
- **D-07:** 首次进入读取失败时，`count` 维持 `0`，同时展示可见错误反馈。
- **D-08:** 回归验证需覆盖“修改后刷新页面，显示值与持久化值一致”。

### Agent 新会话与设置保持
- **D-09:** 点击 New Chat 后立即切换到新会话，消息区为空白并可直接输入发送。
- **D-10:** New Chat 仅要求保持 `API key`、`base URL`、`model` 三项配置不变，其他会话内容按新线程语义重置。
- **D-11:** 读取 `settings.json` 失败时，允许回退默认值继续流程，但必须给出可操作提示（引导用户修正设置）。

### 连通性测试与反馈
- **D-12:** 在 Settings 页面保存按钮旁新增 `Test Connection` 动作入口。
- **D-13:** 连通性结果按三项分开反馈：`API key`、`Base URL`、`Model` 各自输出 pass/fail 与下一步建议。
- **D-14:** 测试失败后不自动回滚当前配置值，保留用户输入用于就地修正与重试。

### the agent's Discretion
- 错误与建议文案的具体措辞（在不改变决策语义前提下）。
- 连通性测试的超时与重试预算。
- 反馈组件的具体呈现形式（inline banner/card）与视觉样式。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 阶段范围与验收口径
- `.planning/ROADMAP.md` — Phase 9 目标、依赖、Success Criteria。
- `.planning/REQUIREMENTS.md` — AUTH-02, AUTH-03, COUNTER-02, AGENT-02, AGENT-03, AGENT-04。
- `.planning/PROJECT.md` — v0.2.1 约束、架构与里程碑背景。

### 架构与实现边界
- `docs/blueprints/agent-native-starter-v1/05-runtime-features-and-adapters.md` — runtime/features/adapters 边界。
- `docs/blueprints/agent-native-starter-v1/06-engineering-standards-rust-tauri-svelte.md` — 前后端分层与路由/store 规范。

### 当前行为基线（实现对齐参考）
- `apps/client/web/app/src/routes/(app)/+layout.svelte` — 受保护路由与导航结构基线。
- `apps/client/web/app/src/lib/stores/auth.svelte.ts` — `checkSession` / `signOut` / `markExpired` 行为。
- `apps/client/web/app/src/routes/(app)/counter/+page.svelte` — 计数器读写与当前错误处理方式。
- `apps/client/web/app/src/routes/(app)/agent/+page.svelte` — New Chat 与发送前设置加载路径。
- `apps/client/web/app/src/routes/(app)/settings/+page.svelte` — 设置读写入口与 UI 基线。

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `auth` store (`apps/client/web/app/src/lib/stores/auth.svelte.ts`): 已有会话检查、清理与登录态变更入口，可直接承接退出策略。
- Tauri Store (`apps/client/web/app/src/routes/(app)/settings/+page.svelte`, `apps/client/web/app/src/routes/(app)/agent/+page.svelte`): 已有 `settings.json` 读写链路。
- Agent IPC (`apps/client/web/app/src/lib/ipc/agent.ts`): 已有 desktop IPC + browser HTTP 双路径模式。
- Counter commands (`packages/adapters/hosts/tauri/src/commands/counter.rs`) 与 service (`packages/core/usecases/src/counter_service.rs`): 已有稳定数据读写 API。

### Established Patterns
- 运行时分流：`__TAURI__` 检测后优先 IPC，失败再按运行环境 fallback。
- 路由守卫：`(app)` layout 统一鉴权检查，未登录跳转 `/login`。
- 设置与会话存储：通过 `@tauri-apps/plugin-store` 管理 `auth.json` / `settings.json`。
- 错误展示：页面内错误条（agent 页面 `loadError`）与全局 Toast 并存。

### Integration Points
- Settings 页面：新增退出入口与 `Test Connection` 入口。
- Auth IPC/command：补充可显式调用的 logout 失效路径（与本地清理联动）。
- Counter 页面：补齐失败可见反馈与一致性回归断言。
- Agent 页面：New Chat 切换与设置保持语义对齐并补失败提示。

</code_context>

<specifics>
## Specific Ideas

- 退出入口仅放 Settings 页面。
- 退出采用“双端失效 + 本地兜底”，并优先返回上一个公开页面。
- 连通性测试必须拆成 API key / Base URL / Model 三项独立结果，不做单一总结果。

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-functional-correctness-baseline-fix*
*Context gathered: 2026-04-06*
