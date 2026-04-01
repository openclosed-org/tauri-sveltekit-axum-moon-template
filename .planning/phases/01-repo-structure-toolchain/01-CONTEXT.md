# Phase 1: 仓库目录结构与工具链对齐 - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

将仓库目录结构对齐 agent-native-starter-v1 蓝图，配置 moon/Just/proto 统一入口，使新 agent 能通过 AGENTS.md 和任务图安全开始工作。

本阶段只做结构对齐和工具链配置，不做功能实现（Phase 4）。

</domain>

<decisions>
## Implementation Decisions

### 目录骨架策略
- **D-01:** 按蓝图创建全部目录骨架（70+ 子目录），空目录放 `.gitkeep`
- **D-02:** 目标目录结构严格遵循 `docs/blueprints/agent-native-starter-v1/02-repo-structure.md`

需要创建的顶层目录（当前缺失）：
- `workers/`（从 servers/workers/ 迁移）
- `tools/`（scripts/, generators/, mcp/servers/, mcp/clients/, evals/datasets/, evals/graders/, evals/suites/）
- `apps/ops/`（docs-site/, storybook/）
- `apps/client/web/hosts/`（telegram-miniapp/, farcaster-miniapp/, base-app/）
- `apps/client/browser-extension/`（已有但检查完整性）

需要补全的二级目录：
- `packages/core/`：补 state/
- `packages/features/`：补 auth/, profile/, feed/, social-graph/, wallet/, payments/, notifications/, admin/
- `packages/adapters/`：补完整的 hosts/protocols/chains/storage/auth/telemetry 子树
- `packages/contracts/`：补 auth/, events/, errors/, protocols/, ui/, codegen/
- `packages/ui/`：补 icons/, tokens/
- `packages/shared/`：补 env/, testing/
- `.agents/`：补 prompts/, playbooks/, rubrics/

### Moon 任务图设计
- **D-03:** 一次性实现蓝图完整集（约 30 个 repo:* 任务）

必须包含的任务：
- setup: `repo:setup`, `repo:bootstrap`, `repo:doctor`, `repo:toolchain-check`
- dev: `repo:dev-web`, `repo:dev-desktop`, `repo:dev-extension`, `repo:dev-api`, `repo:dev-workers`, `repo:dev-fullstack`
- quality: `repo:fmt`, `repo:lint`, `repo:typecheck`, `repo:contracts-check`, `repo:test-unit`, `repo:test-integration`, `repo:test-e2e`, `repo:test-agent`, `repo:verify`
- codegen: `repo:typegen`, `repo:openapi-gen`, `repo:fixtures-gen`, `repo:icons-gen`, `repo:tokens-gen`
- ops: `repo:trace-open`, `repo:evals-run`, `repo:replay-protocol`, `repo:release-dry-run`, `repo:release-desktop`, `repo:release-web`, `repo:release-server`
- security: `repo:audit-rust`, `repo:audit-bun`, `repo:secrets-scan`, `repo:licenses-check`

命名规范：repo 级任务 `repo:*`，package 级任务 `<project>:*`。

### Justfile 入口重命名
- **D-04:** 直接替换，删除旧命名，按蓝图重写 Justfile
- **D-05:** 顶层入口为 `just setup`, `just dev`, `just verify`, `just test`, `just typegen`, `just release`, `just evals`
- **D-06:** Just 只暴露最常用的稳定入口，不承担复杂编排逻辑（复杂编排由 moon 负责）

### proto 工具链版本
- **D-07:** 创建 `.prototools`，proto 只管理 Bun 和 Node 版本
- **D-08:** Rust 继续由 `rust-toolchain.toml` 管理，proto 不重复管理 Rust
- **D-09:** `.moon/toolchains.yml` 保持 node + rust 双工具链配置

### servers vs workers 分层
- **D-10:** 将 `servers/workers/` 整体迁移到顶层 `workers/`
- **D-11:** 当前 workers 下内容（atproto/chains/farcaster/nostr）都是空占位目录，迁移无风险
- **D-12:** 迁移后更新 Cargo workspace 和 `.moon/workspace.yml`

### the agent's Discretion
- `repo:doctor` 的具体检查项实现细节
- 各个 `repo:*` 任务的内部命令组合（planner 可灵活选择）
- `.gitkeep` 文件的具体放置策略

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 仓库结构
- `docs/blueprints/agent-native-starter-v1/02-repo-structure.md` — 完整目录树蓝图和分层解释
- `docs/blueprints/agent-native-starter-v1/03-toolchain-and-taskgraph.md` — 工具职责划分、任务建议、Just 入口规范

### 工程标准
- `docs/blueprints/agent-native-starter-v1/06-engineering-standards-rust-tauri-svelte.md` — 命名规范、代码风格约定

### 现有配置
- `.moon/workspace.yml` — 当前 moon 项目注册列表
- `Cargo.toml` — Cargo workspace members
- `rust-toolchain.toml` — Rust 工具链版本

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `moon.yml` — 现有 cargo 任务可保留作为基础，扩展 repo:* 任务
- `Justfile` — 现有入口需重写但结构可参考
- `.moon/workspace.yml` — 需要更新项目注册列表（添加 workers/tools 等）
- `Cargo.toml` workspace members — 需要更新（添加 workers/*）

### Established Patterns
- moon 使用 `command` + `inputs` + `deps` 定义任务
- Justfile 使用 `set shell := ["bash", "-cu"]` 保证跨平台
- Cargo workspace 使用 path dependencies

### Integration Points
- `.moon/workspace.yml` — 注册新项目
- `Cargo.toml` — 添加新的 workspace members
- `AGENTS.md` — 已存在，可作为 agent 入口参考

</code_context>

<specifics>
## Specific Ideas

- 目录骨架严格对齐蓝图，不要自行发挥
- moon 任务命名遵循 `repo:*` 一级命名规范
- Just 只做薄入口层，复杂逻辑下沉到 moon

</specifics>

<deferred>
## Deferred Ideas

- workers 下各协议（atproto/farcaster/nostr）的实际实现 — Phase 3+
- tools/evals/ 下评估数据集的实际内容 — Phase 5
- apps/ops/ 下 docs-site/storybook 的实际搭建 — 按需
- .agents/prompts/playbooks/rubrics 的实际内容 — Phase 5

</deferred>

---

*Phase: 01-repo-structure-toolchain*
*Context gathered: 2026-04-01*
