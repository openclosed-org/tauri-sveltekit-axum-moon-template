# Repo Layout

> 目标：让仓库从单 VPS 到多服务/多节点演进时，**不需要重构业务骨架**。  
> 原则：**平台模型先于基础设施，契约先于实现，服务是库不是进程，worker 是一等公民。**

---

## 0.5 当前成熟度说明（事实基线）

> 本文件描述的是目标态目录结构与规则。
> 本文件不描述当前实现的成熟度水平（存在明显偏差）。
>
> - 后端：L2（第二档）——结构真相源与局部门禁已落地，但尚未形成第三档闭环。
> - 前端：L1（第一档）——历史遗留结构明显，规则真相源未落地，门禁缺失。
> - 集成：未启动（仅有规划）。
>
> 完整档位定义见 `docs/architecture/maturity-levels.md`。

---

## 1. 核心设计原则

### 1.1 平台模型优先
- `platform/model/*` 是平台真理源。
- 服务、部署单元、资源、工作流、策略、拓扑、环境，必须先在模型层声明。
- `infra/*`、`docs/generated/*`、`packages/sdk/*`、`platform/catalog/*` 都应由模型生成或校验。

### 1.2 契约优先于实现
- `packages/contracts/*` 是协议真理源。
- HTTP / Event / RPC / DTO / ErrorCode 的变化，先改契约，再改 server / worker / app 实现。
- 不允许 server handler 先私自扩展协议，再补契约。

### 1.3 Services 是库，不是进程
- `services/*` 表示业务能力边界，不表示 HTTP 服务、消息消费者、CLI 进程。
- `servers/*` 是同步请求入口。
- `workers/*` 是异步执行单元。
- 同一个 `services/<name>` 可以同时被多个 `servers/*` 和 `workers/*` 复用。

### 1.4 Runtime Ports 高于中间件
- 业务代码只能依赖 `packages/runtime/ports/*` 等平台抽象。
- 具体中间件必须放在 `packages/*/adapters/*`。
- 不允许业务代码直接 import NATS / Dapr / Redis / Turso / OpenFGA 等具体 SDK。

### 1.5 拓扑切换不靠重构
- 单 VPS、单节点 k3s、拆边缘层、拆 worker、全微服务，都只允许通过 `platform/model/topologies/*.yaml` 切换。
- 禁止因部署方式变化而改变业务模块边界。

### 1.6 生成物禁止手改
- `packages/sdk/*`
- `infra/kubernetes/rendered/*`
- `docs/generated/*`
- `platform/catalog/*`

以上目录必须可删可再生，禁止手工长期维护。

---

## 2. 目录总览

```text
boilerplate/
├── 根配置层
├── agent/                  # Agent 约束与模板
├── platform/               # 平台模型层（真理源）
├── apps/                   # 前端与客户端壳层
├── servers/                # 同步请求入口层
├── services/               # 纯业务能力层
├── workers/                # 异步执行层
├── packages/               # 共享抽象 / 适配器 / SDK / 工具
├── infra/                  # 基础设施声明与交付层
├── ops/                    # 运维执行层
├── verification/           # 跨模块 / 跨拓扑验证层
├── docs/                   # ADR / 架构 / 运维文档
├── fixtures/               # 测试数据与种子
└── tools/                  # 本地辅助工具
```

---

## 3. 顶级目录说明与硬规则

> 注：本文档描述的是目标态目录结构。部分子目录当前为空（含 `.gitkeep` 占位符），待后续增量开发时填充。Agent 不得仅凭文档推断"文件应该存在"，应以实际代码为准。

## 3.1 根配置层

### 职责
集中管理工具链、工作区、质量门禁、任务编排、CI 与发布入口。

### 必须
- 所有开发与运维命令统一从 `justfile` 暴露。
- 所有工具版本由 `.mise.toml`、`rust-toolchain.toml` 锁定。
- 所有工作区成员在根 `Cargo.toml` 与根 `package.json` 的 `workspaces` 字段可追踪。
- CI 通过根命令调用，不直接拼接深层脚本。

### 禁止
- 根目录散落临时脚本。
- 子目录私自维护第二套工具版本配置。
- 绕过 `just` 直接执行不可追踪命令。

### 验证
- `just --list` 必须列出完整命令。
- `mise doctor` 必须通过。
- `cargo metadata` 与 monorepo 图一致。

---

## 3.2 `agent/`

### 职责
定义 Agent 的修改边界、依赖规则、禁止模式、生成模板、操作清单。

### 必须
- 新增模块前先更新 `codemap.yml` 或使用模板。
- 危险操作必须有 checklist。
- 依赖白名单、黑名单、禁止模式必须机器可读。

### 禁止
- 存放业务代码。
- 把关键架构约束只留在对话，不落地到文件。
- 在模板中混入真实业务逻辑。

### 验证
- 新 Agent 只读 `agent/` 与 `docs/`，也能按规则新增模块。
- PR 中的越层依赖能被规则拒绝。

---

## 3.3 `platform/`

### 职责
描述平台，不实现业务。  
回答这些问题：
- 系统有哪些逻辑服务
- 哪些是可运行部署单元
- 它们依赖哪些外部资源
- 使用哪些超时 / 重试 / 幂等 / 观测 / 授权策略
- 在什么拓扑和环境下运行

### 结构
- `schema/`：平台模型 Schema
- `model/services/`：逻辑服务定义
- `model/deployables/`：可部署单元定义
- `model/resources/`：外部资源定义
- `model/workflows/`：业务工作流定义
- `model/policies/`：平台策略定义
- `model/topologies/`：部署拓扑定义
- `model/environments/`：环境变量与环境差异抽象
- `generators/`：生成 compose / kustomize / flux / docs / sdk / graph
- `validators/`：模型、依赖、契约、拓扑、安全、观测校验
- `catalog/`：可审查的生成结果

### 必须
- 所有模型文件可被 schema 校验。
- `deployables/` 与 `services/` 分离，不能混淆。
- `topologies/` 只能描述部署组合，不写业务语义。
- `catalog/` 必须是生成结果，不手写。

### 禁止
- 放业务实现代码。
- 手写具体云厂商 SDK 调用。
- 先修改 `infra/rendered` 再反推 model。

### 验证
- 删除生成目录后可重建。
- 任一 deployable 的资源需求、暴露方式、依赖资源都可从模型中回答。

---

## 3.4 `apps/`

### 职责
承载前端与客户端壳层，只消费 SDK、UI、认证与同步协调能力。

### 必须
- 所有 API 调用通过 `packages/sdk/*`。
- 端特定能力封装在各自 `lib/`。
- 同步逻辑收口到 `lib/sync/`，不得散在页面。

### 禁止
- 直接 import `services/*`。
- 手写与后端重复的 DTO。
- 在页面层写复杂业务规则。

### 验证
- 更新契约后，只更新 SDK 即可通过类型检查。
- 应用层与 server 实现解耦。

---

## 3.5 `servers/`

### 职责
同步请求入口层，负责协议适配、聚合、鉴权、限流、租户注入、视图组装。

### 必须
- 每个 server 只有一个 `main.rs` 入口。
- 每个对外 HTTP server 提供 `openapi.yaml`。
- handler 只负责解析请求、调用 application API、组织响应。
- 统一经过 middleware 注入 trace / tenant / auth context。

### 禁止
- 写领域规则。
- 直接操作数据库。
- 执行长轮询、批处理、后台驻留任务。

### 验证
- `cargo build -p <server>` 独立通过。
- contract tests 能覆盖 openapi 与 handler 一致性。

---

## 3.6 `services/`

### 职责
存放领域模型、用例逻辑、业务策略、领域事件与外部依赖抽象。

### 标准结构
每个 service 必须至少包含：
- `src/domain/`
- `src/application/`
- `src/policies/`
- `src/ports/`
- `src/events/`
- `src/contracts/`
- `src/lib.rs`
- `tests/`
- `migrations/`
- `Cargo.toml`

### 必须
- 业务逻辑只通过 `ports/` 接触外部世界。
- 可通过 in-memory / mock ports 做纯业务测试。
- 每个 service 独立可构建、可测试。

### 禁止
- 直接依赖其他 `services/*`。
- 直接依赖具体适配器。
- 暴露框架专属类型到 domain 层。

### 验证
- `cargo test -p <service>` 不需要启动 HTTP server。
- domain 测试不依赖 DB / 网络。

---

## 3.7 `workers/`

### 职责
承载异步执行单元：索引器、投影器、投递器、调度器、同步协调器、工作流执行器。

### 必须
- 一个 worker 只承载一个核心职责。
- 明确输入、输出、幂等键、重试策略、checkpoint 策略。
- 与消息系统交互必须走 runtime / messaging 抽象。

### 禁止
- 复用 BFF handler 充当 worker。
- 把多个不相关职责塞进一个 worker。
- 在 worker 中写端展示逻辑。

### 验证
- 重启 worker 后能从 checkpoint 恢复。
- 重复消息不会产生重复副作用。

---

## 3.8 `packages/`

### 职责
提供共享能力、底层抽象、具体适配器、SDK、开发工具。

### 关键分层
- `kernel/`：最底层通用类型
- `platform/`：平台元信息、配置、健康检查
- `contracts/`：协议真理源
- `sdk/`：生成 SDK
- `runtime/`：分布式系统标准库抽象
- `authn/`：认证与会话
- `authz/`：授权抽象与 OpenFGA 适配
- `data/`：数据库与 outbox/inbox 支撑
- `messaging/`：事件包络与 NATS 约定
- `cache/`：缓存抽象与适配器
- `storage/`：对象存储抽象与适配器
- `observability/`：日志/指标/trace
- `security/`：加密、签名、脱敏
- `web3/`：协议适配层
- `wasm/`：插件执行面
- `ui/`：共享 UI
- `devx/`：测试与开发工具

### 必须
- 先有 ports，再有 adapters。
- vendor 依赖只出现在 adapters。
- 共享包不能包含业务域规则。

### 禁止
- 手改 `sdk/` 生成结果。
- 高层包反向依赖业务层。
- 用 `common`、`utils` 做垃圾桶目录。

### 验证
- 底层包可独立构建。
- 替换 adapter 后业务签名不变。

---

## 3.9 `infra/`

### 职责
管理本地环境、集群基座、GitOps、安全、镜像构建。

### 结构
- `local/`：compose 与本地 seed
- `kubernetes/bootstrap/`：集群初始化
- `kubernetes/base/`：基础命名空间、RBAC、安全、网络策略
- `kubernetes/addons/`：Cilium、Gateway API、NATS、OpenObserve、ZITADEL、OpenFGA 等
- `kubernetes/rendered/`：模型生成的工作负载
- `gitops/flux/`：生产交付面
- `security/`：sops、SBOM、签名、供应链
- `images/`：镜像构建模板

### 必须
- `rendered/` 只读。
- 生产环境交付通过 Flux。
- secrets 统一经 `sops` 管理。

### 禁止
- 在 `infra/` 放业务代码。
- 手工长期维护 app workload manifests。
- 以 shell 脚本取代 GitOps。

### 验证
- bootstrap cluster -> render manifests -> flux reconcile 即可部署。
- 明文 secrets 不得进入仓库。

---

## 3.10 `ops/`

### 职责
承载运维执行逻辑：迁移、压测、故障演练、备份恢复、巡检。

### 必须
- 所有操作通过 `just` 触发。
- 关键动作有 runbook。
- 压测、故障演练、恢复演练分目录管理。

### 禁止
- 被业务运行时代码 import。
- 存留无文档的临时脚本。

### 验证
- 每份 runbook 都能映射到具体命令。
- staging 环境可演练备份恢复。

---

## 3.11 `verification/`

### 职责
提供跨模块、跨拓扑、跨生成层验证。

### 必须
- 包含 e2e、contract、topology、resilience、performance、golden。
- `counter`、多租户、`settings` 必须在这里有完整链路。

### 禁止
- 放生产代码。
- 把单模块单测混入这里。

### 验证
- 切换不同 topology 后，同一批 E2E 仍能运行。
- 契约兼容检查能阻止破坏性修改。

---

## 3.12 `docs/`

### 职责
沉淀 ADR、架构设计、平台模型说明、运维手册、生成资产。

### 必须
- 重大变更对应 ADR。
- `docs/generated/` 必须由命令生成。
- 新成员只读 docs 就能理解系统骨架。

### 禁止
- 架构知识只留在对话记录。
- 文档长期与现状背离。

### 验证
- 任一顶级目录都应有对应说明。
- 架构类 PR 必须更新 docs 或 ADR。

---

## 3.13 `fixtures/`

### 职责
集中存放测试数据、演示数据、同步场景、授权关系 tuples。

### 必须
- 全部可版本控制。
- 演示数据与测试断言一致。
- 多租户、settings、counter、authz 都要有独立 fixture。

### 禁止
- 混入生产真实数据。
- 在测试中长期硬编码大型对象替代 fixtures。

### 验证
- 空库可通过 seed 恢复完整 demo 场景。
- authz tuples 可独立加载重放。

---

## 3.14 `tools/`

### 职责
承载本地协议测试、代码生成辅助、诊断与负载工具。

### 必须
- 每个工具有用途说明。
- 所有入口经 `just` 暴露。
- 工具不进入生产运行时依赖。

### 禁止
- 在 tools 中放正式业务二进制。
- 用 tools 替代 platform generator 或 infra 声明。

### 验证
- 删除 `tools/` 不影响生产构建。

---

## 4. 依赖方向规则

```text
apps/*        -> packages/sdk, packages/ui, packages/authn
servers/*     -> services/*, packages/*
workers/*     -> services/*, packages/*
services/*    -> packages/kernel, packages/platform, packages/contracts, packages/runtime(ports), packages/authn, packages/authz
packages/*    -> 低层可互依，但不得依赖 servers/apps/workers/services
platform/*    -> 只依赖 schema / generators / validators / catalog
infra/*       -> 不被运行时代码 import
ops/*         -> 不被运行时代码 import
verification/*-> 可依赖全仓库，仅用于验证
```

### 强制限制
- `apps/*` 禁止依赖 `services/*`
- `services/*` 禁止依赖其他 `services/*`
- `services/*` 禁止依赖 `packages/**/adapters/*`
- `servers/*` / `workers/*` 允许聚合多个 `services/*`
- `packages/sdk/*`、`infra/kubernetes/rendered/*`、`docs/generated/*` 为只读生成目录

---

## 5. 命名规则

### 逻辑服务命名
- 使用业务域名：`user`、`tenant`、`settings`、`admin`
- 禁止：`common`、`misc`、`helpers`、`utils` 作为顶级业务模块名

### 部署单元命名
- `*-bff`：聚合同步入口
- `*-gateway`：边缘/入口层
- `*-worker`：异步执行单元
- `*-relay`：投递桥接
- `*-projector`：读模型投影
- `*-reconciler`：同步/修复/补偿
- `*-runner`：任务或工作流执行器

### 适配器命名
- 先能力、后 vendor：`cache/adapters/dragonfly`
- 禁止以 vendor 名直接成为顶级共享模块

---

## 6. 新增模块的最小模板要求

## 6.1 新增一个 Service

```text
services/<name>/
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

## 6.2 新增一个 Server

```text
servers/<name>/
├── Cargo.toml
├── openapi.yaml
├── src/
│   ├── handlers/
│   ├── middleware/
│   ├── routes/
│   └── main.rs
└── README.md
```

## 6.3 新增一个 Worker

```text
workers/<name>/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── jobs_or_consumers/
│   └── checkpoint_or_dedupe/
└── README.md
```

## 6.4 新增一个 Resource Model

```text
platform/model/resources/<name>.yaml
```

## 6.5 新增一个 Topology

```text
platform/model/topologies/<name>.yaml
verification/topology/<name>/
docs/operations/<name>.md
```

---

## 7. 推荐统一命令

```text
just doctor
just lint
just test
just test-e2e
just gen-contracts
just gen-sdk
just gen-platform
just render-local
just render-k8s env=staging
just validate-platform
just validate-deps
just validate-contracts
just validate-topology
just verify-generated
just verify-single-vps
just verify-k3s
just perf-smoke
```

---

## 8. 仓库入口必须写明的七条规则

1. 先改 `platform/model/*`，再改 `infra/*`
2. 先改 `packages/contracts/*`，再改 `servers/*`
3. `services/*` 是库，不是进程
4. `workers/*` 是一等公民，不是附属脚本
5. vendor 只能进 `adapters/*`
6. generated 目录禁止手改
7. 拓扑变化通过 topology model 完成，不允许靠重构业务来实现

---

## 9. Gate 演进协议

### 9.1 统一入口
- 本地、pre-push、CI、release 门禁统一从 `just gate-local`、`just gate-prepush`、`just gate-ci`、`just gate-release` 进入。
- `lefthook.yml` 只调用 `just gate-*`，不得在 hook 中散落独立检查命令。
- 人类与 agent 都不得绕过 `just` 直接拼接深层质量命令作为正式门禁入口。

### 9.2 软硬分层
- Phase 0 默认采用 warn-only：`gate-local` 与 `gate-prepush` 允许失败告警但不阻断提交/推送。
- 当某类规则稳定、噪声可控后，再将对应 gate 或单项检查切换为 strict。
- `gate-ci` 与 `gate-release` 可以先于本地 gate 进入 strict，但必须保持规则来源可追踪。

### 9.3 规则同步
- 结构事实变化时，必须同步更新 `agent/codemap.yml` 或本文档。
- 规则进入 gate 后，必须对应真实的检查命令或验证脚本，不能只写文档不落地。
- gate 语义发生变化时，必须同步更新迭代计划中的进度记录与接力交接单。
