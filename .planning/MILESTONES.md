# MILESTONES: Tauri-SvelteKit-Axum Boilerplate

## Milestone History

### v0.1.0 — 基础设施搭建与核心功能实现

**Goal:** 从零搭建 Tauri 2 + SvelteKit + Axum + moon 全栈桌面应用模板，实现基础功能闭环。

**Completed:** 2026-04-01

**Delivered features:**
- Package foundation (frontend + Rust workspace deps, 7 Tauri plugins)
- UI styling infrastructure (TailwindCSS v4, dark mode, component system)
- Application pages (Login, Counter, Admin dashboard)
- Backend dependencies & build optimization (Axum middleware stack)
- Database infrastructure (SurrealDB + libsql dual-DB, domain ports)
- Google OAuth authentication
- Multi-tenant data isolation (TenantId, JWT middleware, tenant init API)
- Desktop native features
- Cross-platform build pipeline (GitHub Actions CI matrix)
- Test suite (30 Rust tests, 28 Vitest tests, 28 Playwright E2E tests = 86 total)

**Phases:** 01-10 (archived at `.planning/milestones/v0.1.0-phases/`)

---

### v0.2.0 — 架构蓝图对齐与核心功能实现

**Goal:** 对齐 agent-native-starter-v1 蓝图，重构仓库结构，建立 contracts/typegen 闭环，实现最小功能集（Google Auth, Counter, Admin Web, Agent 对话）。

**Status:** Active

**Target features:**
- 仓库目录结构对齐蓝图（apps/servers/packages/crates/tools 分层）
- Contracts/typegen 单一真理源闭环
- Runtime 边界收敛（core vs adapters vs hosts）
- 工具链任务图补全（moon + Just + proto）
- 最小功能实现（Google Auth, Counter, Admin, Agent 对话）
- Agent-Friendly 开发基建（AGENTS/skills/playbooks/rubrics）

---

*Last updated: 2026-04-01*
