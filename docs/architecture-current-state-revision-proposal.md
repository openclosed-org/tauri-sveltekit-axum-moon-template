# Architecture Current-State Revision Proposal

> 目的：为 `docs/ARCHITECTURE.md` 提供一份可直接回写的修订建议稿。
> 目标：不是降低最终态标准，而是让文档同时满足两件事：
> 1. 保留最终态方向
> 2. 不再与当前仓库现状明显冲突

## 1. 建议先调整文档定位

当前 `docs/ARCHITECTURE.md` 的最大问题，不是“方向错”，而是“读者很容易把它当成现状说明”。

建议在文档开头显式加入以下定位说明：

```md
> 本文档描述仓库的目标最终态骨架与硬约束，不等于当前仓库现状快照。
> 当前仓库处于迁移期：平台模型、workers、verification 等骨架已较完整落地，
> 但 services、packages 分层、SDK 消费面、部分 app/server 目录仍在收敛中。
> 如需查看现状，请同时阅读：
> - docs/architecture/repo-layout.md
> - docs/architecture-gap-priority-plan.md
```

这样做的好处：

- 保住文档的“最终态约束”价值
- 避免读者把目标骨架误当作现状目录树
- 减少“文档写了但仓库没有”的误解成本

## 2. 建议修改根目录树说明方式

当前文档第 1 节直接给出完整最终态目录树，容易让读者误判“当前应该已经完全长这样”。

建议把根目录树拆成两层：

1. 目标最终态目录树
2. 当前仓库的关键偏差说明

建议在最终态目录树之后追加一个“当前仓库偏差注记”小节。

### 建议新增的小节示例

```md
## 当前仓库偏差注记

当前仓库相对最终态存在以下重要偏差：

- 存在 `justfiles/` 与 `scripts/` 两个根级辅助目录，分别承担任务模块和自动化脚本职责
- `packages/` 仍保留 `core/`、`features/`、`shared/` 等迁移中过渡层
- `services/` 采用 `*-service` 命名，且部分业务仍在从 `packages/core/usecases/*` 迁移
- `servers/` 当前同时存在 `api/` 与 `bff/*`，说明同步入口尚未完全收敛到最终拓扑
- `packages/sdk/` 仍偏占位，前端当前主要消费 app-local generated types
- `apps/mobile/` 尚未落地
```

## 3. 建议更新“根层”规则描述

文档当前根层规则整体方向没问题，但与仓库现状存在两个冲突：

1. 当前真实入口不仅有 `justfile`，还有 `justfiles/`
2. 当前真实自动化层还有 `scripts/`

### 建议把原规则

原表述：

- 所有开发命令从 `justfile` 暴露
- 根目录不出现临时脚本、业务配置、环境差异逻辑

### 建议改成

```md
- 所有稳定的开发/验证/运维入口必须从根 `Justfile` 暴露。
- `justfiles/` 可作为 `Justfile` 的模块化实现层。
- `scripts/` 可承载跨平台自动化脚本，但必须通过 `just` 或 `moon` 暴露，避免隐式入口。
- 根目录禁止散落无人维护的临时脚本；允许存在受治理的根级脚本目录。
```

这样更贴合当前仓库，也更适合真实工程演进。

## 4. 建议更新 `apps/` 章节

文档当前写法：

- API 调用只走 `packages/sdk/*` 生成产物

这在当前仓库里并不成立。当前 Web 侧实际用的是应用内生成代码，例如：

- `apps/web/src/routes/(app)/agent/+page.svelte`
- `apps/web/src/lib/ipc/agent.ts`

都在引用 `$lib/generated/api/*`。

### 建议改成分阶段表述

```md
- 目标最终态：应用层 API 消费统一走 `packages/sdk/*`。
- 迁移期允许使用 app-local generated clients/types，但必须有唯一生成入口，并与 contracts 同步。
- 一旦 `packages/sdk/*` 成熟，应用层应逐步收敛到 SDK 真理源。
```

这样不会把当前实现直接判死，也保留最终收敛方向。

## 5. 建议更新 `servers/` 章节

当前文档的 server 目录树与实际不一致：

目标写法：

- `servers/web-bff`
- `servers/admin-bff`
- `servers/edge-gateway`

当前实际：

- `servers/api`
- `servers/bff/web-bff`
- `servers/bff/admin-bff`
- `servers/gateway`
- `servers/indexer`
- `servers/realtime`

### 建议修订 1：接受 BFF 分组目录

建议把目录示例从平铺改成分组式：

```text
servers/
├── api/                 # 迁移期综合 API 入口，可逐步收敛或下沉
├── bff/
│   ├── web-bff/
│   ├── admin-bff/
│   └── mobile-bff/
├── gateway/             # 迁移期命名，可映射到目标 edge-gateway
└── realtime/
```

### 建议修订 2：明确 `api/` 是迁移期角色

建议新增说明：

```md
- `servers/api` 是迁移期可接受的综合同步入口。
- 长期目标是将对外同步协议逐步收敛到更清晰的 BFF / gateway / dedicated server 拓扑。
```

### 建议修订 3：补充 OpenAPI 规则的适用范围

当前只有 `servers/api/openapi.yaml` 明确存在，BFF 侧未见独立 OpenAPI。

建议把规则从：

- 每个 HTTP 服务必须带 `openapi.yaml`

改成：

```md
- 对外暴露稳定 HTTP 契约的 server 必须提供 `openapi.yaml` 或等价的可验证契约资产。
- 迁移期内部 BFF 若仅作为页面聚合层，可暂不强制独立 OpenAPI，但需在文档中明确契约边界。
```

## 6. 建议更新 `services/` 章节

文档当前假定所有业务能力都已稳定落在 `services/*`。
而当前仓库实际是：

- `counter-service`、`user-service`、`auth-service`、`settings-service` 较完整
- `tenant-service`、`agent-service` 明确仍在迁移
- `event-bus` 放在 `services/` 下，但长期更像共享能力层

### 建议新增迁移规则

```md
- 迁移期允许 `services/*` 与历史业务承载层并存，但必须满足：
  1. 新增业务逻辑不得继续进入历史承载层
  2. 每个迁移中 service 必须在 README 中明确状态和剩余迁移范围
  3. 一旦某业务域在 `services/*` 中具备稳定 API，应优先从历史承载层移除对应实现
```

### 建议调整标准结构表述

当前文档把 service 结构写得很硬：

- `domain/`
- `application/`
- `policies/`
- `ports/`
- `events/`
- `contracts/`

建议改为“核心必需 + 推荐扩展”两层：

```md
核心必需：
- `domain/`
- `application/`
- `ports/`
- `contracts/`
- `lib.rs`
- `tests/`
- `migrations/`

推荐扩展：
- `events/`
- `policies/`
- `sync/`
- `infrastructure/` 或等价适配实现层
```

原因：当前仓库已经证明过渡期很难一步统一到最理想目录形态。

## 7. 建议更新 `workers/` 章节

`workers/` 章节方向基本正确，不建议大改，只建议增加一句现状说明：

```md
- 当前仓库已优先落地 worker 骨架；部分 worker 已具备 checkpoint/dedupe/retry 结构，但具体后端接入仍可能使用 memory/stub 实现。
```

这样更符合实际，也不会误导读者把“目录齐了”理解成“生产级完成”。

## 8. 建议重写 `packages/` 章节为“最终层 + 迁移层并存”

这是当前文档与现状差距最大的章节。

当前文档写的是最终能力型分层，但当前仓库实际存在：

- 最终层方向：`kernel/platform/runtime/contracts/sdk/ui/adapters`
- 迁移层：`core/features/shared`

### 建议新增分层说明

```md
当前 `packages/` 处于双层并存阶段：

- 最终目标层：`kernel/`、`platform/`、`runtime/`、`contracts/`、`sdk/`、`ui/`、能力型共享包
- 迁移中过渡层：`core/`、`features/`、`shared/`

迁移规则：
- 新增共享能力优先进入最终目标层
- `core/` 不再接收新增业务 usecase
- `features/` 可在迁移期继续承担 trait/边界定义，但需要避免成为永久业务堆积层
```

### 建议修改 vendor / adapter 规则表达

当前仓库的 `adapters/` 覆盖 host/auth/storage/chains/protocols/telemetry，多于文档示例。
建议保留“vendor 只能进 adapters”原则，但不要把目录结构写得过死。

建议改成：

```md
- 具体中间件、宿主、链、协议、存储实现必须进入 `packages/**/adapters/**` 或等价的 adapter 分组目录。
- 允许仓库根据当前演进阶段采用统一 `packages/adapters/*` 组织，而不要求一次性拆成最终能力型目录。
```

## 9. 建议更新 `infra/` 章节

当前文档偏向“platform model -> rendered manifests -> gitops”理想闭环。
当前仓库实际是多条基础设施路径并存：

- `infra/docker/`
- `infra/k3s/`
- `infra/kubernetes/`
- `infra/gitops/`
- `infra/security/`
- `infra/terraform/`

### 建议新增现状说明

```md
- 当前仓库允许本地 Compose、k3s、Kubernetes、GitOps 多条交付路径并存。
- 目标最终态仍是“平台模型优先、生成产物可再生、GitOps 为主交付面”。
- 在平台模型生成链未完全覆盖之前，允许存在受治理的手工基础设施目录，但必须有清晰边界与迁移目标。
```

这样能解释为什么当前既有 `infra/k3s/base/*`，又有 `infra/kubernetes/addons/*`。

## 10. 建议更新 `verification/` 章节

文档当前强调 `e2e/contract/topology/resilience/performance/golden` 全覆盖。
当前仓库实际是：

- `contract/resilience/golden` 较强
- `e2e` 中等
- `performance` 偏弱
- `topology` 已有 `single-vps`，其他拓扑较弱

### 建议新增覆盖成熟度说明

```md
- 当前验证层允许分阶段成熟：优先保证 contract、generated drift、resilience、golden baseline；
  再逐步补齐 performance 和多拓扑 E2E。
```

这样更符合真实建设顺序。

## 11. 建议新增“文档外但正式存在的顶级辅助层”

当前文档没有很好吸收两个现实存在且重要的顶级目录：

- `scripts/`
- `justfiles/`

建议在根层或附录中新增：

```md
## 顶级辅助层

- `justfiles/`：根 `Justfile` 的模块化任务实现层
- `scripts/`：跨平台自动化脚本层，只能通过 `just` / `moon` 暴露稳定入口
```

这能减少“这些目录是不是脏东西”的误解。

## 12. 建议保留但弱化的最终态内容

以下内容建议保留为最终目标，但在文档中明确“可预埋未启用”：

- `apps/mobile/`
- 更完整的 `fixtures/` 领域切分
- `tools/codegen`、`tools/loadtest`、`tools/diagnostics`
- 更丰富的 `platform/model/resources/*`
- 更完整的 topology matrix

建议统一加一句：

```md
以下目录为最终态预留位置，不要求当前仓库在任意阶段全部启用。
未启用时应在文档中明确标注“预留/未启用”，而不是默认视为缺失。
```

## 13. 建议追加“当前实际结构摘要”附录

建议在 `docs/ARCHITECTURE.md` 末尾追加一个简短附录，示例：

```md
## 附录：当前仓库实际结构摘要

当前主要顶级目录：

- `agent/`
- `apps/`：`web/`、`desktop/`、`browser-extension/`
- `servers/`：`api/`、`bff/`、`gateway/`、`indexer/`、`realtime/`
- `services/`：`*-service` 命名为主，部分仍在迁移
- `workers/`
- `packages/`：最终层与迁移层并存
- `platform/`
- `infra/`
- `ops/`
- `verification/`
- `docs/`
- `fixtures/`
- `tools/`
- `justfiles/`
- `scripts/`
```

这能显著降低读者初次进入仓库时的认知断裂。

## 14. 一句话修订策略

`docs/ARCHITECTURE.md` 不应该从“目标最终态文档”退化成“现状流水账”，但必须明确：它描述的是目标约束，而不是当前每个目录都已完全落位的事实。

最合适的改法不是删除最终态，而是补上：

1. 文档定位
2. 迁移期偏差说明
3. 当前实际结构附录
4. 对 `packages/`、`services/`、`servers/`、`apps/` 的分阶段规则表达
