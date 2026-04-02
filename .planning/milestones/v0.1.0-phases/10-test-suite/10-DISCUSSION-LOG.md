# Phase 10: Test Suite - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 10-test-suite
**Areas discussed:** Auth E2E策略, 核心流覆盖边界, 测试分层与目录, CI 执行门禁

---

## Auth E2E策略

| Option | Description | Selected |
|--------|-------------|----------|
| Mock OAuth回调 | E2E 中不走真实 Google，模拟 callback，保证 CI 稳定 | ✓ |
| 真实Google OAuth | 全链路真实但依赖外部账号与浏览器，CI 脆弱 | |
| 预置已登录状态 | 直接写入 session，速度快但覆盖较弱 | |

**User's choice:** Mock OAuth回调
**Notes:** 选定后继续细化 mock 粒度。

| Option | Description | Selected |
|--------|-------------|----------|
| 模拟 deep-link 事件 | 触发 `deep-link://new-url`，复用现有 callback 处理链路 | ✓ |
| 直接调用 setSession | 速度快，但绕过 callback/IPC 链路 | |
| 双层：主要事件+少量直设 | 主路径走事件，个别异常场景直设 session | |

**User's choice:** 模拟 deep-link 事件
**Notes:** 明确偏好更接近真实用户路径的 mock。

---

## 核心流覆盖边界

| Option | Description | Selected |
|--------|-------------|----------|
| 最小三流 | login + counter + admin，严格贴 ROADMAP 最小项 | |
| 三流+tenant隔离 | 在最小三流上加入 tenant 隔离 | |
| 三流+tenant+token刷新 | 再加入 token 刷新与过期链路 | ✓ |

**User's choice:** 三流+tenant+token刷新
**Notes:** 将 core 范围从最小三流扩展到安全与会话续期关键链路。

| Option | Description | Selected |
|--------|-------------|----------|
| 以Rust单测为主 | tenant/refresh 主要在 unit 覆盖，E2E 轻量验证 | |
| 都放E2E为主 | 更多端到端覆盖，但更慢更脆弱 | |
| 全都双重覆盖 | unit + e2e 都覆盖关键行为 | ✓ |

**User's choice:** 全都双重覆盖
**Notes:** 扩展 core 的两项（tenant、token 刷新）要求双层验证。

---

## 测试分层与目录

| Option | Description | Selected |
|--------|-------------|----------|
| 模块内单测+tests集成 | 保留 `#[cfg(test)]` 并新增 `tests/` 跨模块集成 | ✓ |
| 全放模块内单测 | 改动小，但跨模块覆盖偏弱 | |
| 全放tests集成 | 结构整齐，但运行/维护成本更高 | |

**User's choice:** 模块内单测+tests集成
**Notes:** Rust 采用混合结构，兼顾局部验证和跨模块行为。

| Option | Description | Selected |
|--------|-------------|----------|
| 集中 tests/ 目录 | `tests/component` + `tests/e2e` 统一管理 | ✓ |
| 与源码同位放置 | 贴近代码，但目录风格混合 | |
| 完全混合自定 | 自由度高，但模板可复制性较差 | |

**User's choice:** 集中 tests/ 目录
**Notes:** 前端测试目录集中到 `apps/desktop-ui/tests/` 下。

---

## CI 执行门禁

| Option | Description | Selected |
|--------|-------------|----------|
| PR全量必跑 | PR 必跑 cargo test + vitest + playwright | ✓ |
| PR轻量，main全量 | PR 快，问题晚暴露 | |
| 分层两级门禁 | PR 跑核心子集，完整集夜间跑 | |

**User's choice:** PR全量必跑
**Notes:** 将三层测试全部提升为 PR 合并前门禁。

---

## the agent's Discretion

- Mock deep-link 的具体 fixture/test utils 组织方式。
- 测试并发、重试、超时与缓存策略的实现细节。

## Deferred Ideas

- 真实 Google OAuth 的外部集成回归可作为后续独立 smoke/staging 流程。
