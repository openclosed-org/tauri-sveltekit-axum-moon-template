# Architecture Research

**Domain:** Agent-Native Cross-Platform Application Engineering Base
**Researched:** 2026-04-01
**Confidence:** HIGH (sourced from authoritative blueprint architecture docs)

## System Overview (Target State)

```
apps/client/desktop/    ← Tauri shell, bootstrap only
apps/client/web/app/    ← SvelteKit canonical app
servers/api/            ← Axum HTTP server
packages/core/domain/   ← Pure business rules, no host deps
packages/core/usecases/ ← Orchestration, CQ handlers
packages/core/state/    ← Session, cache, sync markers
packages/features/      ← auth, counter, admin, agent (each self-contained)
packages/adapters/      ← hosts, storage, auth, telemetry translation
packages/contracts/     ← Single truth source: api, auth, events, errors
packages/ui/            ← Design system, components, tokens
packages/shared/        ← Config, testing, utils
.tools/                 ← Scripts, MCP, evals
.agents/                ← Skills, prompts, playbooks, rubrics
```

## Layer Boundaries (红线)

| Boundary | Rule |
|----------|------|
| core 不得依赖 apps | 纯业务规则，不依赖宿主 |
| features 不得直接依赖 host app | 通过 contracts + adapters 协作 |
| adapters 不得承载业务策略 | 只做外部世界翻译 |
| contracts 不得被实现细节污染 | 单一真理源 |
| apps/servers/workers 只组合 contracts | 不重新定义数据事实 |

## Migration Path (v0.1.0 → blueprint)

Current → Target mapping:
- `src-tauri/` → `apps/client/desktop/`
- `frontend/` → `apps/client/web/app/`
- `crates/runtime_server/` → `servers/api/`
- `crates/domain/` → `packages/core/domain/`
- `packages/contracts/api` (placeholder) → filled with real DTOs
- `packages/adapters/hosts/tauri` (placeholder) → filled with runtime_tauri

Progressive, not big-bang. Each phase moves closer.

## Architectural Patterns

### 1. Contracts-First Single Truth Source
Rust contracts → codegen → TS types. No hand-written mirror types.

### 2. Host Adapter Isolation
Business logic in core/features, host-specific code in adapters.

### 3. Feature Module Pattern
Each feature: model/usecases/contracts/ui/adapters/tests.

### 4. Strangler-Style Runtime Migration
New capabilities go through new path, old code migrates gradually.

## Sources

- docs/blueprints/agent-native-starter-v1/02-repo-structure.md
- docs/blueprints/agent-native-starter-v1/04-contracts-typegen-and-boundaries.md
- docs/blueprints/agent-native-starter-v1/05-runtime-features-and-adapters.md
- docs/blueprints/agent-native-starter-v1/01-north-star-and-principles.md
- docs/blueprints/agent-native-starter-v1/12-migration-path.md

---
*Architecture research for: v0.2.0 架构蓝图对齐与核心功能实现*
*Researched: 2026-04-01*
