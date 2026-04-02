# Tauri-SvelteKit-Axum Boilerplate

## What This Is

An Agent-Native Cross-Platform Application Engineering Base — a production-ready boilerplate for building cross-platform desktop applications using Tauri v2 + SvelteKit 2 + Axum + moon. Designed for long-term evolution with Rust-first, frontier-oriented technology choices and Agent-Friendly development infrastructure.

## Core Value

Provide a runnable, tested, production-ready engineering base with Google Auth, Counter, Admin Web, Agent conversation, contracts/typegen single-truth-source, and clear architectural boundaries — so developers (and AI agents) can start building business logic immediately with minimal cognitive overhead.

## Current Milestone: v0.2.0 架构蓝图对齐与核心功能实现

**Goal:** 对齐 agent-native-starter-v1 蓝图，重构仓库目录结构与边界，建立 contracts/typegen 闭环，实现最小功能集（Google Auth, Counter, Admin Web, Agent 对话）。

**Target features:**
- 仓库目录结构对齐蓝图（apps/servers/packages/crates/tools 分层）
- Contracts/typegen 单一真理源闭环（Rust→TS 自动生成，CI drift 检查）
- Runtime 边界收敛（core vs adapters vs hosts，业务规则不依赖宿主）
- 工具链任务图补全（moon + Just + proto 统一入口）
- 最小功能实现（Google Auth, Counter, Admin, Agent 对话）
- Agent-Friendly 开发基建（AGENTS/skills/playbooks/rubrics）

## Requirements

### Validated (from v0.1.0)

- ✓ Tauri 2 desktop app scaffolding — existing
- ✓ SvelteKit 2 + Svelte 5 frontend foundation — existing
- ✓ Axum 0.8 backend server — existing
- ✓ moon build toolchain — existing
- ✓ Mobile-first responsive layout base — existing
- ✓ Frontend dependencies aligned — Validated in v0.1.0 Phase 01
- ✓ Rust workspace dependencies pinned — Validated in v0.1.0 Phase 01
- ✓ All 7 Tauri plugins registered — Validated in v0.1.0 Phase 01
- ✓ Database infrastructure (SurrealDB + libsql) — Validated in v0.1.0 Phase 05
- ✓ Multi-tenant data isolation — Validated in v0.1.0 Phase 07
- ✓ Rust integration tests + Vitest + Playwright — Validated in v0.1.0 Phase 10

### Active (v0.2.0)

- ✓ 仓库目录结构对齐蓝图，建立 apps/servers/packages/crates/tools 分层边界 — Validated in Phase 01
- ✓ packages/contracts/api 作为 Rust 单一真理源，自动生成 TS 类型 — Validated in Phase 02
- [ ] Runtime 边界收敛：core 不依赖 host，adapters 不承载业务策略
- ✓ moon + Just + proto 提供统一的 setup/dev/verify/typegen 入口 — Validated in Phase 01
- [ ] Google Auth 通过 adapter 接入，不污染 core
- [ ] Counter 功能通过 feature 组合 core + contracts 实现
- [ ] Admin Web 通过 feature + UI 组件实现
- [ ] Agent 对话功能通过 OpenAI 兼容 API key 接入

### Out of Scope

- [HTTP/3 默认启用] — 实验边车，V3 候选
- [多协议 federation runtime] — V2 能力
- [复杂多 agent 自主协作] — V3 候选
- [Email/password auth] — Google OAuth sufficient for V1
- [Full RBAC] — 基本 multi-tenancy only for V1

## Context

**Current state:** v0.2.0 Phase 03 (application-pages) complete. Routing infrastructure (auth/app route groups, responsive nav with sidebar + bottom tabs), Counter page with Svelte 5 $state rune, Admin dashboard placeholder with stats cards and chart placeholders, Settings page with dark mode toggle. All 4 pages functional with SPA navigation. Ready for Phase 04 (backend-dependencies-build-optimization).

**Tech stack (confirmed):**
- Desktop: Tauri v2 (确定)
- Frontend: SvelteKit 2 + Svelte 5 (确定)
- Backend: Axum (确定)
- Package Manager: Bun
- Task Orchestration: moon
- Toolchain: proto
- Task Entry: Just
- Database: SurrealDB + libsql
- Cache: moka
- Agent LLM: OpenAI 兼容 API

**Architecture reference:** `docs/blueprints/agent-native-starter-v1/` (19 documents)

**Date reference:** April 1, 2026 — verify all versions/dependencies are current

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
| 蓝图驱动重构 | docs/blueprints 定义了更全面的架构方向 | ✓ v0.2.0 启动 |
| Rust-first, 前沿导向 | Rust 提供无与伦比的性能、安全性和可靠性 | ✓ 确定 |
| SvelteKit 2 + Svelte 5 确定 | 编译时优化，Runes 响应式，性能领先 | ✓ 不再考虑替代 |
| Tauri v2 确定 | Rust 原生，安全性高，插件生态成熟 | ✓ 不再考虑替代 |
| Axum 确定 | Tokio 生态，类型安全，性能优秀 | ✓ 不再考虑替代 |
| contracts 作为单一真理源 | 避免 agent 开发时四套真相漂移 | ✓ Phase 02 validated |
| --reset-phase-numbers | 蓝图驱动的里程碑与 v0.1.x 完全不同 | ✓ 确定 |

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

*Last updated: 2026-04-02 after Phase 03 (application-pages) — routing, counter, admin dashboard, settings page*
