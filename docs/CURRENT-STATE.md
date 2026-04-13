# Current State of Repository

> **本文档是当前仓库的单一真相源（Single Source of Truth）。**
> **Agent 开发前必须先阅读此文档。**
> 最后更新：2026-04-12

---

## 1. 仓库整体状态

**当前阶段**：平台模型骨架已完成，业务层迁移接近尾声，但文档与代码仍有不一致。

**完成度估算**：~85%（从 ~70% 提升）

| 层级 | 完成度 | 一句话说明 |
|-----|-------|-----------|
| `platform/` | ✅ ~95% | schema/model/generators/validators/catalog 已完整落地 |
| `workers/` | ✅ ~90% | 5 个 worker 已建立并集成 runtime ports |
| `services/` | ✅ ~90% | 9 个 service 代码已就位，counter 补齐 policies/events，README 已更新 |
| `servers/` | ✅ ~85% | servers/indexer 已清理（与 workers 冲突消除），api + bff 完整 |
| `packages/` | ⚠️ ~75% | 分层文档已建立，sdk 策略已明确，过渡层待逐步收敛 |
| `apps/` | ⚠️ ~60% | web/desktop/extension 骨架已建；前端消费 app-local generated client |
| `infra/` | ⚠️ ~60% | docker compose/k3s base/gitops/sops 已建立；rendered 产物待生成 |
| `verification/` | ⚠️ ~55% | README 说明完善，e2e/performance 待实现具体测试代码 |
| `docs/` | ✅ ~90% | 8 ADR + C4 架构图 + 运维文档 + 当前状态文档已完整 |
| `fixtures/` | ⚠️ ~40% | 各域 README 已补齐，实际种子数据待填充 |

---

## 2. 已知文档失真清单

以下文档/文件的内容与实际代码不符，Agent 不得以其为依据推断现状：

| 文件 | 问题 | 当前状态 |
|-----|------|---------|
| `services/README.md` | tenant/agent 标记为 ⚠️ 待迁移；chat/admin 标记为 ❌ 待实现 | ✅ 已修正为实际完成状态 |
| `servers/README.md` | gateway 描述为 stub，未提及 bff/* | ✅ 已重写 |
| `packages/core/README.md` | 引用已删除的 usecases/ | ✅ 已更新 |
| `packages/core/state/README.md` | 引用已删除的 usecases/ | ✅ 已更新 |
| `services/tenant-service/README.md` | 引用不存在的 usecases/tenant_service.rs | ✅ 已更新 |
| `services/agent-service/README.md` | 引用不存在的 usecases/agent_service.rs | ✅ 已更新 |

**已完全修复的失真项**：
- ~~`.refactoring-state.yaml` "100% complete"~~ → 已修正为 ~85%
- ~~`bun-workspace.yaml` 仅 3 个包~~ → 已修正为 5 个
- ~~`README.md` 被删除~~ → 已重建
- ~~`servers/indexer` 与 `workers/indexer` 冲突~~ → servers/indexer 已删除
- ~~`packages/core/usecases/` 文档引用~~ → 所有引用已清理

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
├── counter-service/   ✅ domain/application/contracts/ports/infrastructure/interfaces/sync/policies/events + tests + migrations
├── user-service/      ✅ domain/application/ports/infrastructure + events + tests
├── tenant-service/    ✅ domain/application/contracts/ports/infrastructure/events/sync/tests/migrations（已迁移完成）
├── agent-service/     ✅ domain/application/contracts/ports/infrastructure/interfaces/sync/tests/migrations（已迁移完成）
├── chat-service/      ✅ domain/application/contracts/ports/infrastructure/events/tests（已实现）
├── event-bus/         ✅ 骨架（README 存在）
├── admin-service/     ✅ domain/application/ports/infrastructure/tests/migrations（已实现）
├── auth-service/      ✅ domain/application/contracts/ports/infrastructure/tests（已实现）
└── settings-service/  ✅ domain/application/contracts/ports/infrastructure/interfaces/sync/tests/migrations（已实现）
```

**剩余已知问题**：
- tenant-service 存在 `surrealdb_adapter.rs` 冗余实现（暂保留，待确认是否保留）

### 3.4 `servers/` — ⚠️ 部分占位，职责冲突

```
servers/
├── api/              ✅ routes(admin/agent/counter/settings/user) + adapters + state + openapi.yaml + tests
├── bff/
│   ├── web-bff/      ✅ handlers(user/agent/admin/settings) + middleware(tenant/auth) + routes
│   └── admin-bff/    ✅ handlers(dashboard) + middleware(tenant) + routes(tenant/health/metrics)
└── gateway/          ⚠️ .gitkeep + Cargo.toml（占位）
```

**已清理**：
- ~~servers/indexer~~ → 已删除，职责由 workers/indexer 承担
- ~~servers/realtime~~ → 已删除（空占位）
- ~~servers/bff/mobile-bff~~ → 已删除（空占位）
- BFF OpenAPI 策略已定义在 `servers/bff/OPENAPI-STRATEGY.md`

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
- `packages/core/usecases/` 已删除，文档引用已清理
- `packages/sdk/` 策略已明确：当前前端使用 app-local generated types，SDK 统一方案保留待迁移条件触发
- `packages/LAYERING.md` 已建立，明确各目录职责和去向

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

### P0（已全部完成 ✅）
1. ✅ 修正根 README
2. ✅ 修正 bun-workspace.yaml
3. ✅ 清理 packages/core/usecases 文档引用
4. ✅ 清理 servers/indexer 职责冲突
5. ✅ 建立 CURRENT-STATE.md 为 Agent 必读真相源

### P1（大部分完成）
1. ✅ 统一 services/ 目录结构（counter 补齐 policies/events）
2. ✅ 建立 packages/ 分层文档（LAYERING.md）
3. ✅ 决定 packages/sdk 与 app-local generated 策略（SDK/README.md）
4. ✅ BFF OpenAPI 策略（servers/bff/OPENAPI-STRATEGY.md）
5. ⚠️ packages/ 过渡层（core/domain, features, shared）待逐步收敛（不阻塞开发）

### P2（已建立骨架）
1. ✅ fixtures/ 各域 README 已补齐（users/settings/counter/authz-tuples）
2. ✅ verification/ README 说明完善（e2e/performance/topology）
3. ⚠️ infra/ rendered 产物待生成（由 platform/generators 执行）
4. ⚠️ verification e2e 具体测试代码待实现

---

## 6. 给 Agent 的硬性要求

1. **必须先读本文档**，再读 ARCHITECTURE.md
2. **不得凭记忆或过时的 README 推断现状**
3. **新增代码前**，用 `list_directory` 或 `read_file` 确认实际文件存在
4. **修改文档前**，确认对应代码是否真的按文档描述的方式存在
5. **遇到状态不一致**，以文件系统实际内容为准，不以 Git 历史或不存在的文件为准
