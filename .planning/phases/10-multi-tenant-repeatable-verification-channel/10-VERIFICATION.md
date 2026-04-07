---
phase: 10-multi-tenant-repeatable-verification-channel
verified: 2026-04-06T14:09:04Z
status: human_needed
score: 8/9 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 5/6
  gaps_closed:
    - "Counter mutations in tenant-1 do not alter tenant-2 values, and isolation remains true across repeated runs"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "在 GitHub Actions 触发 .github/workflows/e2e-tests.yml 并下载 web/desktop artifacts"
    expected: "web/desktop 都产出 job-scoped 证据包，保留 7 天，包含 tenant mapping 与 Playwright/WDIO 失败诊断文件"
    why_human: "需要真实 CI 运行和 GitHub artifact UI 下载验证，静态代码扫描无法证明“可检索且可用”"
---

# Phase 10: 多租户可重复验证通道 Verification Report

**Phase Goal:** 多租户可重复验证通道（MTEN-01, MTEN-02, MTEN-03）
**Verified:** 2026-04-06T14:09:04Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Tester can switch between at least two tenants in a repeatable harness without manual environment patching | ✓ VERIFIED | `apps/client/web/app/tests/fixtures/tenant.ts` 固定 `TENANT_A/TENANT_B`，并提供 `initTenantPair/resetTenantPairCounter`；Web E2E 引入并复用该 harness。 |
| 2 | Tenant mapping is fixed and repeatable across runs | ✓ VERIFIED | Web: `tenant_a_user/tenant_b_user`；Desktop: `e2e-tests/helpers/tenant.mjs` 使用同一 userSub，`tenant-isolation.e2e.mjs` 明确断言与 Web 一致。 |
| 3 | Tenant init failure stops the test immediately instead of falling back silently | ✓ VERIFIED | `tenant.ts` 与 `e2e-tests/helpers/tenant.mjs` 均在 init/reset/read 失败时 `throw`，且错误包含 tenant label。 |
| 4 | Counter mutations in tenant-1 do not alter tenant-2 values | ✓ VERIFIED | `counter_service.rs` 读写改为 `tenant_id` 维度（`*_for_tenant` + `WHERE tenant_id = ?`）；`http_e2e_test.rs` 的 `counter_mutation_isolated_between_two_tenants` 断言 A 改变不影响 B。 |
| 5 | Isolation assertions remain true across repeated runs | ✓ VERIFIED | `http_e2e_test.rs` 的 `counter_isolation_repeated_run_stays_stable` 覆盖 run-1/run-2；Web/Desktop tenant isolation spec 也分别含 run-1/run-2 同 seed 断言。 |
| 6 | Web and desktop suites both exercise the same fixed tenant pair | ✓ VERIFIED | Web fixture: `tenant_a_user`,`tenant_b_user`；Desktop helper/suite 使用同一 pair，并有一致性断言。 |
| 7 | API counter handlers consume middleware tenant identity | ✓ VERIFIED | `servers/api/src/routes/counter.rs` 通过 `Option<Extension<TenantId>>` 提取租户，缺失则 401；`servers/api/src/lib.rs` 对 `api_router` 应用 `tenant_middleware`。 |
| 8 | Maintainer can run automated multi-tenant tests in CI and retrieve artifacts sufficient to diagnose failures | ? UNCERTAIN | Workflow 与证据路径/命名/保留策略均已配置，但“可检索且可诊断”需要真实 GitHub Actions run+下载验证。 |
| 9 | Artifacts are job-scoped, retained 7 days, and cover Playwright + WDIO tenant evidence | ✓ VERIFIED | `.github/workflows/e2e-tests.yml` 中 web/desktop artifact 名含 `${github.job}-${matrix.os}-${run_id}-${run_attempt}`，`retention-days: 7`，并包含 `playwright-report/test-results`、`junit/wdio-logs/diagnostics/tenant-mapping`。 |

**Score:** 8/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `apps/client/web/app/tests/fixtures/tenant.ts` | Fixed tenant mapping and reset helper | ✓ VERIFIED | 存在且为实质实现，提供固定租户、init/reset helper，并被测试调用。 |
| `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` | Repeatable Web tenant isolation regression | ✓ VERIFIED | 存在，beforeEach reset + run-1/run-2 isolation 断言。 |
| `apps/client/web/app/tests/e2e/counter.test.ts` | Counter flow reuse of tenant harness | ✓ VERIFIED | `beforeEach` 调用 `resetTenantPairCounter`。 |
| `e2e-tests/helpers/tenant.mjs` | Desktop tenant reset/login/counter helpers | ✓ VERIFIED | 存在，封装 init/reset/read/increment，失败即抛错。 |
| `e2e-tests/specs/tenant-isolation.e2e.mjs` | WDIO desktop isolation regression | ✓ VERIFIED | 存在，覆盖 tenant-1/tenant-2 + run-1/run-2。 |
| `.github/workflows/e2e-tests.yml` | Job-scoped uploads + 7-day retention | ✓ VERIFIED | web/desktop 均 `if: always()` 上传，命名 job-scoped，保留 7 天。 |
| `apps/client/web/app/playwright.config.ts` | Web failure evidence settings | ✓ VERIFIED | `trace: on-first-retry`、`screenshot/video` failure policy、`outputDir: test-results`。 |
| `e2e-tests/wdio.conf.mjs` | WDIO JUnit and logs output | ✓ VERIFIED | JUnit reporter 开启，`outputDir: ./test-results/wdio-logs`。 |
| `packages/core/usecases/src/counter_service.rs` | Tenant-scoped counter read/write/reset | ✓ VERIFIED | 新增 `*_for_tenant` 方法，migration 主键 `tenant_id`，含双租户隔离单测。 |
| `servers/api/src/routes/counter.rs` | Counter endpoints wired to TenantId | ✓ VERIFIED | 路由使用 tenant extension 并调用 tenant-aware service methods。 |
| `servers/api/tests/http_e2e_test.rs` | HTTP cross-tenant regression tests | ✓ VERIFIED | 含 401 缺租户、隔离、重复运行稳定性测试。 |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `apps/client/web/app/tests/fixtures/tenant.ts` | `servers/api/src/routes/tenant.rs` | `POST /api/tenant/init` | ✓ WIRED | gsd key-link 通过，fixture 中存在 `api/tenant/init` 调用。 |
| `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` | `apps/client/web/app/tests/fixtures/auth.ts` | `triggerMockOAuth` | ✓ WIRED | OAuth mock bootstrap 调用存在。 |
| `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` | `apps/client/web/app/tests/fixtures/tenant.ts` | `initTenantPair/resetTenantPairCounter` | ✓ WIRED | import+调用齐全。 |
| `e2e-tests/specs/tenant-isolation.e2e.mjs` | `e2e-tests/helpers/tenant.mjs` | `resetTenantPair` | ✓ WIRED | import+调用齐全。 |
| `.github/workflows/e2e-tests.yml` | `apps/client/web/app/playwright.config.ts` | `upload-artifact` for Playwright evidence | ✓ WIRED | workflow 上传 playwright-report/results/test-results。 |
| `.github/workflows/e2e-tests.yml` | `e2e-tests/wdio.conf.mjs` | `wdio-junit` + desktop evidence | ✓ WIRED | workflow 上传 junit/diagnostics/wdio-logs。 |
| `servers/api/src/middleware/tenant.rs` | `servers/api/src/routes/counter.rs` | request tenant extension extraction | ✓ WIRED | 虽 `verify key-links` 的 regex 误报未命中，但 `lib.rs` 已 route_layer 注入 middleware，`counter.rs` 读取 `Extension<TenantId>`。 |
| `servers/api/src/routes/counter.rs` | `packages/core/usecases/src/counter_service.rs` | tenant-aware counter service calls | ✓ WIRED | `increment_for_tenant/decrement_for_tenant/reset_for_tenant/get_value_for_tenant` 均被调用。 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `apps/client/web/app/tests/e2e/tenant-isolation.test.ts` | `tenantAAfter`, `tenantBAfter` | `/api/counter/*` UI flow | Yes — API route 使用 `TenantId` + tenant-aware service | ✓ FLOWING |
| `e2e-tests/helpers/tenant.mjs` | `body.value` | `/api/counter/value` | Yes — token→middleware→tenant_id→libsql 查询链路闭合 | ✓ FLOWING |
| `servers/api/src/routes/counter.rs` | `tenant_id` | `Extension<TenantId>` from middleware | Yes — 缺失 tenant 直接 401，存在时进入 tenant-aware methods | ✓ FLOWING |
| `packages/core/usecases/src/counter_service.rs` | counter row by tenant | libsql `counter` table | Yes — `tenant_id` 主键+按 tenant 查询/更新，无全局 `id=1` 路径 | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Desktop tenant helper exports required APIs | `node -e "import('./e2e-tests/helpers/tenant.mjs')..."` | `function function function` | ✓ PASS |
| Counter route wired to tenant-aware methods | `node -e "read counter.rs and assert Extension<TenantId> + *_for_tenant"` | `ok` | ✓ PASS |
| CI artifact naming + 7-day retention present | `node -e "read e2e-tests.yml and assert evidence names + retention"` | `ok` | ✓ PASS |
| Real CI artifact retrievability | (需要触发 GitHub Actions) | 未在本地执行 | ? SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| MTEN-01 | `10-01-PLAN.md` | Tester can switch between at least two tenants in a repeatable test harness | ✓ SATISFIED | 固定双租户 fixture + 可复用 init/reset helper 已落地。 |
| MTEN-02 | `10-02-PLAN.md`, `10-04-PLAN.md` | Tester can verify counter values are tenant-scoped | ✓ SATISFIED | 后端 tenant-scoped 数据流已闭合（service+route+HTTP 回归）；Web/Desktop isolation spec 已对齐。 |
| MTEN-03 | `10-03-PLAN.md` | Maintainer can run automated multi-tenant tests in CI and collect artifacts for diagnosis | ? NEEDS HUMAN | Workflow 配置齐备，但需真实 CI run 下载 artifacts 才能最终证明“可诊断可检索”。 |

**Orphaned requirements check (Phase 10):** None. `REQUIREMENTS.md` 中 Phase 10 相关 ID 为 MTEN-01/02/03，且均已在 phase 10 计划 frontmatter 声明并纳入本次核对。

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| — | — | No blocking TODO/stub/hardcoded-empty anti-patterns in Phase 10 key files | ℹ️ Info | 未发现阻断 Phase 10 目标达成的反模式 |

### Human Verification Required

### 1. CI Artifact Usability Check

**Test:** 在 GitHub Actions 手动触发 `.github/workflows/e2e-tests.yml`（建议包含至少一个 tenant isolation 失败样例），并下载 web + desktop 两类证据包。  
**Expected:** artifact 名称带 job 维度标识（含 job/os/run_id/run_attempt），`retention-days = 7`，内容可直接用于诊断（Playwright 报告与失败证据、WDIO JUnit/日志、tenant mapping 诊断文件）。  
**Why human:** 需要真实 CI 执行与 GitHub UI 下载检查，无法通过静态代码扫描完全替代。

### Gaps Summary

本次复验已闭合上次唯一阻断 gap（MTEN-02 后端共享状态问题）：counter 数据流已切换为 tenant-scoped，API 路由强制租户上下文，HTTP 回归已验证隔离与重复运行稳定性。当前无自动化可判定的阻断缺口；仅剩 MTEN-03 的“CI 产物可检索/可诊断”需人工在 GitHub Actions 实际运行中确认，因此总体状态为 `human_needed`。

---

_Verified: 2026-04-06T14:09:04Z_  
_Verifier: the agent (gsd-verifier)_
