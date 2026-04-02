---
phase: 04-minimal-feature-implementation
verified: 2026-04-02T15:14:29Z
status: gaps_found
score: 2/5 must-haves verified
gaps:
  - truth: "用户可以通过 Google 账号登录，auth adapter 不污染 core。"
    status: failed
    reason: "Google 登录仍由 runtime_tauri/commands/auth.rs 内联实现，未接入 adapter-google / feature-auth。"
    artifacts:
      - path: "packages/adapters/hosts/tauri/src/commands/auth.rs"
        issue: "仍保留完整 OAuth/PKCE 实现（start_oauth/handle_oauth_callback/get_session），未委托 GoogleAuthAdapter。"
      - path: "packages/adapters/auth/google/src/lib.rs"
        issue: "GoogleAuthAdapter 存在但未被 runtime_tauri 调用。"
      - path: "packages/features/auth/src/lib.rs"
        issue: "AuthService trait 已定义但无实现/无接线。"
    missing:
      - "将 Tauri auth commands 改为薄封装：调用 adapter-google（至少 start_login/handle_callback/get_session/refresh/clear）。"
      - "补齐 adapter-google -> feature-auth(AuthService) 的实现与注入，完成 AUTH-01 的 adapter 接入闭环。"
      - "删除或最小化 runtime_tauri 中重复 OAuth 逻辑，避免双实现漂移。"
  - truth: "用户可以通过 API key 与 agent 对话，agent 可以操作产品功能。"
    status: partial
    reason: "对话与流式返回可用，但“操作产品功能”仅声明工具列表，未执行工具调用。"
    artifacts:
      - path: "packages/core/usecases/src/agent_service.rs"
        issue: "仅把 AVAILABLE_TOOLS 传给模型；未解析/执行 tool calls（未调用 counter/tenant/system 业务方法）。"
      - path: "servers/api/src/routes/agent.rs"
        issue: "仅透传 chat_stream 文本片段，无工具执行分支。"
    missing:
      - "在 agent_service 中实现 tool call 执行链路（至少 get_counter_value/list_tenants/get_system_status）。"
      - "将工具执行结果写回 assistant/tool 消息并持久化，形成可验证的数据流。"
      - "补充可复现的端到端验证（触发工具调用并返回工具结果）。"
  - truth: "所有功能通过 feature 模块实现，使用 contracts 类型。"
    status: failed
    reason: "counter/admin/agent 基本满足，但 auth 仍绕过 feature-auth 与 adapter 接口，导致全局条件不成立。"
    artifacts:
      - path: "packages/adapters/hosts/tauri/src/commands/auth.rs"
        issue: "未使用 feature-auth::AuthService / adapter-google。"
      - path: "packages/features/auth/src/lib.rs"
        issue: "仅 trait 定义，无实际 host 接入。"
    missing:
      - "以 feature-auth trait 为边界完成 auth 的 host 侧接线与实现。"
      - "确保 phase 4 四项功能都通过 feature 模块路径落地。"
---

# Phase 4: 最小功能实现 Verification Report

**Phase Goal:** 用户可以登录、使用计数器、访问 Admin、与 Agent 对话。  
**Verified:** 2026-04-02T15:14:29Z  
**Status:** gaps_found  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | 用户可以通过 Google 账号登录，auth adapter 不污染 core。 | ✗ FAILED | `packages/adapters/hosts/tauri/src/commands/auth.rs` 仍是完整 OAuth 逻辑；`packages/adapters/auth/google/src/lib.rs` 的 `GoogleAuthAdapter` 未被调用；`feature-auth` trait 未接线。 |
| 2 | 用户可以使用计数器（increment/decrement/reset），前后端通信正常。 | ✓ VERIFIED | Tauri commands: `counter_increment/decrement/reset/get_value` 已注册（`apps/client/native/src-tauri/src/lib.rs:93-97`）；Axum 路由存在（`servers/api/src/routes/counter.rs`）；前端 dual-path 调用（`counter/+page.svelte` invoke/fetch）。 |
| 3 | 用户可以访问管理后台，包含基本统计卡片。 | ✓ VERIFIED | `admin/+page.svelte` 渲染 4 张统计卡；数据来自 `admin_get_dashboard_stats` 或 `/api/admin/stats`；后端 `LibSqlAdminService` 聚合 tenant+counter。 |
| 4 | 用户可以通过 API key 与 agent 对话，agent 可以操作产品功能。 | ✗ FAILED | 对话与流式返回存在（`agent/+page.svelte` + `routes/agent.rs` + `chat_stream`），但工具仅声明 `AVAILABLE_TOOLS`，未执行业务工具调用。 |
| 5 | 所有功能通过 feature 模块实现，使用 contracts 类型。 | ✗ FAILED | counter/admin/agent 基本通过 feature + contracts；auth 路径仍直接走 host 内联实现，未经 feature-auth/adapter-google 边界。 |

**Score:** 2/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `packages/adapters/auth/google/src/lib.rs` | Google OAuth adapter | ⚠️ ORPHANED | 文件充分实现（529行），但未被 host auth command 接线使用。 |
| `packages/features/auth/src/lib.rs` | AuthService 边界 | ⚠️ ORPHANED | trait 存在，但无实现注入、无 runtime 调用。 |
| `packages/core/usecases/src/counter_service.rs` | Counter LibSQL 实现 | ✓ VERIFIED | `execute/query` + migration 常量均存在。 |
| `packages/core/usecases/src/admin_service.rs` | Admin 聚合实现 | ✓ VERIFIED | 使用 `tenant_service.list_tenants` + `counter_service.get_value`。 |
| `packages/core/usecases/src/agent_service.rs` | Agent service + 流式 | ⚠️ HOLLOW | SSE 文本流可用，但工具调用未执行，功能操作链路缺失。 |
| `servers/api/src/routes/agent.rs` | Agent routes | ✓ VERIFIED | `/agent/conversations`、`/agent/conversations/:id/messages`、`/agent/chat` 均存在。 |
| `apps/client/web/app/src/routes/(app)/settings/+page.svelte` | API key 设置页 | ✓ VERIFIED | Tauri Store 读写 `api_key/base_url/model`。 |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `runtime_tauri commands/auth.rs` | `adapter-google` | auth adapter delegation | ✗ NOT_WIRED | `rtk rg -n "GoogleAuthAdapter|adapter-google" packages/adapters/hosts/tauri/src/commands/auth.rs` 无匹配。 |
| `runtime_tauri commands/auth.rs` | `feature-auth` | AuthService trait boundary | ✗ NOT_WIRED | `rtk rg -n "feature_auth::AuthService" .../auth.rs` 无匹配。 |
| `counter/+page.svelte` | `tauri invoke + /api/counter/*` | dual-path runtime detection | ✓ WIRED | `window.__TAURI__` 分支 invoke，browser 分支 fetch `/api/counter/...`。 |
| `admin/+page.svelte` | `admin command + /api/admin/stats` | runtime split | ✓ WIRED | Tauri invoke `admin_get_dashboard_stats`，browser fetch `/api/admin/stats`。 |
| `agent/+page.svelte` | `POST /api/agent/chat` | streaming fetch reader | ✓ WIRED | 逐块解析 `data:` 行并拼接 assistant 回复。 |
| `agent_service.rs` | product tools execution | AVAILABLE_TOOLS + business calls | ✗ PARTIAL | 仅构造 tools JSON；未执行 `get_counter_value/list_tenants/get_system_status`。 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `counter/+page.svelte` | `count` | invoke/fetch → `counter` routes/commands → `LibSqlCounterService` | Yes (`SELECT/UPDATE/INSERT` on `counter`) | ✓ FLOWING |
| `admin/+page.svelte` | `stats` | invoke/fetch → admin route/command → `LibSqlAdminService` | Yes (`list_tenants` + `counter.get_value`) | ✓ FLOWING |
| `agent/+page.svelte` | `messages` | `/agent/conversations/:id/messages` + `/agent/chat` SSE | Yes (conversation/message DB + OpenAI stream) | ✓ FLOWING |
| `agent_service.rs` tools path | `tool_calls` / tools ops | `AVAILABLE_TOOLS` only | No (declared only, not executed) | ⚠️ STATIC |
| auth runtime path | session state | host auth command local logic | Yes (store-based) but bypasses planned adapter path | ⚠️ BYPASSES_BOUNDARY |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Rust phase crates compile | `rtk cargo check -p runtime_tauri -p runtime_server -p feature-agent -p feature-auth` | Finished dev profile successfully | ✓ PASS |
| Frontend Svelte health | `rtk npm --prefix "apps/client/web/app" run check` | `svelte-check found 0 errors and 0 warnings` | ✓ PASS |
| Agent contracts export test | `rtk cargo test -p contracts_api export_agent_config` | `1 passed` | ✓ PASS |
| Auth adapter actually wired | `rtk rg -n "GoogleAuthAdapter|feature_auth::AuthService" packages/adapters/hosts/tauri/src/commands/auth.rs` | no matches | ✗ FAIL |
| Agent tools executable wiring | `rtk rg -n "get_counter_value|list_tenants|get_system_status" packages/core/usecases/src/agent_service.rs` | only tool declaration matches, no execution call sites | ✗ FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| AUTH-01 | 04-01, 04-03 | Google 登录，auth 通过 adapter 接入不污染 core | ✗ BLOCKED | 登录链路可用，但仍在 `runtime_tauri/commands/auth.rs` 内联，未接入 `adapter-google`/`feature-auth`。 |
| COUNTER-01 | 04-02, 04-03 | 计数器功能可用并验证前后端通信 | ✓ SATISFIED | usecase + tauri command + axum route + svelte page 完整闭环。 |
| ADMIN-01 | 04-02, 04-03 | 管理后台可访问且有基本统计卡片 | ✓ SATISFIED | `admin/+page.svelte` 卡片渲染，后端真实聚合数据来源。 |
| AGENT-01 | 04-04 | 通过 API key 与 agent 对话 | ✓ SATISFIED | 设置页持久化 API key，chat 页可发消息并接收 SSE 流式响应。 |

**Orphaned requirements check:** Phase 4 在 REQUIREMENTS.md 映射的 4 个 ID（AUTH-01/COUNTER-01/ADMIN-01/AGENT-01）均已在至少一个 Plan frontmatter 中声明；无 orphaned requirement。

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| `apps/client/web/app/src/routes/+layout.svelte` | 35,40 | `console.log` | ⚠️ Warning | 生产噪声日志；不阻断 Phase 4 核心功能，但建议清理。 |
| `packages/adapters/hosts/tauri/src/commands/auth.rs` | whole file | duplicated OAuth implementation | 🛑 Blocker | 与 adapter-google 双实现并存，违反本 phase auth adapter 接入目标。 |
| `packages/core/usecases/src/agent_service.rs` | tools section | tool list without execution path | 🛑 Blocker (for success criterion #4) | agent 无法“操作产品功能”。 |

### Human Verification Required

### 1. Google OAuth 实机登录回路

**Test:** 在 Tauri 运行环境点击 “Sign in with Google”，完成浏览器授权并回到应用。  
**Expected:** 登录成功后进入 `/counter`，session 可在刷新后保持；过期后触发 auth guard。  
**Why human:** 依赖外部 Google OAuth 与本地回调端口，当前验证未启动 UI/外部服务。

### 2. Agent 对话真实外部模型连通

**Test:** 在 Settings 填入有效 OpenAI-compatible 配置，在 Agent 页发送消息并观察流式回复。  
**Expected:** 可持续流式返回；错误配置时有可理解反馈。  
**Why human:** 依赖外部模型服务与真实 API key，不在无副作用自动验证范围内。

### Gaps Summary

Phase 4 的 **Counter/Admin/基础 Agent 对话** 已达到可运行状态，但整体目标仍被 3 个关键缺口阻断：

1. **AUTH-01 未达成“adapter 接入”**：auth 仍是 host 内联实现，`adapter-google` 与 `feature-auth` 未形成真实调用链。  
2. **Agent “操作产品功能”未落地**：仅向模型声明 tools，未执行工具调用并回写结果。  
3. **“所有功能都通过 feature 模块实现”未成立**：auth 仍绕过 feature 层。

#### Reproducible checks for gaps

- Auth adapter 未接线：
  - `rtk rg -n "GoogleAuthAdapter|feature_auth::AuthService" packages/adapters/hosts/tauri/src/commands/auth.rs`
- Auth 仍内联：
  - `rtk rg -n "pub async fn start_oauth|pub async fn handle_oauth_callback|pub fn get_session" packages/adapters/hosts/tauri/src/commands/auth.rs`
- Agent tools 未执行：
  - `rtk rg -n "AVAILABLE_TOOLS|get_counter_value|list_tenants|get_system_status" packages/core/usecases/src/agent_service.rs`
  - `rtk rg -n "counter_service|tenant_service|health_check|tool_call" packages/core/usecases/src/agent_service.rs`

---

_Verified: 2026-04-02T15:14:29Z_  
_Verifier: the agent (gsd-verifier)_
