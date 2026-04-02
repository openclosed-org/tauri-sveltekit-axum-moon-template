# Phase 4: 最小功能实现 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-02
**Phase:** 04-minimal-feature-implementation
**Areas discussed:** Auth refactoring, Counter persistence & IPC, Agent Chat architecture, Admin real data, IPC architecture

---

## Auth Refactoring

| Option | Description | Selected |
|--------|-------------|----------|
| Full refactor into feature+adapter | Move OAuth to packages/adapters/auth/google, define AuthService trait in feature-auth | ✓ |
| Align types only | Keep code in runtime_tauri, only align type definitions with contracts_auth | |
| Leave as-is | Don't touch auth for v0.1, apply pattern to other features only | |

**User's choice:** Full refactor into feature+adapter

### Follow-up: Adapter location

| Option | Description | Selected |
|--------|-------------|----------|
| Adapters/auth/google crate | Google-specific OAuth adapter, leaves room for other providers | ✓ |
| Adapters/auth/oauth crate | Generic OAuth adapter for all providers | |

**User's choice:** Adapters/auth/google crate

### Follow-up: Type unification

| Option | Description | Selected |
|--------|-------------|----------|
| Keep both (different concerns) | runtime_tauri AuthSession for Tauri store, contracts_auth TokenPair for API layer | ✓ |
| Use contracts types only | Unify to single type definition | |

**User's choice:** Keep both (different concerns)

### Follow-up: Feature-auth pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Define use case trait | AuthService trait with methods: start_login, handle_callback, get_session, refresh, logout | ✓ |
| Re-export only | feature-auth just re-exports adapter + contracts, no new trait | |

**User's choice:** Define use case trait

---

## Counter Persistence & IPC

| Option | Description | Selected |
|--------|-------------|----------|
| LibSQL local persistence | Persist counter value in embedded LibSQL, survives restart, optional cloud sync | ✓ |
| Tauri store (simple JSON) | Simpler JSON file storage, no SQL | |
| In-memory only | Counter resets on restart, no persistence | |

**User's choice:** LibSQL local persistence (recommended)

### Follow-up: IPC mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri commands (standard) | Frontend calls invoke() with counter commands | |
| HTTP to Axum server | Frontend calls REST API via fetch() | |
| Both (Tauri + HTTP) | Both paths exist, frontend chooses based on runtime | ✓ |

**User's choice:** Both (Tauri + HTTP)

### Follow-up: Cloud sync

| Option | Description | Selected |
|--------|-------------|----------|
| Local + optional cloud sync | LibSQL local, synced to Turso if TURSO_SYNC_URL configured | ✓ |
| Local only | No cloud sync, always local | |

**User's choice:** Local + optional cloud sync

### Follow-up: Dual IPC dispatch

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime detection | Detect Tauri vs browser at runtime, use invoke() or fetch() accordingly | ✓ |
| Tauri primary, HTTP fallback | Prefer Tauri commands, HTTP is secondary | |

**User's choice:** Runtime detection (recommended)

---

## Agent Chat Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Chat + read-only product access | Agent has fixed tool set (get_counter_value, list_tenants, get_system_status) | ✓ |
| Chat + tool calling (write) | Agent can create tenants, reset counter, etc. | |
| Chat-only (conversation demo) | No tools, pure conversation | |

**User's choice:** Chat + read-only product access

### Follow-up: API key management

| Option | Description | Selected |
|--------|-------------|----------|
| Settings page input | User enters API key + base URL in Settings, stored in Tauri store | ✓ |
| First-use prompt + Settings | Prompted on first chat, also configurable in Settings | |
| Environment variable only | Not user-configurable in-app | |

**User's choice:** Settings page input

### Follow-up: Chat persistence

| Option | Description | Selected |
|--------|-------------|----------|
| LibSQL persistence | Conversations + messages stored in embedded LibSQL | ✓ |
| In-memory only | Lost on app close | |

**User's choice:** LibSQL persistence

### Follow-up: Streaming

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — SSE or Tauri events | Token-by-token streaming for better UX | ✓ |
| No — full response | Wait for complete response, simpler | |

**User's choice:** Yes — SSE or Tauri events

### Follow-up: Tool integration model

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed small tool set | 2-3 fixed tools (get_counter_value, list_tenants, get_system_status) | ✓ |
| No tools — UI shows state | Chat-only, state shown in sidebar | |

**User's choice:** Fixed small tool set (recommended)

---

## Admin Real Data

| Option | Description | Selected |
|--------|-------------|----------|
| Real data from DB | Pull from LibSQL/SurrealDB, data changes with app state | ✓ |
| Dynamic mock data | Randomize mock data on load, looks alive | |
| Static mock (current) | Keep hardcoded numbers | |

**User's choice:** Real data from DB (recommended)

### Follow-up: Metrics

| Option | Description | Selected |
|--------|-------------|----------|
| Tenant, Counter, Login, Version | 4 cards: Tenant count, Counter value, Last login time, App version | ✓ |
| Custom metrics | User specifies which metrics | |

**User's choice:** Tenant, Counter, Login, Version

### Follow-up: Data access pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Via usecases (feature crate) | feature-admin calls usecases, follows hexagonal boundary | ✓ |
| Direct adapter reads | Simpler, bypasses usecases layer | |

**User's choice:** Via usecases (feature crate pattern)

---

## IPC Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Dual-path per feature | Runtime detection: invoke() in Tauri, fetch() in browser. Same usecases serve both | ✓ |
| Tauri commands only | All communication via Tauri invoke() | |
| HTTP API only | All communication via Axum HTTP endpoints | |

**User's choice:** Dual-path per feature (recommended)

### Follow-up: Axum server role

| Option | Description | Selected |
|--------|-------------|----------|
| Full API + Tauri bridge | Axum has routes for counter, admin, agent. Not just external gateway | ✓ |
| External API gateway only | Axum serves health/tenant only, features are desktop-only | |

**User's choice:** Full API + Tauri bridge

### Follow-up: IPC layer location

| Option | Description | Selected |
|--------|-------------|----------|
| Per-feature IPC module | Each feature has its own IPC module (e.g., feature-counter/ipc/) | ✓ |
| Shared lib/ipc/ (current) | All features' IPC in apps/client/web/app/src/lib/ipc/ | |

**User's choice:** Per-feature IPC module

---

## the agent's Discretion

- feature crate 命名（`feature-auth` vs `auth-feature`）
- AuthService trait 的具体方法签名
- LibSQL migration 的具体 schema 设计
- Agent chat UI 的具体布局
- Streaming 的具体实现方式（SSE vs Tauri event channel）
- IPC 运行时检测的具体机制

## Deferred Ideas

- Agent tool calling 的写操作 — 后续版本
- Email/password auth — Out of scope
- Full RBAC — Out of scope for v0.1
- Counter 的多计数器支持
- Agent 多模态支持
- Settings 页面完整实现
