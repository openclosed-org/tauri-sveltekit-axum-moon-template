# Architecture Gap Priority Plan

> 目的：把 `docs/ARCHITECTURE.md` 与当前仓库现状之间的差距，整理成可执行的优先级清单。
> 范围：只覆盖本次已核实的仓库现状，不额外推导未验证结论。

## 1. 当前状态摘要

当前仓库不是“未按架构搭建”，而是明显处于“最终态骨架已大量落地，但业务层与包分层仍在迁移中”的混合阶段。

已经较成熟的部分：

- `platform/`：`schema/model/generators/validators/catalog` 已成体系落地
- `workers/`：`indexer/outbox-relay/projector/scheduler/sync-reconciler` 已按独立 worker 建立骨架
- `verification/`：`contract/resilience/golden` 已有实际验证文件
- `docs/`：ADR、架构分视图、operations、refactoring 文档较完整
- `agent/`：约束、模板、checklist 已落地

仍处于过渡态的部分：

- `packages/core/usecases/*` 仍承载真实业务实现
- `services/` 完整度不均，`tenant-service`、`agent-service` 明确未迁完
- `packages/` 仍保留 `core/features/shared` 这套旧/中过渡分层
- `packages/sdk/` 仍基本占位，前端并未以其作为主要消费入口
- `servers/api` 仍是强中心入口，`servers/indexer` 仍保留旧位置残留
- 根层文档与工作区描述失真，尤其 `README.md`、`bun-workspace.yaml`

## 2. 优先级原则

排序遵循以下顺序：

1. 先修“真理源失真”和“会误导开发”的问题
2. 再修“阻碍最终架构收敛”的结构性问题
3. 最后补“文档模板里存在但当前不阻塞演进”的目录和能力

## 3. P0：必须优先收敛

这些项不一定工作量最大，但会持续误导后续开发，或者直接阻塞架构收敛。

### P0-1 根层入口文档失真

问题：

- 根 `README.md` 当前实际写的是 `scripts/` 说明，不是仓库总览
- 新成员无法通过根入口快速理解仓库主骨架和当前阶段

证据：

- `README.md`
- `docs/architecture/repo-layout.md`

为什么优先：

- 这是所有人进入仓库的第一入口
- 当前状态会让人误判 `scripts/` 是仓库核心而不是辅助层

建议动作：

1. 重写根 `README.md` 为仓库总览
2. 把现有内容迁移为 `scripts/README.md` 的补充说明或保留现有 `scripts/README.md`
3. 在根 README 中明确：当前仓库是“最终态骨架 + 迁移中实现”的混合态

验收标准：

- 根 `README.md` 能覆盖顶级目录职责、开发入口、文档入口、当前迁移状态
- 新成员只读根 README 与 `docs/architecture/repo-layout.md` 即可理解仓库骨架

### P0-2 `bun-workspace.yaml` 与实际工作区失真

问题：

- 当前只列出 `apps/web`、`apps/browser-extension`、`packages/ui`
- 与实际 JS/TS 结构不一致，例如 `apps/desktop/tests/e2e` 已有独立 `package.json`

证据：

- `bun-workspace.yaml`
- `apps/desktop/tests/e2e/package.json`

为什么优先：

- 这是根层工作区描述文件
- 会误导工具链、依赖安装、未来脚本和包治理

建议动作：

1. 盘点真实需要纳入 Bun workspace 的目录
2. 明确哪些是独立包、哪些只是应用内部目录
3. 更新 `bun-workspace.yaml` 与根文档说明

验收标准：

- `bun-workspace.yaml` 能解释当前实际前端/测试工作区布局
- 根文档中不再出现“工作区成员可追踪”与现实不符的情况

### P0-3 停止向 `packages/core/usecases/*` 继续累积业务逻辑

问题：

- 当前 `tenant-service`、`agent-service` 的业务实现仍在 `packages/core/usecases/*`
- 这与 `services/*` 作为业务能力主载体的目标直接冲突

证据：

- `services/tenant-service/README.md`
- `services/agent-service/README.md`
- `services/README.md`
- `packages/README.md`

为什么优先：

- 如果继续在 `packages/core/usecases/*` 加逻辑，后续迁移成本会不断增加
- 这会让“service 是库，不是进程”长期停留在口号层

建议动作：

1. 明确仓库级规则：`packages/core/usecases/*` 禁止新增业务功能
2. 所有新增业务必须落在 `services/*`
3. 先迁移 `tenant-service`、`agent-service` 两个已明确标记的过渡模块

验收标准：

- 新增 PR 不再向 `packages/core/usecases/*` 引入新业务逻辑
- `tenant-service`、`agent-service` README 中的“待迁移”状态被消除或缩小

### P0-4 修正文档与代码现状的关键冲突点

问题：

- `docs/ARCHITECTURE.md` 写的是最终态，但没有明确标出“当前不是现状文档”
- `servers/README.md` 仍把 `gateway` 记为 stub，而代码已有真实实现
- `services/README.md` 对部分服务状态也有漂移

证据：

- `docs/ARCHITECTURE.md`
- `servers/README.md`
- `servers/gateway/src/main.rs`
- `services/README.md`

为什么优先：

- 比起“目录还没补齐”，文档失真更容易制造错误决策

建议动作：

1. 给 `docs/ARCHITECTURE.md` 加上“最终态目标文档”定位说明
2. 更新 `servers/README.md`、`services/README.md` 的状态描述
3. 增加一份“当前实际结构 vs 目标结构”的索引文档

验收标准：

- 关键 README 与代码现状一致
- 读者能区分“当前现状”和“目标架构”

## 4. P1：下一阶段重点收敛

这些项属于结构治理重点，价值高，但不必先于 P0 执行。

### P1-1 收敛 `services/` 的标准结构和完成度

问题：

- 当前 `counter-service`、`user-service`、`auth-service`、`settings-service` 较完整
- 但 `tenant-service`、`agent-service` 仍是迁移壳
- 目录结构也未完全统一到文档标准，例如 `policies/` 并不普遍存在

建议动作：

1. 以 `counter-service` 为样板定义 service 最小完成标准
2. 优先收敛 `tenant-service`、`agent-service`
3. 补足服务层统一结构规范，尤其是 `policies/`、`events/` 的最低要求

验收标准：

- 所有核心业务 service 都不再依赖 `packages/core/usecases/*`
- service 目录结构具备可审查的一致性

### P1-2 收敛 `packages/` 的最终分层方向

问题：

- 当前并存 `core/features/shared` 与 `kernel/platform/runtime/contracts/adapters/ui/sdk`
- 文档中的能力型目录尚未落位为主结构

建议动作：

1. 先定义“哪些目录是过渡层，哪些是最终层”
2. 先处理职责最模糊的 `core` 与 `shared`
3. 明确 `event-bus`、数据访问、认证、缓存、消息能力的最终落点

验收标准：

- 每个共享包都有明确职责边界
- 过渡层目录数量逐步减少

### P1-3 让 `packages/sdk/` 成为真实消费面，或正式承认当前替代方案

问题：

- 文档要求前端走 `packages/sdk/*`
- 当前前端实际用的是 `apps/web/src/lib/generated/api/*`

证据：

- `packages/sdk/rust/.gitkeep`
- `packages/sdk/typescript/.gitkeep`
- `apps/web/src/routes/(app)/agent/+page.svelte`
- `apps/web/src/lib/ipc/agent.ts`

建议动作：

两个方案二选一：

1. 真正把生成产物统一迁到 `packages/sdk/*`
2. 或在文档中正式承认当前“app-local generated client”是过渡方案，并给出迁移条件

验收标准：

- 文档与真实前端消费路径一致
- 不再同时出现“SDK 真理源”与“应用内生成产物”两套表述冲突

### P1-4 清理或重命名过渡 server 目录

问题：

- `servers/indexer` 已与 `workers/indexer` 职责冲突
- `servers/api` 仍是强中心入口
- `servers/realtime` 仍为空壳

建议动作：

1. 明确 `servers/indexer` 是废弃、迁移中还是未来另有职责
2. 为 `servers/api` 明确长期定位：保留综合 API，还是逐步让位给 BFF
3. 对空壳目录加状态标记或移除

验收标准：

- `servers/` 中每个目录都有清晰长期职责
- 不再出现 server/worker 双重同名能力造成误解

### P1-5 补齐 BFF 的协议文档策略

问题：

- 当前只看到 `servers/api/openapi.yaml`
- `web-bff`、`admin-bff` 已实装，但未见各自 `openapi.yaml`

建议动作：

1. 决定 BFF 是否需要独立 OpenAPI
2. 若需要，补齐各自协议文档
3. 若不需要，更新架构文档说明“哪些 server 是对外 HTTP 契约面”

验收标准：

- server 层的契约策略清晰、一致、可验证

## 5. P2：补齐最终态但暂不阻塞主线

这些项重要，但不应先于 P0/P1。

### P2-1 补齐 `apps/mobile/` 或正式降级其优先级

问题：

- 文档最终态包含 `apps/mobile/`
- 当前并未落地

建议动作：

1. 如果近阶段无移动端计划，在文档中注明为“预留未启用”
2. 如果有计划，再补目录与最小壳层

### P2-2 补齐 `fixtures/` 的领域覆盖

问题：

- 当前只有 `seeds/`、`sync-scenarios/`、`tenants/`
- 文档期望还有 `users/`、`settings/`、`counter/`、`authz-tuples/`

建议动作：

1. 先补最影响 E2E 的 `users/`、`settings/`、`counter/`
2. 若授权模型进入主线，再补 `authz-tuples/`

### P2-3 补齐 `tools/` 的非 Web3 目录

问题：

- 当前仅有 `tools/web3/`
- 模板中的 `codegen/`、`loadtest/`、`diagnostics/` 仍未落地

建议动作：

1. 不建议先造空目录
2. 待真正出现工具需求时再按文档目标补齐

### P2-4 补齐 `verification/performance` 与多拓扑验证

问题：

- 当前 `contract/resilience/golden` 强于 `performance/e2e/topology`
- `topology/single-vps` 已有，但其他拓扑覆盖较弱

建议动作：

1. 先补 `performance/` 的最小 smoke baseline
2. 再补 `k3s` 与更复杂拓扑验证

## 6. 不建议现在做的事

以下动作暂不建议优先推进：

- 为了匹配文档一次性重命名所有 `*-service`
- 为了匹配模板先创建大量空目录
- 在未统一包职责前，大规模拆分 `packages/`
- 在未明确长期 API 拓扑前，直接删除 `servers/api`

## 7. 推荐执行顺序

建议按以下顺序推进：

1. 修正根 README、状态文档、关键 README 失真
2. 修正 `bun-workspace.yaml` 与工作区描述
3. 冻结 `packages/core/usecases/*` 的新增业务逻辑
4. 迁移 `tenant-service`、`agent-service`
5. 明确 `packages/sdk` 与 app-local generated 的最终方案
6. 清理 `servers/indexer`、定义 `servers/api` 长期定位
7. 再补 fixtures、performance、多端壳层等最终态内容

## 8. 一句话结论

当前最该做的不是“补文档里缺的空目录”，而是先让根层说明、业务承载位置、包分层边界、server/worker 职责这四件事停止继续漂移。
