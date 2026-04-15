# Counter-Service Gap Fix Plan

> 状态：Draft / Supersedes previous ad-hoc gap checklist
>
> 目标：把 `counter-service` 这条后端 reference chain 修成一个真正可复制、可验证、可被 agent 直接按规范填空开发的企业级分布式 starter。
>
> 本文档不再接受“先用 `.env` 过渡一下”的设计。后端配置与密钥治理以 `Kustomize + SOPS + age + Flux` 为唯一目标态与默认开发路径。

---

## 1. Why This Plan Was Rewritten

上一版 gap 修复计划默认接受一段过渡期：

1. 先修代码和文档不一致
2. `.env` 暂时继续存在
3. 后续再把 secrets / GitOps / cluster path 对齐到生产级

这不符合当前项目定位。

当前项目不是普通业务仓库，而是：

1. Agent-first
2. Distributed-first
3. Reference-module-driven
4. Production-grade starter

因此，`counter-service` 不能只是“业务逻辑先跑起来”。它必须作为：

1. 新 service 的 copy target
2. 新 worker 的行为模板
3. 新 server 的协议适配模板
4. 新环境与部署生成的验证样例
5. Agent 默认假设的架构真相源

这意味着要求已经提高，而且必须重写计划。

结论：

1. 是的，要求提高了
2. 是的，计划必须重新设计
3. 新计划必须把“配置/密钥/部署/运行时路径”也纳入 reference chain 的一部分

---

## 2. Hard Requirements

以下要求为本计划硬约束，不接受“先凑合”的替代方案。

### 2.1 Counter-Service 是后端参考起点

后端开发统一以 `services/counter-service/` 为最小完整链路起点。

它必须同时覆盖：

1. service-local semantics
2. contracts-first
3. CAS versioning
4. idempotency
5. outbox
6. relay
7. event publication
8. projection/replay friendliness
9. BFF integration
10. production-style deploy/config path

### 2.2 名称统一为 `counter-service`

仓库内凡是表达 owner service、service metadata、deployable 依赖、文档引用时，统一使用 `counter-service`。

不再混用：

1. `counter`
2. `counter-service`

`counter` 只允许在实体名、事件语义、聚合名中使用，例如：

1. entity: `counter`
2. event: `counter.changed`
3. aggregate root: `counter`

### 2.3 后端不以 `.env` 为默认运行入口

后端的配置与密钥真理源必须是：

1. `Kustomize` 组织环境差异
2. `SOPS` 加密配置与密钥
3. `age` 作为密钥机制
4. `Flux` 负责 GitOps 解密与部署

`.env` 不得再作为后端 reference path 的第一入口。

### 2.4 Agent 必须感知到“这是生产级分布式系统”

后续 agent 进入仓库后，不应再做以下推断：

1. “本地先手写 `.env` 吧”
2. “worker 先用 in-memory stub 吧”
3. “OpenAPI 先空着吧”
4. “幂等逻辑先声明不实现吧”
5. “CI 以后再补齐吧”

它应当默认看到的是：

1. 配置路径是统一的
2. 命名是统一的
3. contracts 和实现是一致的
4. worker 是真实连线的
5. gates 是可执行且必跑的

---

## 3. Target End State

本计划完成后，`counter-service` 参考链路必须达到以下状态。

### 3.1 语义层

1. `services/counter-service/model.yaml` 成为 counter 链路细粒度语义真理源
2. `platform/model/services/counter-service.yaml` 成为平台级元数据真理源
3. `platform/model/state/ownership-map.yaml` 与 `counter-service` 命名和 owner 对齐
4. `contracts/events`、`contracts/api`、service 代码、BFF handler、worker 消费逻辑全部与模型一致

### 3.2 运行时层

1. `web-bff` 通过统一 config contract 读取配置
2. `outbox-relay-worker` 不再使用 `MemoryOutboxReader`
3. 本地 dev / staging / prod 采用同一条 config-secrets path，只是 overlay 不同
4. 本地也可以通过 cluster path 跑起来，而不是单独依赖 `.env`

### 3.3 运维层

1. Flux Kustomization 对应用和 secrets 进行统一管理
2. SOPS-encrypted secrets 可直接用于 cluster reconcile
3. 本地开发通过与生产同构的 kustomize overlay 运行
4. CI / gate 会验证模型、边界、contracts、worker 连接和 OpenAPI 对齐

### 3.4 文档层

1. docs 明确写清 reference chain 的唯一运行/配置方式
2. 不再存在“代码说 APP_，文档说 WEB_BFF_，部署写别的 key”这种分裂
3. agent 能在 docs 和 model 中直接找到正确答案

---

## 4. Design Decision: Backend Config And Secrets

这是本次计划最重要的架构冻结项。

### 4.1 Final Decision

后端配置与密钥治理统一为：

1. `infra/k3s/overlays/*` 负责环境差异
2. `infra/gitops/flux/apps/*.yaml` 负责 reconcile 和 SOPS 解密
3. `infra/security/sops/` 负责加密模板与 rules
4. `age` key 作为 Flux 解密密钥
5. backend binaries 只消费标准环境变量，不感知 `.env`

### 4.2 Local Development Rule

本地后端开发也必须沿同一条路径：

1. 使用本地 K3s/K3d overlay 跑服务
2. 使用 SOPS 管理 dev secrets
3. 通过 Kustomize/Flux 或等价的 cluster-apply path 注入配置

允许存在的“非 cluster 快速调试路径”只能是：

1. `sops exec-env ... cargo run -p <package>`

它是 cluster path 的派生辅助命令，不是新的配置真理源，也不写入 `.env` 文件。

### 4.3 No More Backend `.env` Contract

以下文件不再作为后端参考入口：

1. 根目录 `.env`
2. 根目录 `.env.example`

如果需要保留它们，也只能用于：

1. 非后端组件
2. 明确标记为 legacy / transitional

不得让 server-agent / worker-agent / service-agent 再把它们视为默认路径。

### 4.4 Why This Matches The Existing Stack

仓库里已经存在以下事实：

1. `infra/gitops/flux/apps/web.yaml` 已配置 `decryption.provider: sops`
2. `infra/security/sops/secrets.template.yaml` 已存在 SOPS 模板
3. `docs/architecture/deployment/01-deployment.md` 已把 K3s 的 secrets 路径定义为 `SOPS + Flux + age`
4. `platform/model/topologies/k3s-staging.yaml` 已明确 staging 使用 `Flux`

问题不是要不要引入这套，而是要把这套从“半存在状态”收敛成后端唯一标准路径。

---

## 5. Gap Summary By Domain

### 5.1 Platform / Model Gaps

1. `counter` / `counter-service` 命名混用
2. platform model 与 service model 没形成单一命名约定
3. topology / resource 文档与实际路径仍混有旧路径和旧包名

### 5.2 Contracts Gaps

1. `CounterChanged` 已扩展，但 typegen / drift 没形成闭环
2. BFF 没实际使用 `CounterResponse` / `ErrorResponse`
3. 静态 OpenAPI 文件与运行时 utoipa 文档分裂

### 5.3 Service Gaps

1. 领域层存在重复 `Counter` 结构
2. `CounterId` 与对外 DTO 的 id 类型未统一收口
3. `CounterDomainError` 缺少 CAS conflict 语义
4. `events/` 目录仍是占位文档，不是实际参考定义
5. 应用层幂等逻辑是空实现
6. migration 常量过时
7. SQL 定义重复维护

### 5.4 Server / BFF Gaps

1. BFF handler 未使用 contracts DTO
2. cache 命中路径顺序错误
3. config key 与文档、部署 key 不统一
4. remote/embedded backend 分支重复
5. OpenAPI 静态文件是空壳

### 5.5 Worker Gaps

1. `outbox-relay-worker` main 仍使用 `MemoryOutboxReader`
2. `mark_published` 未进入主流程
3. worker 缺少真实 config contract
4. checkpoint / retry / dedupe 的生产级路径未闭合
5. indexer/projector 仍有 placeholder 行为

### 5.6 Infra / Delivery Gaps

1. Flux + SOPS 已有骨架，但未成为统一开发入口
2. k3s overlays 仍有旧资源名、旧 deployment 名
3. deploy manifests 与实际 config key 不对齐
4. secrets template 仍是 generic app-secret，而非按 deployable/service 拆分

### 5.7 CI / Gates / Docs Gaps

1. workflows 引用旧路径和旧包名
2. gate-matrix 要求的验证未全跑
3. docs 没把“无 `.env`、GitOps first”写成开发铁律

---

## 6. Execution Principles

本计划按以下原则执行：

1. 先冻结运行与配置模型，再修实现
2. 先统一命名，再修 contracts / code
3. 先让 chain 真正跑通，再补扩展模块
4. 一处语义只维护一份真理源
5. 所有 reference path 都要可验证，不接受“文档说一套，代码跑另一套”

---

## 7. Workstreams

本计划拆为 7 个 workstream，按依赖顺序推进。

---

## WS-0: Freeze The Golden Operating Model

### Goal

把 counter chain 的“唯一正确运行方式”冻结下来，消除 `.env` 与多套配置入口的歧义。

### Tasks

1. 明确后端统一运行模型：`Kustomize + SOPS + age + Flux`
2. 明确 local/staging/prod 只允许 overlay 不同，不允许配置模型不同
3. 在 docs、platform model、infra comments 中统一写清：backend binaries 消费 env vars，但 env vars 的生成/注入来自 SOPS/Kustomize/Flux，而不是 `.env`
4. 明确 quick inner loop 的唯一允许辅助命令是 `sops exec-env` 风格，不落地 `.env`

### Deliverables

1. 统一的 backend config/secrets policy 文档
2. 更新后的 docs/architecture 与 docs/operations 约定
3. 后续 workstream 的命名与路径基线

### Acceptance Criteria

1. 仓库中没有任何后端文档继续把 `.env` 描述成默认入口
2. 任何 agent 只看 docs 和 infra 结构也能得出正确结论：SOPS/Flux 是标准路径

---

## WS-1: Secrets And Config Control Plane

### Goal

让 secrets/config 治理从“骨架存在”升级为“现在就能跑”的统一控制面。

### Tasks

1. 重构 `infra/security/sops/`，从单个 `app-secrets` 模板改为按 deployable/service 拆分
2. 引入 `.sops.yaml` 作为统一 SOPS rule 文件
3. 约定并落地 age key 的生成、存储、Flux 引用方式
4. 为 `web-bff`、`outbox-relay-worker`、后续 worker 建立独立 secrets 清单
5. 为每个 deployable 建立 config map / secret contract，区分：
   1. public config
   2. sensitive config
   3. runtime override
6. 为 local dev 增加同构 overlay，使本地 cluster 直接 consume 同一套 encrypted secrets
7. 为非 cluster cargo run 提供 `sops exec-env` 脚本，不产生 `.env`

### Deliverables

1. `.sops.yaml`
2. `infra/security/sops/age/` 或等效 key 规范文档
3. `infra/security/sops/<env>/<deployable>-secret.enc.yaml`
4. `just` / `scripts/` 命令：
   1. 生成 age key
   2. 编辑 secrets
   3. 本地 reconcile
   4. `sops exec-env` 启动单个二进制

### Acceptance Criteria

1. `web-bff` 可以不依赖 `.env` 启动
2. `outbox-relay-worker` 可以不依赖 `.env` 启动
3. staging/prod 的 Flux Kustomization 能引用同一套 SOPS/age 结构
4. docs 明确写清本地和集群使用同构 secrets path

---

## WS-2: Naming, Model, And Ownership Normalization

### Goal

彻底消除 `counter` / `counter-service` 的命名漂移，建立可被 validator 与 agent 直接消费的统一语义。

### Tasks

1. 将 `platform/model/services/counter.yaml` 重命名为 `counter-service.yaml`
2. 将其中 `name` 统一改为 `counter-service`
3. 将 `platform/model/state/ownership-map.yaml` 中 `owner_service` 改为 `counter-service`
4. 更新 `agent/codemap.yml` reference module 名称为 `counter-service`
5. 更新 `platform/model/resources/*.yaml` 中 `required_by` 的 service naming
6. 更新所有 docs 中的 reference service naming

### Deliverables

1. 单一命名规范：service 永远叫 `counter-service`
2. entity 永远叫 `counter`

### Acceptance Criteria

1. 所有 platform/service/docs/infra 引用都不再混用 `counter` 和 `counter-service`
2. `validate-state` / `validate-contracts` / grep 都不再出现冲突命名

---

## WS-3: Contracts And Service Semantic Convergence

### Goal

让 counter 语义从 model 到 contracts 再到 service 代码形成单一闭环。

### Tasks

1. 统一 `Counter` 结构，不再同时维护 domain Counter 和重复 DTO Counter
2. 保留 `CounterId` 作为领域内部强类型，同时提供清晰的边界映射，不再让外部 contracts 自己再定义一份平行结构
3. 给 `CounterDomainError` 与对外 `CounterError` 增加 CAS conflict 语义
4. 修复 `src/events/README.md` 和 `src/events/mod.rs`，让它们反映真实事件 `CounterChanged`
5. 确认 `contracts_events::CounterChanged` 与 `model.yaml` 的 event fields 一致
6. 运行并纳入 typegen / contract drift check

### Deliverables

1. 单一 Counter 语义模型
2. 明确的 domain-to-contract mapping
3. 可由 agent 直接照抄的 error taxonomy

### Acceptance Criteria

1. 不再存在重复 `Counter` 数据结构定义
2. `CounterChanged` 在 model / contracts / worker / docs 中完全一致
3. contract typegen 和 drift check 全绿

---

## WS-4: Counter-Service Runtime Integrity

### Goal

把 `counter-service` 从“语义声明大于实现”修成“语义与实现完全对齐”的 reference service。

### Tasks

1. 在 `CounterRepository` trait 中正式建模 idempotency read/write
2. 在 application service 中实现真正的幂等查询与结果缓存
3. 在 `TenantScopedCounterService` 中也走完整幂等链路，不允许只在 `RepositoryBackedCounterService` 有逻辑
4. 删除过时的 `COUNTER_MIGRATION` 常量
5. CAS mismatch 必须显式返回 conflict，而不是 silent stale read
6. 统一 SQL 真理源，避免同一套 schema 在常量、方法、`.sql` 文件中三处漂移
7. 为 reset/decrement/increment 的 CAS 与 idempotency 写完整 unit / integration tests

### SQL Source Of Truth Rule

本仓库在 counter-service 上采用：

1. SQL schema 只维护一份
2. 推荐保留 `services/counter-service/migrations/001_create_counter.sql` 作为 schema 真理源
3. `LibSqlCounterRepository::migrate()` 读取该文件执行，而不是重复硬编码 SQL 字符串

理由：

1. schema 可读性高
2. drift 易发现
3. 对 agent 更友好
4. 便于未来迁移生成/validator 扫描

### Deliverables

1. 真正工作的 idempotency path
2. 真正工作的 CAS conflict path
3. 单一 SQL 真理源
4. 更完整的 service tests

### Acceptance Criteria

1. 提供相同 idempotency key 时不会重复副作用
2. 并发冲突会返回 conflict，不会 silent success
3. schema 只有一份真理源
4. tests 覆盖 increment/decrement/reset/get/outbox/idempotency/CAS conflict

---

## WS-5: BFF As A Real Protocol Adapter

### Goal

把 `web-bff` 修成真正的 server reference，而不是“能调 service 的 handler 集合”。

### Tasks

1. handler 全部使用 `contracts_api::CounterResponse` / `ErrorResponse`
2. cache 读取顺序改为 cache-first
3. remote/embedded backend 分支抽象收口，避免 handler 重复实例化 service
4. config key 全面对齐 secrets/config control plane
5. 删除或生成静态 `openapi.yaml`，不再保留空壳文件
6. 明确 `/scalar` 是运行时 OpenAPI 真理展示，不允许静态文件长期漂移
7. 将 tenant middleware、config 注入、request path 与 docs 对齐

### Deliverables

1. 更干净的 BFF composition root
2. DTO-first handler path
3. 可验证的 OpenAPI 对齐

### Acceptance Criteria

1. handler 不再内联手搓 JSON DTO
2. BFF 可以通过统一 secrets/config path 启动
3. `/scalar` 与 handler annotation 对齐
4. 不存在空壳 OpenAPI 文件误导 agent

---

## WS-6: Outbox / Relay / Event Flow Closure

### Goal

把 worker 链路从 stub 拉通为真正的生产级事件流样例。

### Tasks

1. `outbox-relay-worker` main 改为真实使用 `LibSqlOutboxReader`
2. 为 relay 建立正式 config contract：database, nats, batch size, retry, checkpoint
3. 将 `mark_published()` 接入 publish success path
4. 将 checkpoint 变成持久状态，而不是纯内存
5. 将 dedupe/retry/checkpoint 与 worker-agent 的 strategy requirements 对齐
6. 检查 indexer / projector 对 `CounterChanged` 的消费是否仍存在 placeholder 逻辑，并修为真实 schema-compatible 行为

### Deliverables

1. 真正工作的 outbox relay
2. relay 生产级 config contract
3. worker 级 resilience 语义闭环

### Acceptance Criteria

1. outbox 写入后 relay 能真实读出并发布
2. 发布成功后会标记 published
3. worker 重启后 checkpoint 行为符合预期
4. `validate-resilience` 与 worker tests 能验证核心路径

---

## WS-7: Delivery, Gates, And Documentation Closure

### Goal

把所有 reference path 收尾到“文档、门禁、CI、运行时”四位一体一致。

### Tasks

1. 修正 `.github/workflows/*.yml` 中旧路径和旧包名
2. 将 `packages/messaging/**`、`packages/data/**`、`packages/data-traits/**`、`packages/data-adapters/**` 纳入 CI path filters
3. 跑通并接入：
   1. `validate-state`
   2. `validate-contracts`
   3. `validate-imports`
   4. `boundary-check`
   5. workspace cargo gates
4. 更新 docs，使后续 agent 能直接理解：
   1. counter-service 是 reference origin
   2. secrets 走 SOPS/Flux/age
   3. backend 不走 `.env`
   4. local dev 与 cluster path 同构

### Deliverables

1. CI/Gates 与仓库现状对齐
2. 文档与实现不再分裂
3. 可供 agent 直接消费的 reference chain narrative

### Acceptance Criteria

1. 所有 gates 可执行且结果可信
2. 新 agent 不需要凭经验猜 secrets/config path
3. docs 与代码的 grep 不再互相冲突

---

## 8. Recommended Execution Order

按依赖顺序执行：

1. WS-0 Freeze operating model
2. WS-1 Secrets and config control plane
3. WS-2 Naming and model normalization
4. WS-3 Contracts and service semantic convergence
5. WS-4 Counter-service runtime integrity
6. WS-5 BFF alignment
7. WS-6 Worker closure
8. WS-7 Gates and docs closure

其中：

1. WS-1 是后续所有运行修复的前置条件
2. WS-2 是模型与验证的前置条件
3. WS-3/WS-4 可以局部并行，但要先统一 naming
4. WS-5/WS-6 依赖 WS-1 的 config contract 和 WS-3/WS-4 的 schema 真相

---

## 9. Definition Of Done

当且仅当满足以下条件，counter-service gap 修复才算完成。

### 9.1 Architecture DoD

1. 后端 reference path 不再依赖 `.env`
2. local/staging/prod 使用统一的 secrets/config 模型
3. Flux/SOPS/age 路径真实可运行

### 9.2 Chain DoD

1. `counter-service` 命名一致
2. contracts / service / BFF / worker / docs 一致
3. idempotency / CAS / outbox / relay 都是实装，不是声明占位

### 9.3 Verification DoD

1. `cargo check --workspace`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo fmt --all -- --check`
4. `cargo test --workspace`
5. `bun run scripts/validate-state.ts --mode strict`
6. `bun run scripts/validate-contracts.ts`
7. `bun run scripts/validate-imports.ts`
8. `bun run scripts/boundary-check.ts`
9. BFF `/scalar` smoke test
10. real relay flow smoke test

### 9.4 Agent DoD

1. 一个新 agent 进入仓库后，不会再优先找 `.env`
2. 一个新 agent 看 `counter-service` 后，能自然按生产级分布式模式继续扩展新 service / new worker
3. 一个新 agent 不需要在 secrets/config path 上做取舍题

---

## 10. What This Plan Explicitly Rejects

以下做法被本计划明确拒绝：

1. 继续让根目录 `.env` 充当 backend reference path
2. 继续让 worker main 使用 in-memory reader 充当“已接线”
3. 继续保留空壳 OpenAPI 文件
4. 继续让幂等逻辑只存在 `model.yaml` 而不真正实现
5. 继续让 SQL schema 多地维护
6. 继续让 docs 说 SOPS/Flux，代码却默认 `.env`

---

## 11. Immediate Next Step

本计划文档落地后，下一步不是继续讨论，而是按以下顺序开始实际修复：

1. 先做 WS-1，建立可运行的 secrets/config control plane
2. 再做 WS-2，统一 naming 和 ownership
3. 然后并行推进 WS-3、WS-4
4. 再收口 WS-5、WS-6
5. 最后完成 WS-7 验证与文档收敛

这会把 `counter-service` 从“代码可编译的样例”提升成“生产级 reference module”。
