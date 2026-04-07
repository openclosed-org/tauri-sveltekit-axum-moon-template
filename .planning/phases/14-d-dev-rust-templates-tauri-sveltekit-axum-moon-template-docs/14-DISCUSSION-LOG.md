# Phase 14: 请问根据D:\dev\rust\templates\tauri-sveltekit-axum-moon-template\docs\TAURI_PLAYWRIGHT_MIGRATION_CONTEXT.md 改造升级我的E2E系统,同时还需要完成跑通E2E的测试 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-07
**Phase:** 14-d-dev-rust-templates-tauri-sveltekit-axum-moon-template-docs
**Areas discussed:** 迁移深度、套件放置策略、CI 迁移策略、E2E 验收口径、全量边界

---

## 迁移深度

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 0最小闭环 | 先做插件门控/能力接入/新配置/1个 smoke，风险最小可回滚 | |
| 直接到Phase 1 | 一次性推进 smoke/login/counter 迁移，覆盖收益更快 | ✓ |
| Phase0+部分迁移 | Phase 0 + 仅 smoke/login，counter 后置 | |

**User's choice:** 直接到Phase 1
**Notes:** 用户明确本阶段不只做 bootstrap，要求进入可用迁移深度。

---

## 套件放置策略

| Option | Description | Selected |
|--------|-------------|----------|
| 独立目录新套件 | 与现有 web Playwright、WDIO 隔离，回滚和并行运行更直观 | ✓ |
| 复用现有Playwright | 复用 apps/client/web/app harness，结构更紧凑但耦合更高 | |
| 先复用后拆分 | 前期快，后续有一次拆分重构成本 | |

**User's choice:** 独立目录新套件
**Notes:** 用户接受推荐方案，优先控制耦合和回滚复杂度。

---

## CI 迁移策略

| Option | Description | Selected |
|--------|-------------|----------|
| 先增macOS观察通道 | 新增 macOS tauri-playwright job，过渡期与现有主线并行 | ✓ |
| 立即required门禁 | 新 job 立即作为 required check | |
| 本地先行不改CI | 只做本地验证，后续再接 workflow | |

**User's choice:** 先增macOS观察通道
**Notes:** 用户采用渐进式接入，先观测稳定性再提升门禁强度。

---

## E2E 验收口径

| Option | Description | Selected |
|--------|-------------|----------|
| 双轨最小验收 | WDIO 保持可跑 + 新 tauri smoke 可跑 + 关键 web 项目 green | |
| 新通道优先 | 仅要求新 tauri-playwright 通过 | |
| 全量绿再收敛 | 要求全量 E2E 通过后再收敛迁移 | ✓ |

**User's choice:** 全量绿再收敛
**Notes:** 用户要求高质量门槛，不接受仅迁移范围局部通过。

---

## 全量边界确认

| Option | Description | Selected |
|--------|-------------|----------|
| CI主线全绿 | 现有主线 + 新 tauri macOS 通道全绿（不含非CI矩阵） | |
| 仓库所有E2E项目全绿 | Playwright 全 projects + WDIO 全部 spec + 新 tauri 套件全绿 | ✓ |
| 仅迁移范围全绿 | 仅迁移用例和 required 主线通过 | |

**User's choice:** 仓库所有E2E项目全绿
**Notes:** 用户对“跑通 E2E”做了最严格定义，后续计划需按全仓 E2E 口径设计执行与收敛。

---

## the agent's Discretion

- 新套件目录内部结构与 helper 抽象层级。
- CI job 命名和 artifact 归档细节。

## Deferred Ideas

None.
