---
phase: 10-multi-tenant-repeatable-verification-channel
plan: 03
subsystem: testing
tags: [ci, github-actions, playwright, wdio, multi-tenant, artifacts]

# Dependency graph
requires:
  - phase: 10-multi-tenant-repeatable-verification-channel
    provides: MTEN-01/MTEN-02 fixed tenant mapping and web/desktop isolation regression tests
provides:
  - Web job-scoped Playwright evidence artifact bundle with 7-day retention
  - Desktop job-scoped evidence artifact bundle (JUnit + WDIO logs + tenant mapping diagnostics)
  - Failure-time evidence upload parity for web and desktop tenant isolation runs
affects: [phase-11, release-gate-evidence, multi-tenant-ci-diagnostics]

# Tech tracking
tech-stack:
  added: []
  patterns: [job-scoped artifact naming with run identity, always-upload CI diagnostics for failing E2E jobs]

key-files:
  created: []
  modified:
    - .github/workflows/e2e-tests.yml
    - apps/client/web/app/playwright.config.ts
    - e2e-tests/wdio.conf.mjs
    - e2e-tests/scripts/run-desktop-e2e.mjs

key-decisions:
  - "Web 和 Desktop 分别维持各自 artifact bundle，但都采用 job-scoped naming + run identity，避免 PR 级混淆。"
  - "Desktop 证据通过执行脚本生成 tenant mapping 与 run context JSON，确保失败后无需本地复现即可诊断。"

patterns-established:
  - "CI Evidence Bundle Pattern: upload-artifact 使用 ${github.job}/${matrix.os}/${run_id}/${run_attempt} 组成 job 级证据名称。"
  - "Desktop Diagnostic Seed Pattern: 在 WDIO 启动前生成 tenant-mapping.json 与 run-context.json 并打包上传。"

requirements-completed: [MTEN-03]

# Metrics
duration: 17min
completed: 2026-04-06
---

# Phase 10 Plan 03: CI 最小诊断包与 job 级 artifact 输出 Summary

**CI 现在会按 job 独立上传可下载的最小多租户诊断包：Web 侧包含 Playwright 报告与 tenant mapping 证据，Desktop 侧包含 WDIO JUnit、日志与 tenant mapping 上下文，并统一保留 7 天。**

## Performance

- **Duration:** 17 min
- **Started:** 2026-04-06T20:06:00+08:00
- **Completed:** 2026-04-06T20:23:00+08:00
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- 扩展 Web E2E artifact 上传：采用 job-scoped artifact 名称，打包 Playwright report、results、test-results 与 tenant fixture 证据，并设置 `retention-days: 7`。
- 保持 Playwright 失败证据策略（`trace/video/screenshot` failure-first）不变，同时显式设置 `outputDir: test-results` 便于 CI 归档。
- 保持 WDIO JUnit 输出能力，并新增 Desktop 证据打包：包括 JUnit XML、WDIO logs、tenant mapping 与 run context 诊断文件，且在 `if: always()` 下失败也上传。

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand Web E2E artifact output for tenant diagnostics** - `fded8c3` (feat)
2. **Task 2: Add WDIO evidence uploads for desktop tenant failures** - `543eacd` (feat)

## Files Created/Modified
- `.github/workflows/e2e-tests.yml` - 增加 web/desktop 证据上传策略（job-scoped naming、always upload、7-day retention）并更新 desktop summary 下载链接。
- `apps/client/web/app/playwright.config.ts` - 增加 `outputDir: 'test-results'`，确保 trace/video/screenshot 产物目录稳定可归档。
- `e2e-tests/wdio.conf.mjs` - 增加 WDIO `outputDir` 指向 `test-results/wdio-logs`，补齐 desktop 运行日志证据。
- `e2e-tests/scripts/run-desktop-e2e.mjs` - 在执行前生成 `tenant-mapping.json` 与 `run-context.json` 诊断文件。

## Decisions Made
- 维持 Web 和 Desktop 各自 artifact 步骤，避免将 Desktop 证据语义混入 Web 上传步骤，减少排障歧义。
- 证据最小集只包含诊断所需文件（报告、日志、mapping、上下文），不引入额外敏感信息或超量产物。

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- 本地 `rg` 命令受 `RIPGREP_CONFIG_PATH` 缺失告警影响（`C:\Users\唐斌\.ripgreprc` 不存在），但不影响 grep 结果输出与本计划验收检查。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- MTEN-03 已满足：CI 中 web/desktop 都能在失败后保留可下载诊断证据，并具备 job 维度可追溯命名。
- 可进入 Phase 11 将 Windows desktop E2E 与证据通道绑定到 required check 门禁。

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: `.planning/phases/10-multi-tenant-repeatable-verification-channel/10-03-SUMMARY.md`
- FOUND commits: `fded8c3`, `543eacd`
