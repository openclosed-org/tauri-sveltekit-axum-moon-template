# Project Research Summary

**Project:** Tauri-SvelteKit-Axum Boilerplate (v0.2.0)
**Domain:** Agent-Native Cross-Platform Application Engineering Base
**Researched:** 2026-04-01
**Confidence:** HIGH

## Executive Summary

v0.2.0 是蓝图驱动的里程碑：从 `docs/blueprints/agent-native-starter-v1/` 出发，把仓库从"可运行模板"升级为"可长期演化的制度化工程"。核心工作是：目录结构对齐蓝图、contracts/typegen 单一真理源闭环、runtime 边界收敛、工具链任务图补全、最小功能实现（Google Auth、Counter、Admin、Agent 对话）。

v0.1.0 的 10 个 phases 已归档，v0.1.1 的收敛计划已废弃。本次从蓝图第一性原理出发，--reset-phase-numbers 从 Phase 1 开始。

## Key Findings

### Recommended Stack

确认选型（不再替代）：Tauri v2, SvelteKit 2 + Svelte 5, Axum, Bun, moon, proto, Just。新增：ts-rs（typegen）, async-openai（agent 对话）。

### Expected Features

**Must have (v0.2.0):**
- 仓库目录结构对齐蓝图
- Contracts/typegen 闭环
- 工具链任务图（moon + Just + proto）
- Google Auth / Counter / Admin Web / Agent 对话
- Agent-Friendly 基建（.agents/）

**Future (V2+):**
- Host adapter 体系做实
- Worker replay / offline sync
- Protocol/chains adapter 骨架

### Architecture Approach

Contract-first + Strangler-style runtime migration. Progressive migration from current structure toward blueprint target structure.

### Critical Pitfalls

1. **Directory boundary violation** — core depends on host
2. **Hand-written type drift** — Rust/TS diverge without contracts
3. **Host adapter business logic** — Tauri commands carry business rules
4. **Feature coupling without contracts** — direct cross-feature imports
5. **Secret leakage** — config serialization exposes secrets

## Implications for Roadmap

### Phase 1: 仓库目录结构与工具链对齐
对齐蓝图目录结构，配置 moon/Just/proto 统一入口。

### Phase 2: Contracts/typegen 单一真理源
建立 packages/contracts 作为 Rust→TS 自动生成源，CI drift 检查。

### Phase 3: Runtime 边界收敛
core vs adapters vs hosts 职责清晰化，新能力走新路径。

### Phase 4: 最小功能实现
Google Auth, Counter, Admin Web, Agent 对话通过 feature + adapter 模式实现。

### Phase 5: Agent-Friendly 开发基建
AGENTS.md, skills, playbooks, rubrics, eval suites。

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | 确定选型，不再评估替代 |
| Features | HIGH | 蓝图明确最小实现范围 |
| Architecture | HIGH | 19 份蓝图文档交叉验证 |
| Pitfalls | HIGH | 蓝图红线规则 + 迁移路径 |

**Overall confidence:** HIGH

## Sources

- docs/blueprints/agent-native-starter-v1/ (19 documents)
- .planning/PROJECT.md, STATE.md, REQUIREMENTS.md

---
*Research completed: 2026-04-01*
*Ready for roadmap: yes*
