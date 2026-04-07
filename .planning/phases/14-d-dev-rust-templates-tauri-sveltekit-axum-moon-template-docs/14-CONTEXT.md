# Phase 14: 请问根据D:\dev\rust\templates\tauri-sveltekit-axum-moon-template\docs\TAURI_PLAYWRIGHT_MIGRATION_CONTEXT.md 改造升级我的E2E系统,同时还需要完成跑通E2E的测试 - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

在不改变业务逻辑、API 契约和用户可见行为的前提下，按 `docs/TAURI_PLAYWRIGHT_MIGRATION_CONTEXT.md` 对当前 E2E 体系做可回滚升级：将桌面 E2E 从 `WDIO + tauri-driver` 逐步迁移到 `tauri-playwright`，并把“跑通 E2E”的验收定义为仓库级全量 E2E 通过。

本阶段范围包含迁移到 Phase 1 深度（smoke/login/counter）、新桌面 Playwright 套件落位、CI 过渡接线与证据产出；不包含业务功能新增。

</domain>

<decisions>
## Implementation Decisions

### 迁移深度与节奏
- **D-01:** 本阶段迁移深度锁定到 Migration Context 的 **Phase 1**，而不是仅做 Phase 0 引导层。
- **D-02:** Phase 1 首批迁移用例锁定为 `smoke`、`login`、`counter`（基础交互），并保持旧 WDIO 套件可运行作为回滚路径。

### 套件结构与代码组织
- **D-03:** 新桌面 Playwright 套件采用**独立目录新套件**方案（与现有 web Playwright 和 WDIO 解耦），不复用同一测试根目录。
- **D-04:** 新套件必须复用/对齐现有认证与租户测试语义（mock deep-link 登录、稳定 tenant identity），避免 fixture 漂移。

### CI 迁移策略
- **D-05:** CI 先新增 `desktop-e2e-playwright-tauri` 的 **macOS 观察通道**，作为过渡期可观测能力；不立即替代现有主通道。
- **D-06:** 过渡期保持现有 `desktop-e2e`（WDIO）与 `web-e2e`（Playwright）并行运行，直到达到稳定性判定门槛再讨论 WDIO 退场。

### E2E 跑通验收口径
- **D-07:** “跑通 E2E”锁定为**仓库所有 E2E 项目全绿**（而非仅迁移范围）：包含现有 WDIO 全部规范、现有 Playwright 全项目矩阵、以及新增 tauri-playwright 迁移用例。
- **D-08:** 验收必须包含可诊断证据产出（report/screenshot/video 或等效 artifacts），确保失败可追踪而非只看绿灯。

### 兼容性与安全约束
- **D-09:** `tauri-plugin-playwright` 接入必须受 `e2e-testing` feature 或 `debug_assertions` 门控；release build 不得暴露自动化控制面。
- **D-10:** 迁移实现禁止通过修改产品行为来迎合测试，若新套件与现有行为冲突，优先修正测试实现而非改业务语义。

### the agent's Discretion
- 新套件目录名与内部文件分层（在满足独立套件前提下）。
- 迁移顺序中每个用例的具体断言颗粒度与 helper 抽象层级。
- CI job 命名细节与 artifact 打包目录布局（不改变 D-05~D-08）。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 迁移策略与硬约束
- `docs/TAURI_PLAYWRIGHT_MIGRATION_CONTEXT.md` — 迁移总目标、硬约束、分阶段策略、CI 设计、质量门槛与回滚契约。

### 阶段范围与项目约束
- `.planning/ROADMAP.md` — Phase 14 条目与依赖关系。
- `.planning/PROJECT.md` — 当前里程碑约束（跨平台门禁、缺陷闭环、最小爆炸半径）。
- `.planning/REQUIREMENTS.md` — v0.2.1 质量门禁与测试治理需求背景。
- `.planning/phases/10-multi-tenant-repeatable-verification-channel/10-CONTEXT.md` — 既有多租户测试语义与证据约定（需保持兼容）。

### 当前 E2E 与 CI 基线
- `.github/workflows/e2e-tests.yml` — 现有 desktop/web E2E job、matrix、artifact retention 基线。
- `apps/client/web/app/playwright.config.ts` — web Playwright 项目矩阵、reporter、trace/video/screenshot 策略。
- `e2e-tests/wdio.conf.mjs` — 现有 WDIO desktop 运行模型与 diagnostics/JUnit 输出路径。
- `e2e-tests/scripts/run-desktop-e2e.mjs` — WDIO CI 入口、平台限制与诊断文件产出行为。
- `e2e-tests/package.json` — desktop E2E 现有脚本入口（`test`/`test:ci`）。
- `apps/client/web/app/package.json` — web E2E 与 desktop 触发脚本基线。

### 认证/租户行为兼容基线
- `apps/client/web/app/tests/fixtures/auth.ts` — mock OAuth deep-link 触发机制。
- `apps/client/web/app/tests/fixtures/tenant.ts` — web 侧固定双租户初始化与 reset 语义。
- `e2e-tests/helpers/tenant.mjs` — desktop 侧固定双租户 token 与 counter helper 语义。

### Tauri 接入触点
- `apps/client/native/src-tauri/Cargo.toml` — 插件依赖与 feature 门控触点。
- `apps/client/native/src-tauri/src/lib.rs` — 插件注册与条件编译触点。
- `apps/client/native/src-tauri/capabilities/default.json` — capability 权限配置触点。

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `apps/client/web/app/tests/fixtures/auth.ts`: 已有 deep-link mock 登录能力，可复用于迁移后行为对齐验证。
- `apps/client/web/app/tests/fixtures/tenant.ts` + `e2e-tests/helpers/tenant.mjs`: 已有稳定双租户身份与 reset/read/increment helper，可作为新套件 fixture 语义基准。
- `apps/client/web/app/playwright.config.ts`: 已沉淀 Playwright reporter 与失败证据策略，可复用到桌面套件设计。
- `e2e-tests/scripts/run-desktop-e2e.mjs`: 已有 diagnostics（tenant mapping/run context）产出模式，可迁移复用。

### Established Patterns
- 双栈并行：web 使用 Playwright，desktop 使用 WDIO；迁移需保持可并行运行而非大爆炸替换。
- 证据优先：CI artifact 已按 job 维度上传并设置 `retention-days: 7`。
- 认证测试依赖 mock OAuth callback，而非真实第三方登录。
- 租户隔离依赖固定身份（`tenant_a_user`/`tenant_b_user`）与可重复 reset 流程。

### Integration Points
- Tauri 层：`Cargo.toml`、`src/lib.rs`、`capabilities/default.json`（插件与权限接入）。
- E2E 新套件层：新增独立目录（建议 `e2e-desktop-playwright/`）及其 config/fixtures/specs。
- CI 层：`.github/workflows/e2e-tests.yml` 新增 tauri-playwright macOS job，并与现有 job 并行。
- 兼容层：复用 `apps/client/web/app/tests/fixtures/*` 与 `e2e-tests/helpers/tenant.mjs` 的既有行为语义。

</code_context>

<specifics>
## Specific Ideas

- 优先按迁移文档执行，但不止停在 bootstrap，要做到 Phase 1 可用深度。
- 新桌面 Playwright 套件必须物理隔离，降低与现有测试栈耦合。
- CI 先加 macOS 观察通道，同时保留现有主通道，逐步收敛。
- 本阶段“跑通 E2E”按仓库全量口径验收，不做迁移范围内的局部放行。

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 14-d-dev-rust-templates-tauri-sveltekit-axum-moon-template-docs*
*Context gathered: 2026-04-07*
