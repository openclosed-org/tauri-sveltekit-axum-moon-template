# Current State of Repository

> **本文档是当前仓库的单一真相源（Single Source of Truth）。**
> **Agent 开发前必须先阅读此文档。**
> 最后更新：2026-04-12

---

## 1. 仓库整体状态

**当前阶段**：平台模型骨架已完成，业务层迁移接近尾声，但文档与代码仍有不一致。

**完成度估算**：~70%

| 层级 | 完成度 | 一句话说明 |
|-----|-------|-----------|
| `platform/` | ✅ ~95% | schema/model/generators/validators/catalog 已完整落地 |
| `workers/` | ✅ ~90% | 5 个 worker 已建立并集成 runtime ports |
| `services/` | ⚠️ ~80% | 9 个 service 代码已就位，但部分 README 状态失真、结构未完全统一 |
| `servers/` | ⚠️ ~70% | api + web-bff + admin-bff 有真实实现；gateway/realtime/mobile-bff 仍占位；indexer 职责冲突 |
| `packages/` | ⚠️ ~65% | kernel/platform/runtime/contracts 已就位；core/features/shared 为过渡层；sdk/ 为空 |
| `apps/` | ⚠️ ~60% | web/desktop/extension 骨架已建；前端消费 app-local generated client 而非 packages/sdk |
| `infra/` | ⚠️ ~60% | docker compose/k3s base/gitops/sops 已建立；rendered 产物待生成 |
| `verification/` | ⚠️ ~50% | contract/resilience/golden 有基础；e2e/performance/topology 覆盖不足 |
| `docs/` | ✅ ~85% | 8 ADR + C4 架构图 + 运维文档已完整；但部分 README 未及时更新 |

---

## 2. 已知文档失真清单

以下文档/文件的内容与实际代码不符，Agent 不得以其为依据推断现状：

| 文件 | 问题 | 实际状态 |
|-----|------|---------|
| `.refactoring-state.yaml` | 声称 "100% complete" | 实际 ~70%，多项已知 gap 未收敛 |
| `services/README.md` | tenant/agent 标记为 ⚠️ 待迁移；chat/admin 标记为 ❌ 待实现 | 均已实现并有测试 |
| `servers/README.md` | gateway 描述为 stub，未提及 bff/* | 已重写（见本 commit） |
| `packages/README.md` | 描述旧结构 (core/features/shared/adapters) | 仍部分有效，但需明确过渡层 |
| `bun-workspace.yaml` | 仅列 3 个包 | 已修复为 5 个包 |
| `README.md`（根） | 在 cee4ee5 中被删除 | 已重建为当前状态总览 |

---

## 3. 各目录实际状态详情

### 3.1 `platform/` — ✅ 基本完整

```
platform/
├── schema/          ✅ 6 个 JSON Schema（service/deployable/resource/workflow/topology/policy）
├── model/           ✅ services(9)/deployables(9)/resources(4)/policies(5)/workflows(3)/topologies(3)/environments(3)
├── generators/      ✅ Rust 生成器（main.rs 生成 catalog/ 下的 .yaml/.md）
├── validators/      ✅ 7 个验证器（model-lint/contract-drift/dependency-graph/topology-check/security-check/observability-check）
└── catalog/         ✅ 生成产物（services/deployables/resources/topology/architecture.generated.*）
```

### 3.2 `workers/` — ✅ 基本完整

```
workers/
├── indexer/          ✅ sources/transforms/sinks/checkpoint + main.rs
├── outbox-relay/     ✅ polling/publish/dedupe/checkpoint + main.rs
├── projector/        ✅ consumers/readmodels/checkpoint + main.rs + error.rs
├── scheduler/        ✅ jobs/dispatch + main.rs
└── sync-reconciler/  ✅ plans/executors/conflict + main.rs
```

**已完成**：全部集成 `packages/runtime/ports/*`（8 个端口）和 memory adapters。

### 3.3 `services/` — ⚠️ 代码完整，文档失真

```
services/
├── counter-service/   ✅ domain/application/contracts/ports/infrastructure/interfaces/sync + tests + migrations
├── user-service/      ✅ domain/application/ports/infrastructure + events + tests
├── tenant-service/    ✅ domain/application/contracts/ports/infrastructure/events/sync/tests/migrations（已迁移完成）
├── agent-service/     ✅ domain/application/contracts/ports/infrastructure/interfaces/sync/tests/migrations（已迁移完成）
├── chat-service/      ✅ domain/application/contracts/ports/infrastructure/events/tests（已实现）
├── event-bus/         ✅ 骨架（README 存在）
├── admin-service/     ✅ domain/application/ports/infrastructure/tests/migrations（已实现）
├── auth-service/      ✅ domain/application/contracts/ports/infrastructure/tests（已实现）
└── settings-service/  ✅ domain/application/contracts/ports/infrastructure/interfaces/sync/tests/migrations（已实现）
```

**已知问题**：
- counter-service 缺少 `policies/` 和 `events/`
- tenant-service 存在 `surrealdb_adapter.rs` 冗余实现
- services/README.md 的状态标记已在本 commit 中修正

### 3.4 `servers/` — ⚠️ 部分占位，职责冲突

```
servers/
├── api/              ✅ routes(admin/agent/counter/settings/user) + adapters + state + openapi.yaml + tests
├── bff/
│   ├── web-bff/      ✅ handlers(user/agent/admin/settings) + middleware(tenant/auth) + routes
│   ├── admin-bff/    ✅ handlers(dashboard) + middleware(tenant) + routes(tenant/health/metrics)
│   └── mobile-bff/   ⚠️ 空目录
├── gateway/          ⚠️ .gitkeep + Cargo.toml（占位）
├── indexer/          ⚠️ lib.rs + sources/transformers/sinks（与 workers/indexer 冲突）
└── realtime/         ⚠️ 空目录（占位）
```

**已知问题**：
- servers/indexer vs workers/indexer 职责冲突，需清理
- BFF 缺少独立 openapi.yaml

### 3.5 `packages/` — ⚠️ 过渡层与最终层并存

```
packages/
├── kernel/           ✅ 核心类型（ids/error/money/pagination/tenancy/time）
├── platform/         ✅ config/health/buildinfo/env/service_meta
├── runtime/          ✅ 8 个 ports + memory adapters（invocation/pubsub/state/workflow/lock/binding/secret/queue）
├── contracts/        ✅ http/events/rpc/jsonschema/error-codes/compat/sdk-gen
├── adapters/         ✅ hosts/tauri（Tauri commands）
├── features/         ✅ settings 等 feature trait
├── core/             ⚠️ 过渡层：domain/ports/state/workspace-hack（仍有部分业务逻辑）
├── shared/           ⚠️ 过渡层：工具函数/错误处理
├── ui/               ✅ Svelte 组件库
└── sdk/              ⚠️ rust/.gitkeep + typescript/.gitkeep（空占位符）
```

**关键冲突**：
- `packages/core/usecases/` 仍被部分 service 的 README 引用为"待迁移源"，实际大部分已迁出
- `packages/sdk/` 为空，前端实际使用 `apps/web/src/lib/generated/api/*`

### 3.6 `apps/` — ⚠️ 骨架已建，消费路径未统一

```
apps/
├── web/              ✅ SvelteKit 应用，含 generated API types（7 个 .ts 文件）
├── desktop/          ✅ Tauri 2 应用，含 Tauri commands 集成
└── browser-extension ✅ 扩展骨架
```

**已知问题**：
- 前端消费 `apps/web/src/lib/generated/api/*` 而非 `packages/sdk/*`
- 缺少 `apps/mobile/`（文档最终态包含，当前未建立）

### 3.7 `infra/` — ⚠️ 基础已建，产物待生成

```
infra/
├── docker/           ✅ compose/core.yaml
├── local/            ✅ compose/seeds/bootstrap
├── k3s/              ✅ base/（RBAC/network-policies）
├── kubernetes/       ✅ addons/（minio/nats/valkey）
├── gitops/           ✅ flux/（apps/infrastructure）
├── security/         ✅ sops/（SETUP + secrets.template）
└── terraform/        ⚠️ 待实现
```

**已知问题**：
- `infra/kubernetes/rendered/` 不存在（应由 platform/generators 生成）

### 3.8 `verification/` — ⚠️ 基础有，覆盖不足

```
verification/
├── contract/         ✅ backward-compat/event-schema/sdk-roundtrip
├── resilience/       ✅ retry/idempotency/outbox/failover
├── golden/           ✅ architecture/deployables/resources/services/topology.generated.*
├── e2e/              ⚠️ READMEs only（demo-counter/multi-tenant/settings/desktop-web-roundtrip）
├── performance/      ⚠️ 空
└── topology/         ⚠️ 仅 single-vps 有基础测试
```

### 3.9 `docs/` — ✅ 文档完整，但部分过时

```
docs/
├── ADR/              ✅ 001-008（8 个架构决策记录）
├── architecture/     ✅ C4 模型（context/container/component/sequence/deployment/topology/sync-flow）
├── platform-model/   （待补）
├── contracts/        ✅ HTTP API/Event schemas/Tauri RPC/Error codes
├── operations/       ✅ local-dev/single-vps/k3s-cluster/gitops/secret-management
├── generated/        ✅ service-catalog/resource-catalog/dependency-graphs
└── refactoring/      ✅ handoffs/phase-tasks/package-migration-plan
```

---

## 4. 禁止依赖的过时信息源

以下文件或位置的内容**不应作为判断依据**：

1. **`.refactoring-state.yaml` 的 `quick_state.overall_progress`** — 显示 "100%" 但实际不是
2. **任何 README 中的 ⚠️/❌ 状态标记** — 以本文件第 3 节为准
3. **CCC 索引**（`.cocoindex_code/`）— 已被删除，不再使用
4. **`docs/ARCHITECTURE.md` 的目录树** — 这是最终态目标，不是当前现状

---

## 5. 下一步收敛优先级

按 `docs/architecture-gap-priority-plan.md` 排序：

### P0（必须优先）
1. ✅ 修正根 README（本 commit 已完成）
2. ✅ 修正 bun-workspace.yaml（本 commit 已完成）
3. 冻结 `packages/core/usecases/*` 新增业务逻辑
4. 清理 `servers/indexer` 职责冲突
5. 建立本文档为 Agent 必读真相源

### P1（下一阶段）
1. 统一 services/ 目录结构（补齐 policies/events）
2. 收敛 packages/ 过渡层（core/features/shared）
3. 决定 packages/sdk 与 app-local generated 的最终方案
4. 补齐 BFF OpenAPI 策略

### P2（后续补齐）
1. fixtures/ 领域覆盖（users/settings/counter/authz-tuples）
2. verification/ 覆盖（e2e/performance/topology）
3. infra/ rendered 产物生成

---

## 6. 给 Agent 的硬性要求

1. **必须先读本文档**，再读 ARCHITECTURE.md
2. **不得凭记忆或过时的 README 推断现状**
3. **新增代码前**，用 `list_directory` 或 `read_file` 确认实际文件存在
4. **修改文档前**，确认对应代码是否真的按文档描述的方式存在
5. **遇到状态不一致**，以文件系统实际内容为准，不以 Git 历史或不存在的文件为准
