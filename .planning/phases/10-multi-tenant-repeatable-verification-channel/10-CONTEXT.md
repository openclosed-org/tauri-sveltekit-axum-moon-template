# Phase 10: 多租户可重复验证通道 - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

<domain>
## Phase Boundary

建立双租户可复现验证通道，使测试者可在无需手工环境补丁的前提下稳定切换至少两个租户并验证隔离行为，同时让维护者在 CI 中运行自动化多租户测试并拿到可诊断失败证据。

本阶段不新增业务能力，仅锁定多租户验证与证据输出的实现口径。

</domain>

<decisions>
## Implementation Decisions

### 执行矩阵与门禁范围
- **D-01:** 采用 Web E2E 与 Desktop E2E 双线并行推进，而不是单线先行。
- **D-02:** 并行方案的最小门禁集合锁定为 `Web + WDIO 最小集`：Playwright 仅要求 `desktop-chrome` 的双租户场景，WDIO 保持 desktop 路径并纳入双租户隔离断言。

### 租户身份建模与初始化失败策略
- **D-03:** 测试身份采用固定双租户映射（稳定 `user_sub`/tenant 绑定），不使用动态随机租户映射。
- **D-04:** 租户初始化失败采用 fail-fast：任一租户 init 失败即终止当前测试/作业，不降级为单租户继续。

### 隔离断言口径与重复运行策略
- **D-05:** MTEN-02 验收以行为断言为主：验证 tenant-A 写入后 tenant-B 不受影响，覆盖写入-读取链路，并要求重复运行仍成立。
- **D-06:** Phase 10 不强制绑定底层存储直连断言，优先锁定 API/UI 行为层可重复性与可诊断性。
- **D-07:** 为保证 repeated runs 一致性，每个用例前显式重置 tenant-A/tenant-B 到已知 counter 初值，再执行断言。

### CI 诊断证据输出
- **D-08:** 失败时上传最小诊断包：Playwright trace/video/screenshot、WDIO JUnit、关键运行日志、租户映射文件。
- **D-09:** Artifact 按 job 维度独立上传，不按 PR 聚合。
- **D-10:** Artifact 保留期锁定为 7 天。

### the agent's Discretion
- 固定双租户映射的具体命名（例如 `tenant_a_user` / `tenant_b_user`）与存放位置。
- 重置机制落点（测试前置脚本、fixture 封装或 helper 层）及错误文案。
- 最小诊断包中文件目录结构与命名规范（在不改变 D-08~D-10 前提下）。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 阶段范围与验收
- `.planning/ROADMAP.md` — Phase 10 目标、依赖与 Success Criteria。
- `.planning/REQUIREMENTS.md` — MTEN-01, MTEN-02, MTEN-03 的正式要求。
- `.planning/PROJECT.md` — v0.2.1 里程碑约束（沿用既有测试栈、跨平台质量目标）。
- `.planning/phases/09-functional-correctness-baseline-fix/09-CONTEXT.md` — Phase 9 已锁定的测试与行为基线。

### 现有多租户与隔离实现
- `servers/api/src/middleware/tenant.rs` — JWT 提取 `TenantId` 的中间件入口。
- `servers/api/src/routes/tenant.rs` — `/api/tenant/init` 的租户初始化行为与响应结构。
- `packages/adapters/storage/surrealdb/src/lib.rs` — tenant filter 注入与 tenant-aware 查询约束。
- `servers/api/tests/integration_test.rs` — tenant filter 注入与租户相关集成断言样例。

### 现有计数器与测试入口
- `servers/api/src/routes/counter.rs` — counter HTTP 路由及返回值协议。
- `packages/core/usecases/src/counter_service.rs` — counter 读写服务实现。
- `packages/adapters/hosts/tauri/src/commands/counter.rs` — desktop IPC counter 命令链路。
- `apps/client/web/app/src/routes/(app)/counter/+page.svelte` — 前端 counter 调用与可见错误反馈。

### 现有 E2E 与 CI 证据通道
- `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` — 当前 tenant isolation E2E 基线。
- `apps/client/web/app/tests/fixtures/auth.ts` — mock OAuth 触发 fixture（双租户身份可复用入口）。
- `apps/client/web/app/playwright.config.ts` — Playwright 项目矩阵、reporter、trace/video/screenshot 配置。
- `e2e-tests/wdio.conf.mjs` — WDIO desktop 配置与 JUnit 输出位置。
- `e2e-tests/scripts/run-desktop-e2e.mjs` — desktop E2E 运行入口与 CI 模式控制。
- `.github/workflows/e2e-tests.yml` — 现有 Web/WDIO 任务与 artifact 上传基线。

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `apps/client/web/app/tests/fixtures/auth.ts`: 已有 `triggerMockOAuth`，可直接承接固定双租户身份注入。
- `apps/client/web/app/tests/e2e/tenant-isolation.test.ts`: 已有多上下文隔离测试骨架，可扩展为显式 tenant-A/B 写读与重置流程。
- `.github/workflows/e2e-tests.yml`: 已有 WDIO JUnit 与 Playwright report 上传，适合作为最小诊断包扩展基础。
- `apps/client/web/app/playwright.config.ts`: 已启用 `trace`/`video`/`screenshot`，满足 D-08 证据项的一部分。

### Established Patterns
- 双测试栈并存：Web 场景用 Playwright，desktop 场景用 WDIO + tauri-driver。
- CI artifact 以 job 维度上传，失败后 `if: always()` 保证证据可取。
- 认证测试依赖 mock OAuth callback 事件，避免真实外部 OAuth 依赖。
- 租户隔离在后端依赖 `TenantId` middleware + tenant-aware query 注入。

### Integration Points
- Playwright: `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` 与 `tests/fixtures/auth.ts`。
- WDIO: `e2e-tests/specs/*.e2e.mjs` 与 `e2e-tests/wdio.conf.mjs`。
- CI: `.github/workflows/e2e-tests.yml` 的 `web-e2e` / `desktop-e2e` jobs 与 artifact step。
- API/Storage: `servers/api/src/routes/tenant.rs`, `servers/api/src/middleware/tenant.rs`, `packages/adapters/storage/surrealdb/src/lib.rs`。

</code_context>

<specifics>
## Specific Ideas

- 双线并行但先锁最小可行门禁集（Web `desktop-chrome` + WDIO desktop），避免过早扩大矩阵。
- 固定双租户映射并 fail-fast，优先追求可重复和快速定位。
- 失败证据以“最小诊断包”原则组织，按 job 上传且只保留 7 天。

</specifics>

<deferred>
## Deferred Ideas

- 增加存储层直连强断言（行为层之外的 DB-level 验证）——可作为后续强化项，不在 Phase 10 必达范围内。
- 扩展到更大浏览器/设备矩阵或 PR 聚合长保留 artifact 策略——可在 Phase 11/13 结合门禁成熟度再讨论。

</deferred>

---

*Phase: 10-multi-tenant-repeatable-verification-channel*
*Context gathered: 2026-04-06*
