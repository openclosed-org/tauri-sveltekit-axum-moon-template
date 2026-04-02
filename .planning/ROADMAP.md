# ROADMAP: Tauri-SvelteKit-Axum Boilerplate

**Generated:** 2026-04-01
**Milestone:** v0.2.0
**Granularity:** fine
**Phase numbering:** Reset to Phase 1 (per --reset-phase-numbers)
**Total v0.2.0 Requirements:** 12

## Phases

- [ ] **Phase 1: 仓库目录结构与工具链对齐** — 对齐蓝图目录结构，配置 moon/Just/proto 统一入口 ✓
- [ ] **Phase 2: Contracts/typegen 单一真理源** — 建立 packages/contracts 作为 Rust→TS 自动生成源，CI drift 检查
- [ ] **Phase 3: Runtime 边界收敛** — core vs adapters vs hosts 职责清晰化，新能力走新路径
- [ ] **Phase 4: 最小功能实现** — Google Auth, Counter, Admin Web, Agent 对话通过 feature + adapter 模式实现
- [ ] **Phase 5: Agent-Friendly 开发基建** — AGENTS.md, skills, playbooks, rubrics, eval suites

## Phase Details

### Phase 1: 仓库目录结构与工具链对齐
**Goal**: 仓库目录结构对齐 agent-native-starter-v1 蓝图，开发者可通过单条命令启动和验证。
**Depends on**: Existing v0.1.0 scaffold
**Requirements**: STRUCT-01, TOOL-01
**Success Criteria** (what must be TRUE):
  1. 仓库目录结构按蓝图分层（apps/servers/packages/crates/tools/.agents），目录边界红线文档化。
  2. moon 任务图包含 repo:setup, repo:dev-fullstack, repo:typegen, repo:verify 入口。
  3. Just 提供 just setup, just dev, just verify, just typegen 顶层命令。
  4. proto 管理 Rust/Bun 工具链版本，.prototools 就位。
  5. 新 agent 能在一小时内通过 AGENTS.md 和任务图安全开始工作。
**Plans**: 4 plans in 3 waves
Plans:
- [x] 01-01-PLAN.md — Directory scaffold & migration & .prototools
- [x] 01-02-PLAN.md — Moon task graph & workspace config
- [x] 01-03-PLAN.md — Justfile rewrite
- [x] 01-04-PLAN.md — Integration verification (checkpoint)

### Phase 2: Contracts/typegen 单一真理源
**Goal**: Rust 和 TypeScript 共享一个契约真相源，不能无声漂移。
**Depends on**: Phase 1
**Requirements**: CONTRACT-01, CONTRACT-02
**Success Criteria** (what must be TRUE):
  1. packages/contracts/api 定义跨边界共享 DTO，不再有手写 mirror types。
  2. typegen 从 Rust contracts 自动生成 TS 类型，输出到 frontend/generated/。
  3. CI/verify 在 typegen 后有未提交 diff 时失败。
  4. 前端和 server 路由引用 generated types，不再手写接口定义。
**Plans**: 2 plans in 2 waves
Plans:
- [x] 02-01-PLAN.md — Contracts crates setup + typegen pipeline
- [x] 02-02-PLAN.md — Server migration + frontend integration + drift check

### Phase 3: Runtime 边界收敛
**Goal**: 核心业务规则不依赖宿主，adapters 只做外部世界翻译，新增 capability 走新路径。
**Depends on**: Phase 2
**Requirements**: RUNTIME-01, RUNTIME-02, RUNTIME-03
**Success Criteria** (what must be TRUE):
  1. core/domain 不依赖任何 host/protocol/chain crate。
  2. adapters/hosts/tauri (runtime_tauri) 承载 Tauri command 桥接，native host 仅保留 builder/bootstrap。
  3. 新增 capability 通过 feature 模块组合实现，不绕过 contracts/adapters 边界。
  4. 目录边界红线有 CI 或 agent rubric 强制检查。
**Plans**: 4 plans in 3 waves
Plans:
- [x] 03-01-PLAN.md — Port implementations migration to packages/adapters/storage/
- [x] 03-02-PLAN.md — usecases decoupling from contracts_api
- [x] 03-03-PLAN.md — runtime_tauri command bridge + native-tauri refactor
- [x] 03-04-PLAN.md — Boundary enforcement (deny.toml, CI check, agent rubric)

### Phase 4: 最小功能实现
**Goal**: 用户可以登录、使用计数器、访问 Admin、与 Agent 对话。
**Depends on**: Phase 3
**Requirements**: AUTH-01, COUNTER-01, ADMIN-01, AGENT-01
**Success Criteria** (what must be TRUE):
  1. 用户可以通过 Google 账号登录，auth adapter 不污染 core。
  2. 用户可以使用计数器（increment/decrement/reset），前后端通信正常。
  3. 用户可以访问管理后台，包含基本统计卡片。
  4. 用户可以通过 API key 与 agent 对话，agent 可以操作产品功能。
  5. 所有功能通过 feature 模块实现，使用 contracts 类型。
**Plans**: 4 plans in 4 waves
Plans:
- [x] 04-01-PLAN.md — Auth refactor: adapter-google + feature-auth crates (Wave 1)
- [ ] 04-02-PLAN.md — Counter + Admin features: traits + LibSQL implementations (Wave 2)
- [ ] 04-03-PLAN.md — Counter + Admin wiring: Tauri commands + Axum routes + frontend (Wave 3)
- [ ] 04-04-PLAN.md — Agent chat: contracts + service + streaming + UI (Wave 4)

### Phase 5: Agent-Friendly 开发基建
**Goal**: Agent 能读取规则、发现工具、执行任务、验证结果。
**Depends on**: Phase 4
**Requirements**: AGENT-DEV-01
**Success Criteria** (what must be TRUE):
  1. .agents/ 目录包含 skills (rust-core, tauri-host, sveltekit-ui, contracts-typegen, testing)。
  2. .agents/prompts/ 包含 add-feature, add-host, refactor-boundary 等标准 prompt。
  3. .agents/playbooks/ 包含 create-feature, update-contracts 等多步任务规范。
  4. .agents/rubrics/ 包含 code-review, boundary-compliance, task-completion 评估标准。
  5. 新 agent 一小时内能通过规则矩阵和 playbook 安全开始工作。
**Plans**: TBD

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. 仓库目录结构与工具链对齐 | 4/4 | Complete | 2026-04-01 |
| 2. Contracts/typegen 单一真理源 | 1/2 | In progress | - |
| 3. Runtime 边界收敛 | 0/4 | Planned | - |
| 4. 最小功能实现 | 1/4 | In Progress|  |
| 5. Agent-Friendly 开发基建 | 0/TBD | Not started | - |

## Coverage Map (v0.2.0)

| Requirement | Phase | Status |
|-------------|-------|--------|
| STRUCT-01 | Phase 1 | Completed (Plans 01-02) |
| TOOL-01 | Phase 1 | Completed (Plans 02-03) |
| CONTRACT-01 | Phase 2 | Plan 1/2 complete |
| CONTRACT-02 | Phase 2 | Plan 1/2 complete |
| RUNTIME-01 | Phase 3 | Pending |
| RUNTIME-02 | Phase 3 | Pending |
| RUNTIME-03 | Phase 3 | Pending |
| AUTH-01 | Phase 4 | Pending |
| COUNTER-01 | Phase 4 | Pending |
| ADMIN-01 | Phase 4 | Pending |
| AGENT-01 | Phase 4 | Pending |
| AGENT-DEV-01 | Phase 5 | Pending |

**Coverage: 12/12 v0.2.0 requirements mapped ✓**

## Prior Milestone History

- v0.1.0: Phases 01-10 (archived at .planning/milestones/v0.1.0-phases/)
  - Package foundation, UI styling, app pages, backend deps, database, auth, multi-tenancy, desktop features, build pipeline, test suite

---

*Roadmap created: 2026-04-01*
*Ready for phase planning: `/gsd-plan-phase 1`*
