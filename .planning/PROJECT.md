# Tauri-SvelteKit-Axum Boilerplate

## What This Is

An Agent-Native Cross-Platform Application Engineering Base — a production-ready boilerplate for building cross-platform desktop applications using Tauri v2 + SvelteKit 2 + Axum + moon. Designed for long-term evolution with Rust-first, frontier-oriented technology choices and Agent-Friendly development infrastructure.

## Core Value

Provide a runnable, tested, production-ready engineering base with Google Auth, Counter, Admin Web, Agent conversation, contracts/typegen single-truth-source, and clear architectural boundaries — so developers (and AI agents) can start building business logic immediately with minimal cognitive overhead.

## Current Milestone: v0.2.1 跨平台测试与缺陷闭环强化

**Goal:** 建立发布门禁级别的 Windows E2E 自动化与 Windows/macOS 完整 QA/UAT/E2E 流程，并形成持续 bug 提报-分级-修复-回归闭环。

**Target features:**
- Windows desktop E2E 成为 required check（deterministic baseline + evidence）
- Windows + macOS QA/UAT 双平台门禁（平台差异化执行模型）
- Bug 生命周期治理（状态、严重级、SLA、关闭定义）
- Regression-on-fix 规则（P0/P1 修复必须补回归验证）
- 发布质量门槛与证据看板（测试结果、缺陷状态、UAT 签核可追溯）

## Current State

**Shipped:** v0.2.0 架构蓝图对齐与核心功能实现 (2026-04-04)

**In progress:** v0.2.1 跨平台测试与缺陷闭环强化 (started 2026-04-06)

**Delivered:**
- 仓库目录结构对齐蓝图（apps/servers/packages/crates/tools 分层）
- Contracts/typegen 单一真理源闭环（Rust→TS 自动生成，CI drift 检查）
- Runtime 边界收敛（core vs adapters vs hosts，业务规则不依赖宿主）
- 工具链任务图补全（moon + Just + proto 统一入口）
- 最小功能实现（Google Auth, Counter, Admin, Agent 对话）
- Agent-Friendly 开发基建（AGENTS/skills/playbooks/rubrics/prompts）
- 前端消费 generated types，消除 inline 重复定义
- Agent 页面 Tauri IPC 双路径（desktop vs browser）
- Phase 9 功能正确性基线修复完成（AUTH-02, AUTH-03, COUNTER-02, AGENT-02, AGENT-03, AGENT-04）
- Phase 10 多租户可重复验证通道完成（MTEN-01, MTEN-02, MTEN-03；CI artifact 可用性待人工 UAT 关闭）

**Known Tech Debt:**
- AUTH-01: GoogleAuthAdapter not fully wired into Tauri commands (Phase 6 empty)
- runtime_tauri directly instantiates services rather than through feature trait boundary (minor)

## Requirements

### Validated (from v0.2.0)

- ✓ 仓库目录结构对齐蓝图，建立 apps/servers/packages/crates/tools 分层边界 — v0.2.0 Phase 01
- ✓ packages/contracts/api 作为 Rust 单一真理源，自动生成 TS 类型 — v0.2.0 Phase 02
- ✓ typegen 从 Rust contracts 自动生成 TS 类型，CI drift check — v0.2.0 Phase 02
- ✓ Runtime 边界收敛：core 不依赖 host，adapters 不承载业务策略 — v0.2.0 Phase 03
- ✓ moon + Just + proto 提供统一的 setup/dev/verify/typegen 入口 — v0.2.0 Phase 01
- ✓ Counter 功能通过 feature 组合 core + contracts 实现 — v0.2.0 Phase 04
- ✓ Admin Web 通过 feature + UI 组件实现 — v0.2.0 Phase 04
- ✓ Agent 对话通过 OpenAI 兼容 API key 接入 — v0.2.0 Phase 04/08
- ✓ .agents/ 目录包含 skills、prompts、playbooks、rubrics — v0.2.0 Phase 05/08
- ✓ 前端消费 generated types，消除 inline 重复定义 — v0.2.0 Phase 07
- ✓ Settings 可见登出动作与会话清理闭环（AUTH-02, AUTH-03）— v0.2.1 Phase 09
- ✓ Counter 显示值与持久值一致，并具备失败可见反馈（COUNTER-02）— v0.2.1 Phase 09
- ✓ Agent New Chat 新线程语义且保留已保存配置（AGENT-02, AGENT-03）— v0.2.1 Phase 09
- ✓ Settings 连接诊断支持 API key/Base URL/Model 可操作反馈（AGENT-04）— v0.2.1 Phase 09
- ✓ 多租户固定测试通道、tenant-scoped counter 数据流与双栈证据归档（MTEN-01, MTEN-02, MTEN-03）— v0.2.1 Phase 10

### Active (Next Milestone)

- [ ] Windows desktop E2E required check（稳定、可重复、可审计）
- [ ] Windows + macOS QA/UAT/E2E 双平台放行标准
- [ ] Bug 生命周期治理与 triage 自动化
- [ ] P0/P1 修复回归强制规则与证据归档
- [ ] 发布质量看板与可回滚验证摘要

### Out of Scope

- [HTTP/3 默认启用] — 实验边车，V3 候选
- [多协议 federation runtime] — V2 能力
- [复杂多 agent 自主协作] — V3 候选
- [Email/password auth] — Google OAuth sufficient for current scope
- [Full RBAC] — 基本 multi-tenancy only

## Context

**Current state:** v0.2.0 shipped. 8 phases, 24 plans, 46 tasks complete. ~649K LOC across Rust + TypeScript + Svelte. 50 commits over 3 days (2026-04-01 → 2026-04-03).

**Tech stack (confirmed):**
- Desktop: Tauri v2
- Frontend: SvelteKit 2 + Svelte 5
- Backend: Axum
- Package Manager: Bun
- Task Orchestration: moon
- Toolchain: proto
- Task Entry: Just
- Database: SurrealDB + libsql
- Cache: moka
- Agent LLM: OpenAI 兼容 API

**Architecture reference:** `docs/blueprints/agent-native-starter-v1/` (19 documents)

## Constraints

- **[Stack]**: Tauri v2 + SvelteKit 2 + Svelte 5 + Axum + moon — Full-stack Rust/WebView
- **[Architecture]**: 必须对齐 agent-native-starter-v1 蓝图的分层边界
- **[Timeline]**: Best effort for production-ready quality
- **[Scope]**: Desktop-first but web-accessible, mobile-responsive
- **[Testing]**: Must have passing tests for core flows before release
- **[Agent-Friendly]**: 所有基建要让 agent 开发更友好

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 蓝图驱动重构 | docs/blueprints 定义了更全面的架构方向 | ✓ v0.2.0 shipped |
| Rust-first, 前沿导向 | Rust 提供无与伦比的性能、安全性和可靠性 | ✓ 确定 |
| SvelteKit 2 + Svelte 5 确定 | 编译时优化，Runes 响应式，性能领先 | ✓ 不再考虑替代 |
| Tauri v2 确定 | Rust 原生，安全性高，插件生态成熟 | ✓ 不再考虑替代 |
| Axum 确定 | Tokio 生态，类型安全，性能优秀 | ✓ 不再考虑替代 |
| contracts 作为单一真理源 | 避免 agent 开发时四套真相漂移 | ✓ Phase 02 validated |
| --reset-phase-numbers | 蓝图驱动的里程碑与 v0.1.x 完全不同 | ✓ 确定 |
| contracts 分为 api/auth/events 三个独立 crate | 蓝图 D-03 对齐，每个 concern 有独立命名空间 | ✓ Phase 02 shipped |
| ts-rs + utoipa 共存于同一 struct | OpenAPI 和 TS 类型来自单一事实源 | ✓ Phase 02 shipped |
| Tauri 命令使用 AppHandle + Manager::state 避免循环依赖 | AppState 定义在 native-tauri，runtime_tauri 无法导入 | ✓ Phase 03 shipped |
| ts-rs i64 字段使用 #[ts(type = "number")] 避免 bigint | bigint 不能 JSON 序列化 | ✓ Phase 07 shipped |
| Agent 页面 Tauri IPC 双路径 | Desktop 环境无需外部 API server | ✓ Phase 08 shipped |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---

*Last updated: 2026-04-06 after Phase 9 completion*
