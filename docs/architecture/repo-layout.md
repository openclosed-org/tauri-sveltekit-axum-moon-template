# Repo Layout

> 目标：构建 **Agent-First / Distributed-First / Harness-First** 的 monorepo，
> 让后续多个 agent 在最少隐性上下文下，仍然能沿正确的分布式边界高速填空开发。
>
> 核心原则：**先冻结不变量，再放开实现；先建设 harness，再进入业务并行开发。**

---

## 0. 当前前提

### 0.1 本文档描述的是目标态

本文件描述的是 **重建后的目标态仓库结构**，不是当前代码现状说明。

默认前提：

1. 当前业务代码可重建，迁移成本不是首要约束。
2. 目标不是“先跑起来”，而是“先把 agent 写对的轨道铺好”。
3. 参考样例固定为：`counter`、`tenant`。
4. agent 分工采用专职工种：`planner`、`platform-ops-agent`、`service-agent`、`server-agent`、`worker-agent`、`contract-agent`、`app-shell-agent`。
5. 完全体分布式的关键语义必须成为 schema、model、template、validator 的一等公民。

### 0.2 这份 V1 解决什么

这份 V1 主要解决：

1. 后续不会因“单点思维写法”导致系统性返工。
2. agent 新增模块时天然按正确边界拆分。
3. topology、deployable、worker、workflow、projection 不再混为一谈。
4. 平台模型先于基础设施和实现代码。
5. 未来从单节点到完全体分布式，变化主要落在 model / topology / generator / adapter，而不是业务骨架。

### 0.3 这份 V1 不解决什么

这份 V1 不提前冻结：

1. 所有未来 service 名
2. 所有未来 DTO / event 细节
3. 所有未来数据库表
4. 所有未来 topic / queue 名
5. 所有未来 vendor 选型

它只冻结：**无论业务如何变化，都不应变化的那部分。**

---

## 1. 核心设计原则

### 1.1 分布式状态高于分布式进程

系统是否是“完全体分布式”，不取决于进程有没有拆开，而取决于以下语义是否被提前建模：

1. 状态归谁拥有
2. 写入边界在哪里
3. 如何传播状态变化
4. 如何恢复、回放、补偿
5. query 允许什么一致性
6. 如何处理重复、延迟、乱序
7. 如何做分区、多租户和热点治理

### 1.2 Service 是能力边界，不是部署边界

1. `services/*` 表示业务能力与状态边界。
2. `servers/*` 表示同步入口。
3. `workers/*` 表示异步推进单元。
4. `platform/model/deployables/*` 表示承载单元。
5. `platform/model/topologies/*` 表示承载组合。

### 1.3 平台模型先于实现

任何新增能力，都应先在模型层表达：

1. 属于哪个 service
2. 拥有哪些状态
3. 暴露什么 contracts
4. 如何与其他能力交互
5. 要求什么一致性 / SLO / 恢复语义
6. 在什么 topology 下承载

### 1.4 Service 语义声明归属 service-agent

这是本次重建的关键边界：

1. `platform/model/*` 只保留平台级元数据、全局规则、deployable、workflow、topology、resource、environment。
2. 每个 service 的细粒度分布式语义写在 `services/<name>/model.yaml`。
3. 这样 `service-agent` 可以在自己的写边界内完成完整语义声明，不需要跨 agent 协作到 `platform/model/**`。

### 1.5 默认分布式语义基线

1. 跨进程消息默认按“至少一次投递”思考，幂等不是优化，是基本要求。
2. 长事务默认按 durable workflow 思考，恢复不是补丁，是模型的一部分。
3. 权限与部分查询一致性必须显式选择，不允许隐式默认。
4. 需要稳定身份与稳定存储的单元，不按普通无状态进程思维建模。
5. 观测命名与属性统一收口，不允许每个模块各说各话。

### 1.6 参考样例优先于抽象空谈

1. `counter` 用于最小完整链路与模板校准。
2. `tenant` 用于多租户、多实体、workflow、补偿。
3. 如需继续扩大参考集，应在 `counter`、`tenant` 稳定后再追加。

参考样例不是 demo，它们是后续 agent 的学习材料和模板验证器。

---

## 2. 目录总览

```text
boilerplate/
├── agent/                      # Agent 最小规则层：codemap、routing、gates
├── platform/                   # 平台控制面与真理源
├── apps/                       # 前端 / 客户端壳层
├── servers/                    # 同步入口层
├── services/                   # 业务能力与状态边界层
├── workers/                    # 异步执行与状态推进层
├── packages/                   # 共享抽象、traits、adapters、sdk、tooling
├── infra/                      # 基础设施声明与交付层
├── ops/                        # 运维执行、演练、恢复
├── verification/               # 跨层验证、兼容、回放、golden
├── docs/                       # ADR、架构说明、runbooks、生成文档
├── fixtures/                   # 测试数据、事件样例、authz tuples、demo 数据
├── tools/                      # 本地辅助工具，不进入生产运行时
├── justfile
├── Cargo.toml
├── package.json
├── .mise.toml
└── rust-toolchain.toml
```

---

## 3. 真理源分层

### 3.1 一级真理源

以下目录属于长期真理源：

1. `platform/schema/*`
2. `platform/model/*`
3. `services/*/model.yaml`
4. `packages/contracts/*`
5. `agent/*`
6. `docs/adr/*`

### 3.2 二级实现层

以下目录属于实现层，必须服从真理源：

1. `services/*/src/**`
2. `servers/**`
3. `workers/**`
4. `packages/*`（非 generated）
5. `infra/*`（非 rendered）

### 3.3 生成层

以下目录必须可删可再生：

1. `packages/sdk/*`
2. `platform/catalog/*`
3. `infra/kubernetes/rendered/*`
4. `docs/generated/*`

### 3.4 变更顺序

变更顺序必须遵循：

1. 先改 `platform/schema/*`
2. 再改 `platform/model/*` 与 `services/*/model.yaml`
3. 再改 `packages/contracts/*`
4. 再改 `services/*/src/**`
5. 再改 `servers/*` / `workers/*`
6. 最后改 `infra/*` / `docs/generated/*` / `packages/sdk/*`

---

## 4. `platform/`：平台控制面

```text
platform/
├── schema/
├── model/
│   ├── services/              # 平台级 service 元数据，不放 service 细粒度语义
│   ├── deployables/           # 可部署单元定义
│   ├── resources/             # 外部资源定义
│   ├── workflows/             # 长事务 / durable workflow 定义
│   ├── policies/              # 平台策略定义
│   ├── topologies/            # 承载组合定义
│   ├── environments/          # 环境差异抽象
│   ├── state/                 # 全局状态规则，不放每个 service 的细碎语义文件
│   │   ├── ownership-map.yaml
│   │   ├── consistency-defaults.yaml
│   │   └── idempotency-defaults.yaml
│   ├── partitioning/
│   │   └── defaults.yaml
│   ├── failures/
│   │   └── domains.yaml
│   └── slo/
│       └── defaults.yaml
├── generators/
├── validators/
└── catalog/
```

### 4.1 `platform/model/services/`

只定义平台级元数据，不定义某个 service 的完整业务语义。

每个 service 元数据至少声明：

1. `name`
2. `kind`
3. `criticality`
4. `tenant_scope`
5. `logical_dependencies`
6. `status`

### 4.2 `platform/model/deployables/`

每个 deployable 至少声明：

1. `kind: server | worker | projector | reconciler | relay | runner | gateway`
2. `hosts_services`
3. `runtime_profile`
4. `statefulness: stateless | checkpointed | stateful`
5. `required_identity: ephemeral | stable`
6. `required_storage: none | ephemeral | persistent`
7. `resource_bindings`
8. `scaling_axis`
9. `failure_domain`

### 4.3 `platform/model/workflows/`

定义跨 service 长事务与 durable workflow。

每个 workflow 至少声明：

1. `trigger`
2. `idempotency_key`
3. `timeout`
4. `checkpoint_policy`
5. `steps`
6. `compensation`
7. `recovery`

### 4.4 `platform/model/state/`

只保留全局规则：

1. 全局 owner map
2. 全局 consistency 默认规则
3. 全局 idempotency 默认规则

**禁止**：把每个 service 的具体语义拆散到 `platform/model/state/**` 一堆零散文件中。

---

## 5. `services/`：业务能力与状态边界层

这是本次重建最关键的变化。

```text
services/<name>/
├── model.yaml                 # service-local distributed semantics
├── Cargo.toml
├── src/
│   ├── domain/
│   ├── application/
│   ├── policies/
│   ├── ports/
│   ├── events/
│   ├── contracts/
│   └── lib.rs
├── tests/
├── migrations/
└── README.md
```

### 5.1 `services/<name>/model.yaml` 必须表达什么

每个 service 的 `model.yaml` 至少表达：

1. `owns_entities`
2. `accepted_commands`
3. `published_events`
4. `served_queries`
5. `cross_service_reads`
6. `spec_completeness`

### 5.2 硬规则

1. 每个 service 是独立可构建的库。
2. 业务逻辑只通过 `ports/` 接触外部世界。
3. service 的分布式语义由 `service-agent` 在 `model.yaml` 中声明。
4. service 不得直接依赖其他 `services/*`。
5. service 不得直接依赖 concrete adapters。
6. service 不得暴露框架专属类型到 domain。

### 5.3 两个固定 Reference Modules

| 模块 | 作用 |
|---|---|
| `counter-service` | 最小完整链路、CAS、event、projection 样例 |
| `tenant-service` | 多租户、多实体、workflow、补偿样例 |

这些模块不是临时 demo，而是长期 reference。

---

## 6. `servers/`：同步入口层

```text
servers/<name>/
├── Cargo.toml
├── openapi.yaml
├── src/
│   ├── handlers/
│   ├── middleware/
│   ├── routes/
│   ├── composition/
│   └── main.rs
└── README.md
```

### 必须

1. 只有一个 `main.rs`
2. 只做协议解析、鉴权、限流、租户注入、组合调用、响应组装
3. 所有 handler 都可追溯到 contracts
4. trace / auth / tenant context 统一经 middleware 注入

### 禁止

1. 直接操作数据库
2. 在 handler 写领域规则
3. 把长事务写在 HTTP 请求上下文里

---

## 7. `workers/`：异步执行与状态推进层

```text
workers/<name>/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── job_or_consumer/
│   ├── checkpoint/
│   ├── dedupe/
│   ├── retry/
│   └── metrics/
└── README.md
```

### 必须

1. 一类核心职责一个 worker
2. 输入、输出、幂等键、checkpoint、重试策略必须明确
3. 所有消息处理默认可重复执行
4. 所有对外副作用必须可去重
5. 支持 replay / resume

### Reference Workers

1. `outbox-relay`：至少一次投递、去重、checkpoint、retry
2. `projector`：projection、rebuild、replay、lag SLO

### 禁止

1. 混入展示逻辑
2. 复用 BFF handler 充当 worker
3. 把多个不相关职责堆进一个 worker

---

## 8. `packages/`：共享能力与基础设施抽象

```text
packages/
├── kernel/
├── platform/
├── contracts/
├── sdk/                       # generated
├── runtime/
│   ├── ports/
│   ├── execution/
│   ├── messaging/
│   ├── clock/
│   ├── dedupe/
│   └── checkpoint/
├── authn/
├── authz/
├── authz-traits/
├── authz-adapters/
├── data/
├── data-traits/
├── data-adapters/
├── cache/
├── cache-traits/
├── cache-adapters/
├── storage/
├── storage-traits/
├── storage-adapters/
├── observability/
├── security/
├── ui/
└── devx/
```

### 关键原则

1. 先有 trait / port，再有 adapter
2. vendor 依赖只出现在 `*-adapters/`
3. `packages/*` 不得依赖 `services/*`
4. runtime/* 负责分布式共性，不承载业务语义
5. `event-bus` 类能力应落在 `packages/messaging/`，而不是伪装成业务 service

---

## 9. `verification/`：验证层

```text
verification/
├── contract/
├── e2e/
├── topology/
├── resilience/
├── performance/
├── golden/
├── compatibility/
└── replay/
```

### 必须验证的能力

1. contract compatibility
2. topology 不改变业务边界
3. projection 可重建
4. workflow 可恢复
5. 重复消息不会产生重复副作用
6. stale read 行为符合声明
7. 双版本共存不破坏语义

### `golden/`

至少一套完整金样例，覆盖：

1. 一个 command
2. 一个 event
3. 一个 query
4. 一个 workflow
5. 一个 projection
6. 一个 replay
7. 一个 authz check
8. 一个 topology switch

---

## 10. `agent/`：最小编排层

`agent/` 只保留最小且高杠杆的 agent 协作真理源：

1. `codemap.yml`
2. `manifests/routing-rules.yml`
3. `manifests/gate-matrix.yml`
4. `README.md`

### 各自职责

1. `repo-layout.md` 负责让 agent 快速理解项目目标态、后端分布式边界、真理源分层与依赖方向。
2. `codemap.yml` 负责让 agent 不敢乱写：路径边界、写权限、依赖方向、禁止模式、required fields、分布式硬规则都必须机器可读。
3. `routing-rules.yml` 负责 touched paths 到 subagent 的路由与派发顺序。
4. `gate-matrix.yml` 负责 subagent 完成后必须执行的 scoped gates 与总验证。

### 关键要求

1. `agent/` 不再承载 prompts、templates、checklists、handoff 散文说明；这些内容如果不能被脚本或 CI 直接消费，不应继续膨胀上下文。
2. 关键约束必须机器可读，不允许只写散文说明。
3. 真正的生产级分布式语义应落在 `platform/schema/**`、`platform/model/**`、`services/*/model.yaml`、`packages/contracts/**`，而不是堆积在 agent 文档层。
4. `codemap.yml` 必须告诉 agent：何时声明 owner、何时走 workflow、何时拒绝 cross-service write，以及哪些 generated 目录禁止手改。

---

## 11. 依赖方向

```text
apps/*         -> packages/sdk, packages/ui, packages/authn
servers/*      -> services/*, packages/*
workers/*      -> services/*, packages/*
services/*     -> packages/kernel, packages/platform, packages/contracts, packages/runtime, packages/authn, packages/authz
packages/*     -> 低层可互依，但不得依赖 apps/servers/workers/services
platform/*     -> schema/generators/validators/catalog
infra/*        -> 不被运行时代码 import
ops/*          -> 不被运行时代码 import
verification/* -> 可依赖全仓库，仅用于验证
```

### 强制限制

1. `apps/*` 禁止依赖 `services/*`
2. `services/*` 禁止依赖其他 `services/*`
3. `services/*` 禁止依赖 `*-adapters/*`
4. `servers/*` / `workers/*` 允许聚合多个 `services/*`
5. generated 目录一律只读

---

## 12. 状态语义硬规则

### 12.1 唯一写主

任何核心实体必须有唯一 owner service。非 owner 禁止写主状态。

### 12.2 跨进程副作用必须幂等

任何可能重试、重投、重放的动作，都必须有 idempotency key。

### 12.3 长事务必须 workflow 化

任何跨 service 多步状态变更，不得写成 handler 链式 RPC，必须进入 workflow model。

### 12.4 Projection 必须可删可重建

如果删了不能重建，就不是 projection。

### 12.5 Query 必须声明一致性等级

不允许默认“刚写完就一定读到”。

### 12.6 Stateful 单元必须声明身份与存储

需要稳定身份、稳定存储、唯一持有语义的单元，必须在 deployable model 中显式声明。

### 12.7 双版本兼容是默认要求

API、event、projection、workflow 的变更，必须假设新旧版本会共存。

---

## 13. 仓库入口必须写明的 12 条规则

1. 先改 `platform/schema/*`，再改 `platform/model/*` 与 `services/*/model.yaml`
2. 先改 `packages/contracts/*`，再改 `servers/*` / `workers/*`
3. `services/*` 是能力边界，不是进程
4. `workers/*` 是一等公民，不是附属脚本
5. `services/<name>/model.yaml` 是 service-local distributed semantics 真理源
6. `platform/model/*` 只保留平台级元数据与全局规则
7. vendor 只能进入 `*-adapters/*`
8. generated 目录禁止手改
9. topology 只允许改变承载形态，不允许改变状态语义
10. 任何核心实体必须声明唯一 owner service
11. 任何跨进程副作用必须声明幂等 / 重试 / 去重 / 回放语义
12. 任何跨 service 长事务必须通过 workflow model 定义，不得用 handler 链式 RPC 代替

---

## 14. 开发准入标准

只有当以下问题都能被 model、template、validator、reference 共同回答时，才允许进入大规模业务开发：

1. 这个新实体归谁拥有？
2. 这个变更是 command、event、query 还是 workflow？
3. 这个副作用的幂等键是什么？
4. 这个 query 的一致性等级是什么？
5. 这个 worker 如何 checkpoint 与恢复？
6. 这个 projection 是否可删可重建？
7. 这个 deployable 是否需要 stable identity 与 persistent storage？
8. topology 切换后，状态语义是否不变？

若这些问题仍主要依赖人脑解释，而不是 harness 自动约束，则仓库尚未进入大规模业务开发阶段。
