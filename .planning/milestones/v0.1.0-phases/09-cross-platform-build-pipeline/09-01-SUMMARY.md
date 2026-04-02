---
phase: 09-cross-platform-build-pipeline
plan: 01
subsystem: infra
tags: [tauri, bundle, nsis, webview2, macos, entitlements, ci]
requires:
  - phase: 08-desktop-native-features
    provides: Tauri 桌面能力基础配置与运行时假设
provides:
  - Tauri 跨平台 bundle 基线配置（Windows NSIS + WebView2 bootstrapper + macOS entitlements）
  - 可解析的 Entitlements.plist 基础权限文件
affects: [09-02, phase-09-verification, BUILD-02]
tech-stack:
  added: []
  patterns: [单一 tauri.conf.json 覆盖三平台打包前置条件]
key-files:
  created: [apps/desktop-ui/src-tauri/Entitlements.plist]
  modified: [apps/desktop-ui/src-tauri/tauri.conf.json]
key-decisions:
  - "保持 bundle.targets=all，不改为显式平台列表"
  - "Windows 采用 NSIS currentUser + downloadBootstrapper，避免离线 runtime 打包扩展范围"
patterns-established:
  - "平台特定配置收敛在 bundle.windows / bundle.macOS，避免散落在脚本中"
requirements-completed: [BUILD-02]
duration: 12min
completed: 2026-03-30
---

# Phase 9 Plan 1: Cross-Platform Bundle Baseline Summary

**为 Tauri 打包补齐可执行的 Windows NSIS 与 macOS entitlements 基线配置，并保持单一配置文件跨平台复用。**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-30T01:30:00Z
- **Completed:** 2026-03-30T01:42:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- 在 `tauri.conf.json` 中新增 `bundle.windows.nsis.installMode=currentUser` 与 `displayLanguageSelector=false`
- 显式配置 `bundle.windows.webviewInstallMode.type=downloadBootstrapper`
- 新增 `bundle.macOS.hardenedRuntime=true` 与 `bundle.macOS.entitlements=./Entitlements.plist`
- 创建 `Entitlements.plist`，写入网络与 JIT 相关基础 entitlement

## Task Commits

Each task was committed atomically:

1. **Task 1: 配置 Windows NSIS + WebView2 策略与 macOS entitlements** - `95da9f9` (feat)

## Files Created/Modified
- `apps/desktop-ui/src-tauri/tauri.conf.json` - 增加 Windows/macOS 打包关键字段，保留 `targets: all`
- `apps/desktop-ui/src-tauri/Entitlements.plist` - 提供 macOS hardened runtime 基础 entitlements

## Decisions Made
- 保持 `bundle.targets: "all"`，符合计划 D-10，避免平台列表漂移。
- 仅实现 boilerplate 级别 entitlements，不接入 notarization / release 自动化，严格遵守 Phase 9 边界。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] 清理误生成的 `nul` 文件**
- **Found during:** Task 1 提交前检查
- **Issue:** 在 Windows shell 重定向时产生了仓库根目录 `nul` 文件，若不清理会污染提交
- **Fix:** 删除误生成文件并重新检查工作区
- **Files modified:** `nul`（删除）
- **Verification:** `git status --short` 不再出现该文件
- **Committed in:** `95da9f9`（同任务提交）

---

**Total deviations:** 1 auto-fixed（Rule 3: 1）
**Impact on plan:** 仅清理阻塞性噪音文件，无范围扩张。

## Issues Encountered
- 外部文档拉取受网络证书/404影响，未阻塞实现；本次按计划给定字段与仓库上下文完成配置落地与脚本校验。

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 09-02 可直接基于该配置接入 CI 三平台构建验证。
- 当前未引入发布与签名自动化，符合本阶段边界。

## Self-Check: PASSED

- FOUND: `.planning/phases/09-cross-platform-build-pipeline/09-01-SUMMARY.md`
- FOUND: commit `95da9f9`
