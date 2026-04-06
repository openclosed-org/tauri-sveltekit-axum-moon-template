# ROADMAP: Tauri-SvelteKit-Axum Boilerplate

**Last updated:** 2026-04-06

## Milestones

- ✅ **v0.2.0 架构蓝图对齐与核心功能实现** — Phases 1-8 (shipped 2026-04-04) — [archive](milestones/v0.2.0-ROADMAP.md)
- 📋 **v0.2.1 跨平台测试与缺陷闭环强化** — Phases 9-13 (planning)

## Phases

<details>
<summary>✅ v0.2.0 架构蓝图对齐与核心功能实现 (Phases 1-8) — SHIPPED 2026-04-04</summary>

- [x] Phase 1: 仓库目录结构与工具链对齐 (4/4 plans) — completed 2026-04-01
- [x] Phase 2: Contracts/typegen 单一真理源 (2/2 plans) — completed 2026-04-01
- [x] Phase 3: Runtime 边界收敛 (4/4 plans) — completed 2026-04-02
- [x] Phase 4: 最小功能实现 (6/6 plans) — completed 2026-04-02
- [x] Phase 5: Agent-Friendly 开发基建 (2/2 plans) — completed 2026-04-02
- [x] Phase 7: 前端消费 Generated Types (3/3 plans) — completed 2026-04-03
- [x] Phase 8: Agent 双路径 + Prompts + Phase 5 验证 (3/3 plans) — completed 2026-04-03

**Known Gaps:**
- AUTH-01: GoogleAuthAdapter not fully wired into Tauri commands (Phase 6 empty, tech debt)

</details>

- [x] **Phase 9: 功能正确性基线修复** - 修复认证/计数器/Agent 关键交互缺陷，建立可回归的用户行为基线（3/3 plans, completed 2026-04-06）
- [x] **Phase 10: 多租户可重复验证通道** - 建立双租户可复现测试与 CI 诊断证据输出（3 plans） (completed 2026-04-06)
- [ ] **Phase 11: Windows 桌面 E2E 门禁固化** - 将稳定、可重复、可审计的 Windows desktop E2E 设为合并硬门禁
- [ ] **Phase 12: 缺陷生命周期与回归强制闭环** - 固化严重级、状态、责任人与 P0/P1 修复回归义务
- [ ] **Phase 13: 跨平台放行证据与质量摘要** - 形成 Windows+macOS 同构建证据放行与发布质量总览

## Phase Details

### Phase 9: 功能正确性基线修复
**Goal**: 用户在关键路径上可稳定完成登出、计数器变更与 Agent 新会话操作，为后续门禁测试提供真实基线。
**Depends on**: Phase 8
**Requirements**: AUTH-02, AUTH-03, COUNTER-02, AGENT-02, AGENT-03, AGENT-04
**Success Criteria** (what must be TRUE):
  1. Signed-in user can click a visible Google logout action and is signed out successfully.
  2. After logout, user is returned to unauthenticated state and previous session credentials are not reused in desktop and browser flows.
  3. User can increment and decrement counter, and value changes are consistent between displayed value and persisted state after reload.
  4. User can start a new chat thread with New Chat without losing saved API key/base URL/model settings.
  5. User can trigger connectivity test and receive actionable pass/fail feedback for API key, base URL, and model reachability.
**Plans**: 3 plans
Plans:
- [x] 09-01-PLAN.md — Settings logout + connection diagnostics baseline
- [x] 09-02-PLAN.md — Counter correctness and persistence regression baseline
- [x] 09-03-PLAN.md — Agent New Chat reset semantics with settings retention
**UI hint**: yes

### Phase 10: 多租户可重复验证通道
**Goal**: 测试者和维护者可以稳定验证租户隔离行为，并在 CI 中获得可诊断证据。
**Depends on**: Phase 9
**Requirements**: MTEN-01, MTEN-02, MTEN-03
**Success Criteria** (what must be TRUE):
  1. Tester can switch between at least two tenants in a repeatable harness without manual environment patching.
  2. Counter mutations in tenant-1 do not alter tenant-2 values, and isolation remains true across repeated runs.
  3. Maintainer can run automated multi-tenant tests in CI and retrieve artifacts sufficient to diagnose failures.
**Plans**: 3 plans
Plans:
- [x] 10-01-PLAN.md — 固定双租户映射与可重复初始化 harness
- [x] 10-02-PLAN.md — tenant-1/tenant-2 隔离断言与重复运行回归
- [x] 10-03-PLAN.md — CI 最小诊断包与 job 级 artifact 输出

### Phase 11: Windows 桌面 E2E 门禁固化
**Goal**: 受保护分支的合并决策可由 Windows desktop E2E required check 直接约束。
**Depends on**: Phase 10
**Requirements**: QGATE-01
**Success Criteria** (what must be TRUE):
  1. Maintainer cannot merge to protected branches when Windows desktop E2E required check is failing or missing.
  2. Maintainer can observe deterministic Windows desktop E2E pass/fail outcomes for the same commit under the gate workflow.
  3. Each gate run produces auditable evidence that can be inspected after failures.
**Plans**: TBD

### Phase 12: 缺陷生命周期与回归强制闭环
**Goal**: 缺陷处理从提报到关闭具备统一治理规则，且高严重缺陷修复必须附带回归证据。
**Depends on**: Phase 11
**Requirements**: BUG-01, BUG-02, BUG-03
**Success Criteria** (what must be TRUE):
  1. Maintainer can triage each bug with standard severity (P0-P3), workflow state, and explicit owner.
  2. P0/P1 bug cannot be moved to closed state unless linked regression verification evidence is present.
  3. Flaky tests appear in a dedicated quarantine flow with visible repair SLA and current status.
**Plans**: TBD

### Phase 13: 跨平台放行证据与质量摘要
**Goal**: 维护者可基于同一候选构建的跨平台证据与缺陷态势完成发布放行判断。
**Depends on**: Phase 12
**Requirements**: QGATE-02, QGATE-03
**Success Criteria** (what must be TRUE):
  1. Maintainer can verify release readiness from Windows and macOS QA/UAT evidence tied to the same candidate build.
  2. Maintainer can view a release quality summary with automated test results, UAT sign-offs, and open defects grouped by severity.
  3. Release decision evidence is traceable and reviewable after the release window.
**Plans**: TBD

## Progress Table

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. 仓库目录结构与工具链对齐 | v0.2.0 | 4/4 | Complete | 2026-04-01 |
| 2. Contracts/typegen 单一真理源 | v0.2.0 | 2/2 | Complete | 2026-04-01 |
| 3. Runtime 边界收敛 | v0.2.0 | 4/4 | Complete | 2026-04-02 |
| 4. 最小功能实现 | v0.2.0 | 6/6 | Complete | 2026-04-02 |
| 5. Agent-Friendly 开发基建 | v0.2.0 | 2/2 | Complete | 2026-04-02 |
| 6. 连接 Auth Adapter 到 Tauri 命令 | v0.2.0 | 0/0 | Gap (tech debt) | - |
| 7. 前端消费 Generated Types | v0.2.0 | 3/3 | Complete | 2026-04-03 |
| 8. Agent 双路径 + Prompts + Phase 5 验证 | v0.2.0 | 3/3 | Complete | 2026-04-03 |
| 9. 功能正确性基线修复 | v0.2.1 | 3/3 | Complete | 2026-04-06 |
| 10. 多租户可重复验证通道 | v0.2.1 | 4/4 | Complete    | 2026-04-06 |
| 11. Windows 桌面 E2E 门禁固化 | v0.2.1 | 0/0 | Not started | - |
| 12. 缺陷生命周期与回归强制闭环 | v0.2.1 | 0/0 | Not started | - |
| 13. 跨平台放行证据与质量摘要 | v0.2.1 | 0/0 | Not started | - |

---

*Ready for next phase: `/gsd-discuss-phase 10` or `/gsd-plan-phase 10`*
