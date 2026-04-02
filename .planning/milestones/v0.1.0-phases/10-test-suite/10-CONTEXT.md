# Phase 10: Test Suite - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

为核心应用流程建立并跑通三层测试：Rust unit、Svelte component、Playwright E2E，且核心流测试结果必须为绿色并且不允许通过 skip/ignore 规避。

本阶段聚焦测试覆盖与门禁，不新增业务功能。

</domain>

<decisions>
## Implementation Decisions

### Auth E2E Strategy
- **D-01:** Playwright 登录流不走真实 Google OAuth，采用 mock callback 策略，确保 CI 可重复、稳定。
- **D-02:** Mock 以模拟 `deep-link://new-url` 事件为主，复用现有 `+layout.svelte` 中 OAuth callback 处理链路，而不是直接写登录态。

### Core Flow Coverage Boundary
- **D-03:** Core flows 扩展为：login、counter、admin、tenant 隔离、token 刷新。
- **D-04:** 上述 core flows 对应测试不得使用 skip/ignore。
- **D-05:** `tenant` 与 `token refresh` 采用双重覆盖：Rust 单测/集成测试 + E2E 可感知行为验证。

### Test Layering and Directory Layout
- **D-06:** Rust 保持“模块内单测 + 集成测试目录”双层结构：保留现有 `#[cfg(test)]`，并补充 `crates/runtime_server/tests/` 跨模块测试。
- **D-07:** 前端测试目录集中管理：`apps/desktop-ui/tests/component/`（Vitest）与 `apps/desktop-ui/tests/e2e/`（Playwright）。

### CI Gate Policy
- **D-08:** PR 必跑全量测试门禁：`cargo test` + Vitest component tests + Playwright E2E。
- **D-09:** 任一测试层失败即阻断合并，不降级为仅 main 分支全量执行。

### the agent's Discretion
- Mock deep-link 测试辅助函数的具体实现位置（test utils 或 per-suite fixture）。
- Vitest/Playwright 配置细节（project 划分、重试、并发、超时）。
- CI 中测试执行顺序与缓存优化策略（在不放松 D-08/D-09 的前提下）。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and acceptance anchors
- `.planning/ROADMAP.md` §Phase 10 — Test Suite 的目标、依赖与成功标准
- `.planning/REQUIREMENTS.md` §TEST-01, §TEST-02, §TEST-03 — 三层测试验收要求
- `.planning/PROJECT.md` §Core Value, §Constraints, §Testing — 模板定位与测试约束

### Existing auth flow (for E2E mock boundary)
- `apps/desktop-ui/src/routes/(auth)/login/+page.svelte` — 登录页触发 OAuth 与错误/加载态
- `apps/desktop-ui/src/routes/+layout.svelte` — `deep-link://new-url` 监听与 callback 处理入口
- `apps/desktop-ui/src/lib/stores/auth.svelte.ts` — 前端登录态、过期态与事件监听逻辑
- `apps/desktop-ui/src-tauri/src/commands/auth.rs` — OAuth 命令、session 存储、refresh timer 与 `auth:expired` 事件

### Existing core flows to cover
- `apps/desktop-ui/src/routes/(app)/counter/+page.svelte` — counter 交互核心流
- `apps/desktop-ui/src/routes/(app)/admin/+page.svelte` — admin 导航与页面渲染核心流
- `crates/runtime_server/src/middleware/tenant.rs` — tenant token 提取中间件
- `crates/runtime_server/src/ports/surreal_db.rs` — tenant-aware SQL 注入与隔离逻辑
- `crates/runtime_server/src/routes/tenant.rs` — tenant init API 行为

### Existing test/build execution surfaces
- `apps/desktop-ui/package.json` — `test:unit`, `test:e2e`, `test:mobile` 脚本基线
- `.github/workflows/ci.yml` — 当前 CI 流水线与待增强门禁入口
- `moon.yml` — workspace 聚合任务（lint/test）现状
- `apps/desktop-ui/moon.yml` — 前端任务定义（build/check/lint，待接入测试任务）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/runtime_server/src/middleware/tenant.rs`、`crates/runtime_server/src/ports/surreal_db.rs`、`crates/runtime_server/src/routes/tenant.rs` 已有 Rust 单测，可直接扩展为更完整断言。
- `apps/desktop-ui/src/routes/+layout.svelte` 已提供 deep-link 监听与 panic/auth 事件处理，可作为 E2E 触发点。
- `apps/desktop-ui/src/lib/stores/auth.svelte.ts` 已集中管理 auth state，便于 component/E2E 校验登录与过期行为。
- `apps/desktop-ui/package.json` 已有 Vitest/Playwright 依赖与脚本，无需新增测试框架依赖。

### Established Patterns
- 前端以 Bun 脚本运行检查与构建；CI 也统一使用 Bun。
- Rust 侧已有 `#[cfg(test)]` 风格并分布在 runtime_server 模块中。
- CI 当前为三平台 matrix，但测试门禁仍偏基础，尚未纳入 Vitest/Playwright 执行。

### Integration Points
- 新增并接线 `apps/desktop-ui/tests/component/**` 与 `apps/desktop-ui/tests/e2e/**`。
- 为前端补充测试配置入口（Vitest/Playwright config）并与现有脚本/CI 绑定。
- 在 `crates/runtime_server/tests/` 新增跨模块集成测试，覆盖 tenant 与 refresh 相关核心行为。
- 更新 `.github/workflows/ci.yml` 使 PR 执行三层全量测试门禁。

</code_context>

<specifics>
## Specific Ideas

- Auth E2E 明确采用“模拟 deep-link 事件”而非“直接 setSession”，以保留 callback 链路覆盖。
- Core flows 相比 ROADMAP 最小集合进一步收紧到“业务三流 + 安全隔离 + 会话续期”。
- CI 选择 PR 阶段全量必跑，不采用“PR 轻量 / main 全量”的延迟暴露模式。

</specifics>

<deferred>
## Deferred Ideas

- 真实 Google OAuth 端到端回归（含真实账号）可作为后续独立 smoke/staging 流程，不作为本阶段核心门禁。

</deferred>

---

*Phase: 10-test-suite*
*Context gathered: 2026-03-30*
