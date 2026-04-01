# Phase 1: 仓库目录结构与工具链对齐 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 01-仓库目录结构与工具链对齐
**Areas discussed:** 目录骨架策略, Moon 任务图设计, Justfile 入口重命名, proto 工具链版本, servers vs workers 分层

---

## 目录骨架策略

| Option | Description | Selected |
|--------|-------------|----------|
| 全部占位（推荐） | 按蓝图创建所有目录，空目录放 .gitkeep。新 agent 一眼看出完整架构。 | ✓ |
| 仅结构层占位 | 创建一级目录 + 核心二级目录，跳过目前明显不需要的。 | |
| 只创建有代码的 | 不创建空目录，只在有实际代码时才建。 | |

**User's choice:** 全部占位（推荐）
**Notes:** 按蓝图创建全部目录骨架（70+ 子目录），空目录放 .gitkeep。

---

## Moon 任务图设计

| Option | Description | Selected |
|--------|-------------|----------|
| 最小集（推荐） | 先实现 repo:setup + repo:dev-fullstack + repo:verify + repo:typegen 四个核心任务。 | |
| 蓝图完整集 | 一次性实现蓝图建议的所有任务（约 30 个）。 | ✓ |
| 你来决定 | 根据当前仓库实际能力，选择合理的任务子集。 | |

**User's choice:** 蓝图完整集
**Notes:** 一次性实现约 30 个 repo:* 任务，覆盖 setup/dev/quality/codegen/ops/security 六大类。

---

## Justfile 入口重命名

| Option | Description | Selected |
|--------|-------------|----------|
| 直接替换（推荐） | 删除旧命名，按蓝图重写 Justfile。 | ✓ |
| 别名共存 | 保留旧命名作为别名，同时添加蓝图要求的新入口。 | |
| 只加新的 | 只添加蓝图要求的新入口，现有的 dev/test 保持不变。 | |

**User's choice:** 直接替换（推荐）
**Notes:** 仓库处于重构阶段，不存在需要向后兼容的用户。

---

## proto 工具链版本

| Option | Description | Selected |
|--------|-------------|----------|
| proto 统一管理（推荐） | 创建 .prototools 锁定 Rust + Bun + Node 版本。 | |
| proto 只管 Bun/Node | Rust 继续由 rust-toolchain.toml 管理，proto 只管 Bun 和 Node。 | ✓ |
| 暂不引入 proto | Phase 1 只创建 .prototools 空文件或最小配置。 | |

**User's choice:** proto 只管 Bun/Node
**Notes:** Rust 继续由 rust-toolchain.toml 管理（与 cargo 生态一致），proto 只管 Bun 和 Node 版本。

---

## servers vs workers 分层

| Option | Description | Selected |
|--------|-------------|----------|
| 直接迁移 | 将 servers/workers/ 提升为顶层 workers/，更新 Cargo workspace 和 moon workspace.yml。 | ✓ |
| 保留结构，标记迁移 | Phase 1 先创建顶层 workers/ 目录骨架，实际代码迁移推迟到 Phase 3。 | |
| 你来决定 | 先检查 servers/workers/ 下的实际内容，再决定迁移时机。 | |

**User's choice:** 直接迁移
**Notes:** servers/workers/ 下都是空占位目录（.gitkeep + README.md），迁移无风险。

---

## the agent's Discretion

无 — 所有区域用户均给出了明确选择。

## Deferred Ideas

- workers 下各协议的实际实现 — Phase 3+
- tools/evals/ 下评估数据集 — Phase 5
- apps/ops/ 下 docs-site/storybook — 按需
- .agents/prompts/playbooks/rubrics 实际内容 — Phase 5
