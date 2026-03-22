# State: geek taste

## Project Reference

**Core Value:** 高信噪比、可行动、低打扰的技术信号
**What This Is:** 跨端技术雷达工作台 — TopK 趋势发现 + Repo 订阅跟踪 + Agent 资源雷达
**Stack:** Tauri v2 + SvelteKit 5 + SQLite (rusqlite) + octocrab

## Current Position

**Phase:** — (not started)
**Plan:** — (not started)
**Status:** Roadmap created, awaiting Phase 0 planning
**Progress:** [░░░░░░░░░░░░░░░░░░░░] 0/29 requirements

## Phase Summary

| Phase | Goal | Reqs | Status |
|-------|------|------|--------|
| 0. moonrepo 工程化基建 | monorepo 配置 + 目录结构 + Cargo workspace + CI/CD | — | Not started |
| 1. 项目脚手架与认证 | 启动应用、认证 GitHub、导航结构 | 4 | Not started |
| 2. 数据层与 TopK 发现引擎 | SQLite + GitHub 客户端 + TopK 排名 | 11 | Not started |
| 3. 订阅系统与信号模型 | 订阅 CRUD + Signal + Home | 10 | Not started |
| 4. Agent 资源雷达 | MCP/Skills/Agent 资源发现 | 3 | Not started |
| 5. 打磨与发布准备 | 离线 + 性能 + 发布 | 1 | Not started |

## Performance Metrics

| Metric | Value |
|--------|-------|
| Requirements total | 29 |
| Requirements validated | 0 |
| Phases complete | 0/6 |
| Current phase progress | — |

## Accumulated Context

### Key Decisions
- 桌面优先，本地优先 — SQLite 为主权威数据库
- REST-first — Search/Releases/Tags 端点清晰
- 轮询+差分+摘要 — 不依赖 webhook
- TopK = 产品定义排行榜 — 非 GitHub Trending 复制品
- 规则+模板摘要为主 — v1 不做全自动 LLM 摘要

### Known Risks
- Tauri v2 ACL 权限遗漏 → Phase 1 必须验证
- SQLite 并发写入 → Phase 2 启用 WAL + busy_timeout
- GitHub API 速率预算 → Phase 2 core/search 分池隔离
- SvelteKit static-adapter 误用 server 功能 → Phase 1 配置 ssr=false

### Blockers
- None currently

## Session Continuity

**Last action:** Roadmap updated — added Phase 0 (moonrepo monorepo setup)
**Next action:** `/gsd-plan-phase 0` — Plan Phase 0: moonrepo 工程化基建
**Context needed for next session:** moonrepo/moon docs, docs/03 architecture spec (Cargo workspace layout), research STACK.md (Tauri v2 + SvelteKit versions)

---
*Last updated: 2026-03-22 — Phase 0 added to roadmap*
