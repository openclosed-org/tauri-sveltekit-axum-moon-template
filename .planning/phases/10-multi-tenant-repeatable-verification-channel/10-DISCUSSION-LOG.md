# Phase 10: 多租户可重复验证通道 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 10-multi-tenant-repeatable-verification-channel
**Areas discussed:** 执行矩阵, 租户身份建模, 隔离断言口径, CI 诊断证据

---

## 执行矩阵

| Option | Description | Selected |
|--------|-------------|----------|
| 先 Web 后 Desktop | 先做 web 路径双租户回归，再迁移到 WDIO desktop。 | |
| 双线并行 | Playwright 与 WDIO 同时推进并同时达标。 | ✓ |
| 先 Desktop | 先仅做 WDIO desktop 双租户。 | |

**User's choice:** 双线并行
**Notes:** 在并行路径下，用户进一步锁定最小门禁为 `Web+WDIO最小集`（Playwright `desktop-chrome` + WDIO desktop）。

---

## 租户身份建模

| Option | Description | Selected |
|--------|-------------|----------|
| 固定双租户映射 | 固定 tenant-A/tenant-B 对应 mock user_sub，并自动初始化/清理。 | ✓ |
| 动态生成映射 | 每次随机生成并记录映射到 artifact。 | |
| 你来定 | 由 agent 决定映射与清理策略。 | |

**User's choice:** 固定双租户映射
**Notes:** 初始化失败策略锁定为 fail-fast（任一租户 init 失败即终止）。

---

## 隔离断言口径

| Option | Description | Selected |
|--------|-------------|----------|
| 行为断言为主 | 写入-读取链路验证 tenant-A/B 隔离，重复运行保持一致。 | ✓ |
| 行为+存储双断言 | 在行为断言外增加底层存储直连断言。 | |
| 仅 UI 断言 | 只看 UI 值变化。 | |

**User's choice:** 行为断言为主
**Notes:** 重复运行策略锁定为每个用例前显式重置 tenant-A/tenant-B 到已知初值。

---

## CI 诊断证据

| Option | Description | Selected |
|--------|-------------|----------|
| 最小诊断包 | Playwright trace/video/screenshot + WDIO JUnit + 关键日志 + 租户映射。 | ✓ |
| 最全证据包 | 额外包含 HAR 与完整录像。 | |
| 轻量证据包 | 仅 JUnit + 日志。 | |

**User's choice:** 最小诊断包
**Notes:** 上传策略锁定为按 job 上传，保留 7 天。

---

## the agent's Discretion

- 固定映射命名方案与文件布局。
- 重置逻辑是 fixture 前置还是 helper 封装。
- 诊断包的目录结构与命名细节。

## Deferred Ideas

- 存储层直连强断言（超出本阶段最小验收口径）。
- 扩展更大测试矩阵与更长证据保留（后续阶段再升级）。
