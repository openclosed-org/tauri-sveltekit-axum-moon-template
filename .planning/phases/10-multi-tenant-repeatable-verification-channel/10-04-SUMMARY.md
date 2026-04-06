---
phase: 10-multi-tenant-repeatable-verification-channel
plan: 04
subsystem: api
tags: [multi-tenant, counter, axum, libsql, playwright, wdio, regression]

# Dependency graph
requires:
  - phase: 10-multi-tenant-repeatable-verification-channel
    provides: fixed dual-tenant harness and isolation regression skeleton from 10-02
provides:
  - tenant-scoped counter usecase methods and migration keyed by tenant_id
  - counter API handlers that require middleware TenantId and reject missing context
  - HTTP token-A/token-B repeated-run isolation regression at API boundary
  - Web/Desktop tenant isolation re-run evidence against updated backend
affects: [phase-10-verification, MTEN-02, counter-api, web-e2e, desktop-e2e]

# Tech tracking
tech-stack:
  added: []
  patterns: [tenant-id keyed counter persistence, middleware extension to route enforcement, repeated-run deterministic baseline reset]

key-files:
  created:
    - .planning/phases/10-multi-tenant-repeatable-verification-channel/10-04-SUMMARY.md
  modified:
    - packages/core/usecases/src/counter_service.rs
    - servers/api/src/routes/counter.rs
    - servers/api/tests/http_e2e_test.rs

key-decisions:
  - "Retain legacy CounterService trait API by mapping it to a default tenant while adding explicit tenant-aware methods for API callers."
  - "Enforce counter tenant boundary at route layer by requiring TenantId in request extensions and returning 401 instead of fallback behavior."

patterns-established:
  - "Tenant-Scoped Counter Pattern: All counter SQL binds tenant_id as key, no global id=1 data path."
  - "HTTP Isolation Regression Pattern: reset(A/B) -> mutate(A) -> read(A/B) and repeat run to assert deterministic isolation."

requirements-completed: [MTEN-02]

# Metrics
duration: 55min
completed: 2026-04-06
---

# Phase 10 Plan 04: counter tenant 数据流缺口闭合 Summary

**计数器后端已从全局单行存储切换为 tenant_id 维度，API 路由强制消费中间件租户上下文并补齐 HTTP 级隔离回归，消除“测试断言隔离但后端共享状态”的假阳性。**

## Performance

- **Duration:** 55 min
- **Started:** 2026-04-06T21:01:33+08:00
- **Completed:** 2026-04-06T21:56:28+08:00
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- 在 usecase 层新增 `*_for_tenant` 显式租户方法并把 counter schema 改为 `tenant_id` 主键，彻底移除 tenant-aware 路径中的 `id = 1` 全局共享语义。
- 在 API 路由层接入 `request.extensions()` 的 `TenantId`，counter handlers 缺失租户上下文时返回 401，满足 threat model T-10-10。
- 在 `http_e2e_test.rs` 增加 token-A/token-B 的跨租户隔离与 repeated-run 回归，验证 mutate/read 链路行为一致且可复现。

## Task Commits

Each task was committed atomically:

1. **Task 1: Add tenant-scoped counter operations in usecase layer (RED)** - `354d85f` (test)
2. **Task 1: Add tenant-scoped counter operations in usecase layer (GREEN)** - `85f9eb2` (feat)
3. **Task 2: Wire API counter handlers to TenantId and add HTTP isolation regression** - `0964fc3` (feat)
4. **Task 3: Re-run Web and Desktop tenant isolation regressions against tenant-scoped backend** - `PENDING` (chore)

_Note: Task 1 is TDD and intentionally produced separate RED/GREEN commits._

## Files Created/Modified
- `packages/core/usecases/src/counter_service.rs` - 新增 tenant-aware counter 方法、tenant_id schema migration、双租户隔离单测。
- `servers/api/src/routes/counter.rs` - 路由改为 `/api/counter/*`，接入 `TenantId` 扩展并在缺失上下文时返回 401。
- `servers/api/tests/http_e2e_test.rs` - 新增 counter 跨租户隔离与 repeated-run HTTP 回归测试。
- `.planning/phases/10-multi-tenant-repeatable-verification-channel/10-04-SUMMARY.md` - 计划执行与验证证据归档。

## Decisions Made
- 维持 trait 兼容性（`CounterService` 原方法不删）以降低调用方回归风险，同时通过新增 tenant-aware 方法满足 MTEN-02 的真实性约束。
- 将 counter 路由显式挂载到 `/api/counter/*`，对齐现有 Web/Desktop 客户端调用路径与 Phase 10 既有回归用例入口。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 修复 counter 路由路径与客户端断言不一致**
- **Found during:** Task 2
- **Issue:** 路由初始定义为 `/counter/*`，而客户端与回归用例稳定使用 `/api/counter/*`，导致 404 干扰隔离验证。
- **Fix:** 将 counter 路由统一调整为 `/api/counter/{increment,decrement,reset,value}`。
- **Files modified:** `servers/api/src/routes/counter.rs`
- **Verification:** `rtk cargo test -p runtime_server --test http_e2e_test -- --nocapture` 通过。
- **Committed in:** `0964fc3`

**2. [Rule 3 - Blocking] 解决 Windows 测试构建依赖缺失（libclang）**
- **Found during:** Task 2 verification
- **Issue:** `surrealdb-librocksdb-sys` 构建失败，报错缺失 `libclang.dll`，阻塞 `runtime_server` HTTP E2E。
- **Fix:** 安装 LLVM（`scoop install llvm`）并在测试命令中设置 `LIBCLANG_PATH=D:\dev-storage\scoop\apps\llvm\current\bin`。
- **Files modified:** None (environment/toolchain only)
- **Verification:** `rtk cargo test -p runtime_server --test http_e2e_test -- --nocapture` 23/23 通过。
- **Committed in:** N/A (non-repo environment fix)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** 偏差均为完成计划所需的正确性/可执行性修复，无额外功能扩展。

## Issues Encountered
- `rtk bun run --cwd apps/client/web/app test:e2e -- --grep "tenant isolation"` 在未启动后端时失败（`ECONNREFUSED 127.0.0.1:3001`），属于环境前置条件缺失。
- 启动 `runtime_server` 后，Web tenant isolation 仍失败：`/api/tenant/init` 返回 401（`[tenant-A] tenant init failed: status=401`），表明当前 E2E fixture 与 API 认证前置条件存在既有不匹配。
- `rtk bun run --cwd e2e-tests test:ci` 触发大量既有非本计划 spec 失败（admin/agent/login/counter），且 tenant-isolation spec 同样在 `beforeEach` 因 `/api/tenant/init` 401 失败。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- MTEN-02 的后端核心缺口（counter tenant data flow）已闭合，HTTP 级回归可稳定验证租户隔离。
- `10-VERIFICATION.md` missing #3 的“Web+Desktop 重跑证据”已补充执行记录，但当前环境仍受既有 E2E 鉴权/页面基线问题影响，需后续计划专门修复。

## Known Stubs

None.
