# Phase 02: Contracts/typegen 单一真理源 - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Rust 和 TypeScript 共享一个契约真相源，不能无声漂移。建立 packages/contracts 作为 Rust→TS 自动生成源（ts-rs），CI drift 检查。蓝图对齐的 contracts 分层（api/auth/events）。

本阶段只做 contracts 定义 + typegen 管线 + drift 检查。不实现业务功能（Phase 4）。

</domain>

<decisions>
## Implementation Decisions

### Typegen 工具链
- **D-01:** 使用 ts-rs 作为 Rust→TS 代码生成库。成熟、serde 兼容、简单 derive 宏。
- **D-02:** 所有跨边界 DTO 必须 `#[derive(TS)]` + `#[ts(export)]`。

### Contract DTO 范围
- **D-03:** Blueprint 对齐 — 按 concern 分三个 crate/module:
  - `packages/contracts/api` — 路由级 DTO（request/response），如 HealthResponse, InitTenantRequest/Response
  - `packages/contracts/auth` — 认证相关类型（token, session, OAuth payload）
  - `packages/contracts/events` — 领域事件载荷类型
- **D-04:** Phase 2 完成 api + auth + events 三个模块的骨架和初始 DTO 定义。不等 Phase 4 再补。

### TS 输出与集成
- **D-05:** ts-rs 输出到 `packages/contracts/generated/`（按 crate 分子目录: api/, auth/, events/）
- **D-06:** 前端通过 `$lib/generated/` 路径别名导入（symlink 或 copy step 从 packages/contracts/generated/ 同步）
- **D-07:** Server 路由从 contracts_api crate 导入 DTO，不再使用内联定义（现有的 InitTenantRequest/Response 等需迁移）

### Drift 检查
- **D-08:** repo:contracts-check 实现双保险：
  1. 运行 typegen 后 `git diff --exit-code` 检查 generated 文件是否有未提交变更
  2. Checksum 校验 generated 文件完整性
- **D-09:** repo:contracts-check 既集成到 repo:verify 中，也可独立调用 (`moon run repo:contracts-check`)

### Agent's Discretion
- ts-rs 在 Cargo.toml 中的具体版本选择
- packages/contracts/api vs auth vs events 是独立 crate 还是 workspace member 下的 module
- `$lib/generated/` 的同步机制是 symlink、post-typegen copy、还是 vite alias
- utoipa 和 ts-rs 是否共存（utoipa 用于 OpenAPI，ts-rs 用于 TS 类型）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 2 — Phase goal, success criteria, dependencies
- `.planning/REQUIREMENTS.md` §CONTRACT-01, §CONTRACT-02 — Acceptance criteria

### Stack & Architecture
- `.planning/PROJECT.md` §Tech stack — Frontend stack overview
- `.planning/PHASE-01-CONTEXT.md` — Prior decisions (moon tasks, Justfile, Cargo workspace)
- `docs/ARCHITECTURE.md` — Layer boundaries, crates/shared_contracts mentioned

### Existing Code (migration targets)
- `servers/api/src/routes/tenant.rs` — InitTenantRequest, InitTenantResponse (inline DTOs to migrate)
- `servers/api/src/routes/health.rs` — HealthResponse (inline DTO to migrate)
- `servers/api/src/lib.rs` — utoipa OpenAPI schemas (coexistence question)

### Toolchain
- `Cargo.toml` — workspace members, contracts_api already a member
- `moon.yml` — repo:typegen and repo:contracts-check stubs to implement
- `Justfile` — `just typegen` already wired to moon

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `packages/contracts/api/` — 已存在，Cargo.toml 基本为空壳，lib.rs 只有 placeholder 注释
- `Cargo.toml` workspace — contracts_api 已作为 member 和 workspace dependency
- `servers/api/src/routes/tenant.rs` — InitTenantRequest/Response 可直接迁移为 contracts 模板
- utoipa 已安装并使用 — ts-rs 可与 utoipa 共存（struct 上同时 derive）

### Established Patterns
- Cargo workspace 使用 path dependencies
- moon 任务使用 command + inputs + deps
- Serde derive 用于所有序列化类型

### Integration Points
- `moon.yml` — repo:typegen stub 需替换为实际 typegen 命令
- `moon.yml` — repo:contracts-check stub 需实现 drift 检查
- `apps/client/web/app/src/lib/` — 需创建 generated/ 目录或 symlink
- `servers/api/src/routes/` — 现有 DTO 定义需迁移至 contracts

### Gaps
- ts-rs 未安装 — 需添加到 Cargo.toml
- packages/contracts/auth/ 和 events/ 仅有 .gitkeep — 需创建 Cargo.toml + src/lib.rs 或作为 api crate 的 module
- 无 generated/ 目录 — 需创建输出目录
- 前端无 $lib/generated — 需创建目录或配置 alias

</code_context>

<specifics>
## Specific Ideas

- ts-rs 是 2026 年 Rust→TS 最成熟的选择，社区活跃，文档完善
- utoipa 和 ts-rs 可以共存：同一个 struct 同时 derive ToSchema 和 TS，分别服务 OpenAPI 和 TS 类型生成
- 生成的 TS 类型应该是 "boring" 的 — 只是类型镜像，不包含业务逻辑
- CI drift 检查是这个 phase 的核心价值 — 没有它，contracts 就退化为手写 mirror

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-contracts-typegen*
*Context gathered: 2026-04-01*
