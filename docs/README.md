# Docs Index

> 目标：把 `docs/` 收敛成 agent 与开发者都能快速进入后端开发轨道的唯一文档入口。
>
> 本仓库当前只关注后端默认学习路径，不把 `apps/**` 作为默认上下文。

## 1. 默认原则

这套文档体系遵循以下原则：

1. 默认只保留 A 类与 B 类文档。
2. A 类负责仓库级目标态、规则、边界、真理源。
3. B 类负责局部 owner 文档与 reference chain。
4. 其余文档如果不能稳定服务 agent 开发，就不应成为默认上下文。
5. 文档与代码冲突时，以代码、schema、validator、gate 为准。
6. `counter-service` 不是最小 demo，而是后端默认参考锚点。
7. 文档精简不能导致 secrets、deploy、GitOps、promotion、runbook 等生产工具链从默认学习路径消失。

## 2. 默认阅读顺序

agent 或开发者进入后端任务时，默认按以下顺序阅读：

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. `docs/architecture/repo-layout.md`
6. `docs/operations/counter-service-reference-chain.md`

如果任务是规划、审计、重构，则继续读：

1. `docs/backend-infrastructure-audit-checklist.md`
2. `docs/counter-service-reference-chain-checklist.md`
3. `docs/document-pruning-ab-checklist.md`
4. `docs/gate-ci-decoupling-checklist.md`
5. `docs/backend-execution-plan.md`

## 3. A 类文档

A 类是仓库级目标态与规范，属于默认必读集。

### 3.1 仓库外层入口

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`

### 3.2 `docs/` 内的 A 类文档

1. `docs/architecture/repo-layout.md`
2. `docs/backend-infrastructure-audit-checklist.md`
3. `docs/counter-service-reference-chain-checklist.md`
4. `docs/document-pruning-ab-checklist.md`
5. `docs/gate-ci-decoupling-checklist.md`
6. `docs/backend-execution-plan.md`

### 3.3 A 类职责

A 类只负责：

1. 目标态边界
2. 真理源分层
3. 变更顺序
4. 后端 admission 轨道
5. 文档保留/删除规则
6. gate / CI / tooling 解耦规则
7. 当前后端分阶段执行边界

A 类不负责：

1. 局部目录细节
2. 手把手运行教程
3. 宏大但不可执行的架构散文

## 4. B 类文档

B 类是局部 owner 文档与 reference chain，不重复仓库级规范。

当前默认保留的 B 类文档：

1. `docs/operations/counter-service-reference-chain.md`
2. `packages/contracts/STRUCTURE.md`
3. `platform/model/README.md` 或其收敛后的替代物
4. `services/*/README.md`
5. `workers/*/README.md`
6. `infra/local/README.md`
7. 必要的 `ops/runbooks/**`
8. 与 counter 直接相关的 secrets / deploy / GitOps / topology 局部文档

### 4.1 B 类职责

B 类只负责：

1. 某个局部目录负责什么
2. 当前状态是 `reference`、`implemented`、`stub` 还是 `planned`
3. 入口文件或入口路径
4. 如何验证
5. 不该做什么
6. 如果是生产工具链文档，它如何挂接到 `counter-service` 参考链路

## 5. `counter-service` 的特殊地位

`counter-service` 是默认后端参考锚点。

它承载的不是单纯业务样例，而是两条并行链：

1. 业务主链：service -> contracts -> server -> outbox -> relay -> projector
2. 工程横切链：platform model -> secrets -> deploy -> GitOps -> promotion -> drift -> runbook

因此：

1. 文档精简后，agent 应该仍然能通过 `counter-service` 学到生产工具链默认路径。
2. 如果某个高价值横切能力还没有挂接到 `counter-service` 参考链路，就不应轻易删除相关文档。
3. 后续新服务应优先复用 counter 的工程模式，而不是绕开它另起炉灶。

## 6. 当前最关键的 5 份文档

如果只允许保留最少文档，当前最关键的是：

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `docs/architecture/repo-layout.md`
4. `docs/operations/counter-service-reference-chain.md`
5. `docs/gate-ci-decoupling-checklist.md`

这 5 份一起定义了：

1. 仓库如何分层
2. agent 如何路由与遵守边界
3. 后端默认参考链在哪里
4. gate / CI / 工具链应如何收口

## 7. 非默认上下文

以下内容当前不应作为 agent 默认学习上下文：

1. 大量 C4/系统上下文/容器/时序图文档
2. 宏观愿景型架构散文
3. 手工维护的大型 API 参考文档
4. 与当前代码脱节的未来态说明
5. 单独存在、但没有挂接到 `counter-service` 参考链路的生产工具链说明

这些内容不是一定无价值，而是：

1. 它们不应抢占默认入口。
2. 如果保留，应按需阅读、逐步归档或重写。

## 8. 后续重构方向

当前 `docs/` 目录的后续方向很明确：

1. 继续瘦身 A 类文档，去重并修复失效引用。
2. 继续强化 `counter-service` 参考链路，尤其是 secrets / GitOps / promotion / runbook 覆盖。
3. 将高漂移、低价值、非默认文档迁移到 `docs/archive/`。
4. 让更多规则下沉到 schema、validator、scripts、gate，而不是继续膨胀散文。

当前已经迁入 archive 的内容包括：

1. `docs/archive/architecture/**` 下的历史性 C4 / topology / sequence / deployment 文档。

## 9. 一句话结论

这份 `docs/README.md` 的作用不是列出所有文档，而是明确告诉后续 agent 和开发者：

1. 默认该读什么
2. 不默认读什么
3. 后端参考锚点在哪里
4. 文档精简后如何仍然保住生产级工程链路的默认可学习性
