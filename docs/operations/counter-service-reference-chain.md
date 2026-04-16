# Counter-Service Reference Chain

> 目的：把 `counter-service` 定义为本仓库后端开发的默认参考锚点。
>
> 它不是“最小 demo”，而是“业务最小、工程链路尽量完整”的生产参考样例。
>
> 状态声明：当前这条链路已经具备较完整的参考价值，但仍有未闭环部分。本文档只描述已确认的事实、明确的目标态，以及这些目标态当前缺失在哪里。

## 1. 如何使用这份文档

这份文档面向两类任务：

1. agent 或开发者需要快速理解本仓库后端应如何新增 service/server/worker。
2. agent 或开发者需要判断某个后端能力是否已经进入默认生产链路。

阅读顺序建议：

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `docs/architecture/repo-layout.md`
4. 本文档
5. 对应代码与 platform/model 文件

## 2. 核心定位

`counter-service` 是本仓库的默认后端参考锚点，承载两条链路：

1. 业务主链：最小业务能力如何按 DDD、contracts、server、worker、projection 实现。
2. 工程横切链：一个后端能力如何进入 secrets、deploy、GitOps、promotion、runbook、gate、CI 等生产轨道。

这意味着：

1. counter 的业务逻辑可以保持简单。
2. 但它不能因为简单，就省略后续新服务几乎必经的生产工具链。
3. 如果某个高价值横切能力尚未通过 counter 链路显式落地，它就还不能算仓库的默认工程惯性。

## 3. 当前事实状态

### 3.1 已经存在的参考链路部分

当前仓库中，counter 参考链路已经具备：

1. `services/counter-service/model.yaml` 作为 service-local semantics 真理源。
2. `services/counter-service/src/**` 的 DDD 结构。
3. `packages/contracts/**` 中的 counter DTO / event / error 相关契约。
4. `servers/bff/web-bff/src/handlers/counter.rs` 的同步入口样例。
5. `services/counter-service` 内的 CAS、idempotency、outbox write 样例。
6. `workers/outbox-relay` 作为异步发布链路样例。
7. `workers/projector` 作为 projection worker 样例。
8. `platform/model/services/counter-service.yaml` 与 ownership mapping。
9. 与 counter 相关的 SOPS 模板、Kustomize overlay、Flux app 定义的初步落点。

### 3.2 尚未完全闭环的部分

截至当前代码状态，这条链路还没有完全达到“生产完备参考样例”：

1. `projector` 已补到 `outbox replay + optional NATS live tail`，但仍不是最终事件平台形态。
2. `counter-service` 在当前阶段仍主要嵌入 `web-bff` 进程内，而不是独立 deployable 主路径。
3. counter 相关的 secrets / GitOps / promotion / runbook 已有落点，但尚未形成完整 admission 闭环。
4. 独立 worker 一旦进入多 Pod/k3s 交付路径，就必须把数据库 secret 切到 shared libSQL/Turso，而不能继续依赖本地 `file:` 数据库路径。
5. platform model、schema、ownership 命名仍有部分漂移需要清理。

所以对这条链路的正确理解是：

1. 它是当前最重要的参考样例。
2. 它不是已经完全收敛的最终生产模板。
3. 后续所有后端基建、gate、文档瘦身，都应优先围绕它补齐，而不是另外新建一套平行样例。

## 4. 参考链路总览

```text
client/request
  -> web-bff handler
  -> counter-service application service
  -> repository + CAS + idempotency + outbox write
  -> counter_outbox table
  -> outbox-relay worker
  -> event backbone
  -> projector worker / indexer / downstream consumers
  -> read model / replay / rebuild path

同时并行存在一条工程横切链：

platform model
  -> deployables / ownership / topology / validators
  -> secrets templates + encrypted artifacts (SOPS)
  -> kustomize overlay / rendered manifests
  -> Flux GitOps reconciliation
  -> gate / CI / drift / runbook / promotion
```

## 5. 主业务链

### 5.1 Service 模型真理源

入口文件：`services/counter-service/model.yaml`

它应该回答：

1. counter 实体归谁拥有。
2. 接受哪些 command。
3. 发布哪些 event。
4. 提供哪些 query。
5. consistency / idempotency / partitioning / failure behavior 是什么。

这是新增 service 时最先应看的真理源之一。

### 5.2 DDD 结构样例

主要路径：

1. `services/counter-service/src/domain/**`
2. `services/counter-service/src/application/**`
3. `services/counter-service/src/ports/**`
4. `services/counter-service/src/infrastructure/**`

这条结构表达的是：

1. domain 负责规则与对象。
2. application 负责 orchestration。
3. ports 定义外部依赖抽象。
4. infrastructure 负责具体存储与适配实现。

新增 service 时，优先复用这套分层，而不是从别的更复杂 service 抽象总结。

### 5.3 Contracts-first 样例

主要路径：

1. `packages/contracts/api/**`
2. `packages/contracts/events/**`
3. `packages/contracts/errors/**`

counter 链路要求：

1. handler 对外响应形状来自 shared contracts。
2. event payload 要有可复用契约表达。
3. contract drift 必须可被脚本和 CI 检测。

### 5.4 同步入口样例

主要路径：

1. `servers/bff/web-bff/src/handlers/counter.rs`
2. `servers/bff/web-bff/src/main.rs`

当前 counter 的同步入口承担的参考意义：

1. 如何在 BFF 内部消费 service。
2. 如何做 DTO-first handler。
3. 如何做 tenant context 与错误映射。
4. 如何在 mutation 后做 cache invalidation。

注意：

1. 当前 counter-service 主要以内嵌库的方式由 `web-bff` 组合使用。
2. 因此它目前更适合做“server 如何组合 service”的样例，而不是“service 已经独立进程化”的最终样例。

### 5.5 持久化、CAS、Idempotency、Outbox 样例

主要路径：

1. `services/counter-service/src/application/service.rs`
2. `services/counter-service/src/infrastructure/libsql_adapter.rs`
3. `services/counter-service/migrations/001_create_counter.sql`

当前它承载的默认实践：

1. mutation 前先做 idempotency check。
2. mutation 通过 CAS/版本校验防止并发覆盖。
3. 成功写主状态后写 outbox event。
4. idempotency、主状态、outbox 属于同一条参考链上的核心数据结构。

## 6. 异步链路

### 6.1 Outbox Relay

主要路径：

1. `workers/outbox-relay/src/main.rs`
2. `workers/outbox-relay/src/polling/**`
3. `workers/outbox-relay/src/publish/**`
4. `workers/outbox-relay/src/checkpoint/**`
5. `workers/outbox-relay/src/idempotency/**`

它承载的目标态样例：

1. 读取 outbox。
2. 去重与幂等处理。
3. 发布到消息骨干。
4. 标记已发布。
5. checkpoint 与恢复。

当前真实状态：

1. 已具备相当明确的 worker 结构。
2. 默认 reader 已改为真实 `counter_outbox` 数据库读取，不再把数据库失败静默降级为 in-memory stub。
3. 事件发布侧已切到真实 NATS adapter，不再仅停留在内存 event bus / pubsub。
4. 已补到独立的 dev secret / kustomize / Flux 落点，并可通过 `counter-shared-db` secret 接入 shared libSQL/Turso；但当前默认仍保持 `replicas=0`，直到已加密 secret 的真实值被核实后再启用。

因此新增 worker 时：

1. 可以复用它的结构与职责划分。
2. 不能假定它已经代表最终的生产发布路径。

### 6.2 Projector

主要路径：

1. `workers/projector/src/main.rs`
2. `workers/projector/src/consumers/**`
3. `workers/projector/src/readmodels/**`
4. `workers/projector/src/checkpoint/**`
5. `workers/projector/src/replay/**`

它承载的目标态样例：

1. replayable event source 消费。
2. rebuildable read model。
3. projection checkpoint。
4. lag / health / recovery。

当前真实状态：

1. projector 已从纯骨架升级为真实 `counter_outbox` replay source。
2. 已具备持久化 `counter_projection` read model 与磁盘 checkpoint。
3. 当配置 `PROJECTOR_NATS_URL` 时，可从 `events.counter.changed` subject 继续做 live tail，并默认通过 queue group 降低多副本重复消费。
4. 已补到独立的 dev secret / kustomize / Flux 落点，并可通过 `counter-shared-db` secret 接入 shared libSQL/Turso；但当前默认仍保持 `replicas=0`，直到已加密 secret 的真实值被核实后再启用。
5. 它当前仍不是最终的独立消息骨干订阅平台实现，也不提供 durable broker checkpoint 语义。

因此本文把它视为“已经进入真实最小闭环、但仍需继续补齐交付链路”的参考环节。

## 7. 平台模型链路

### 7.1 Service 元数据

主要文件：

1. `platform/model/services/counter-service.yaml`
2. `platform/model/state/ownership-map.yaml`

它们表达：

1. counter-service 是 reference-service。
2. `counter` 的 owner 是 `counter-service`。

当前仍需注意：

1. ownership map 中其他 service 命名还存在 `tenant` / `auth` / `user` / `indexing` 这类与目录名不完全一致的漂移。
2. 这说明平台模型链路还需要整体收敛，不能只修 counter 一处就宣称完全一致。

### 7.2 Deployables

主要文件：

1. `platform/model/deployables/web-bff.yaml`
2. `platform/model/deployables/outbox-relay-worker.yaml`
3. `platform/model/deployables/projector-worker.yaml`

当前 counter 参考链路通过这些 deployable 表达：

1. 同步入口由 `web-bff` 承载。
2. 异步 relay 由 `outbox-relay-worker` 承载。
3. projection 由 `projector-worker` 承载。

当前未完全到位的部分：

1. `counter-service` 自己的独立 deployable 主路径还没有真正进入默认链路。
2. 但 SOPS 模板已经为独立 deployable 的未来路径预留了位置。

## 8. 生产工具链横切链

这是本文最重要的补充部分。

### 8.1 为什么必须把横切链挂到 counter

如果 `counter-service` 只展示业务代码链路，那么以下高价值能力就会从默认学习路径里消失：

1. secrets 管理
2. 环境提升
3. GitOps/Flux
4. deployable 与 topology
5. drift 检查
6. rollback / runbook

所以 counter 参考链必须把这些能力显式挂接进来，即使当前仍有部分未闭环。

### 8.2 Secrets / SOPS

主要文件：

1. `justfiles/sops.just`
2. `infra/security/sops/templates/dev/web-bff.yaml`
3. `infra/security/sops/templates/dev/outbox-relay-worker.yaml`
4. `infra/security/sops/templates/dev/projector-worker.yaml`
5. `infra/security/sops/templates/dev/counter-shared-db.yaml`
6. `infra/security/sops/templates/dev/counter-service.yaml`
6. `infra/security/sops/dev/outbox-relay-worker.enc.yaml`
7. `infra/security/sops/dev/projector-worker.enc.yaml`

当前真实状态：

1. 仓库已经明确规定后端默认通过 SOPS/Kustomize/Flux 注入环境变量，而不是通过 `.env` 文件。
2. `web-bff`、`outbox-relay-worker`、`projector-worker` 都已经有 dev secret 模板与加密文件落点。
3. `counter-shared-db` 已作为独立 secret 模板存在，用来把 `web-bff` 与独立 worker 指向同一个远程 libSQL/Turso。
4. `web-bff` dev overlay 已显式消费 `counter-shared-db` secret，使 cluster 路径优先走远端 Turso，而本地 `just sops-run web-bff` 仍可保留嵌入式 fallback。
5. `projector-worker` 与 `outbox-relay-worker` 的独立 dev overlay 已接入 `counter-shared-db` secret，但默认仍保持副本数为 0，避免在 secret 真实值未核实前伪造多 Pod 闭环。
6. `counter-service` 本身也有 dev template，但注释明确说明 Phase 0 仍嵌入 `web-bff`，独立 deployable 属于 Phase 1+。

这意味着：

1. SOPS 已经进入 counter 参考链。
2. `projector-worker` 已具备独立 secret artifact 与 overlay/Flux 入口。
3. 但 counter-service 自身的独立 secret artifact 还不是默认运行主路径。

### 8.3 Kustomize / Overlay / Environment

主要文件：

1. `infra/k3s/overlays/dev/kustomization.yaml`
2. `infra/k3s/overlays/dev/projector-worker/kustomization.yaml`
3. `infra/k3s/base/configmap-projector-worker.yaml`
4. `infra/k3s/overlays/dev/outbox-relay-worker/kustomization.yaml`

当前真实状态：

1. dev overlay 已消费 `web-bff` 的加密 secret，并额外显式挂接 `counter-shared-db` 作为 shared remote DB 入口。
2. `outbox-relay-worker` 与 `projector-worker` 都已有独立 dev overlay，且 overlay 内已挂接 `counter-shared-db` secret。
3. 二者当前仍默认保持 `replicas=0`，因为当前环境还不能从已加密 secret 直接证明远端 DB 凭证已经就绪。
4. `counter-service.enc.yaml` 仍被注释掉，说明它尚未成为独立 deployable 的默认部署路径。

因此新增服务若要进入目标态：

1. 必须在 counter 现有模式基础上补齐自己的 overlay 与 secret 绑定。
2. 不能绕开这条路径重新发明另一套环境注入方式。

### 8.4 Flux / GitOps

主要文件：

1. `infra/gitops/flux/apps/web.yaml`
2. `infra/gitops/flux/apps/projector-worker.yaml`
3. `infra/gitops/flux/apps/outbox-relay-worker.yaml`
4. `docs/operations/gitops.md`

当前真实状态：

1. `web-bff` 的 Flux Kustomization 已存在。
2. 该 Kustomization 已包含 SOPS decryption 配置。
3. `outbox-relay-worker` 与 `projector-worker` 的 Flux Kustomization 都已存在，并指向各自独立的 dev overlay。
4. 二者当前仍默认保持 `replicas=0`。

当前缺口：

1. `indexer-worker` 等其余 worker 的 Flux app 映射尚未全部补齐。
2. `outbox-relay-worker` 与 `projector-worker` 虽然已具路径，但默认关闭，说明 counter 的 GitOps 链路仍未完整覆盖所有环节。

### 8.5 Delivery / Promotion / Drift / Runbook

当前仓库中，这部分能力已经有若干落点，但尚未围绕 counter 形成统一入口：

1. `just verify-generated`、`drift-check`、`sdk-drift-check`
2. `docs/operations/gitops.md`
3. `docs/operations/secret-management.md`
4. `ops/**` 与 `infra/**` 下的运维与交付文件

当前缺口：

1. 还没有“counter delivery gate”将这些生产治理信号统一收口。
2. 因此 agent 还不能只靠 gate/CI 就完整学到 delivery/promotion/runbook 路径。

## 9. 默认学习地图

后续 agent 若要以 counter 作为默认参考链路，应按以下路径学习：

### 9.1 看业务主链

1. `services/counter-service/model.yaml`
2. `services/counter-service/src/**`
3. `packages/contracts/**`
4. `servers/bff/web-bff/src/handlers/counter.rs`
5. `workers/outbox-relay/src/**`
6. `workers/projector/src/**`

### 9.2 看平台与交付链

1. `platform/model/services/counter-service.yaml`
2. `platform/model/deployables/web-bff.yaml`
3. `platform/model/deployables/outbox-relay-worker.yaml`
4. `platform/model/deployables/projector-worker.yaml`
5. `justfiles/sops.just`
6. `infra/security/sops/templates/dev/web-bff.yaml`
7. `infra/security/sops/templates/dev/outbox-relay-worker.yaml`
8. `infra/security/sops/templates/dev/projector-worker.yaml`
9. `infra/security/sops/templates/dev/counter-shared-db.yaml`
10. `infra/security/sops/templates/dev/counter-service.yaml`
11. `infra/k3s/overlays/dev/kustomization.yaml`
12. `infra/k3s/overlays/dev/outbox-relay-worker/kustomization.yaml`
13. `infra/k3s/overlays/dev/projector-worker/kustomization.yaml`
14. `infra/gitops/flux/apps/web.yaml`
15. `infra/gitops/flux/apps/outbox-relay-worker.yaml`
16. `infra/gitops/flux/apps/projector-worker.yaml`

## 10. 当前明确缺口

当前若要把 counter 参考链真正提升为“最小生产链路完备样例”，优先需要补齐：

1. 把 `web-bff` 的集群配置入口也完全收敛到真实 `APP_*` secret/key 形状，并与 shared counter DB secret 串成同一条主链。
3. 为 counter 链路建立统一的 delivery / GitOps / promotion / runbook admission 检查。
4. 修正 platform model / schema / ownership 的命名漂移。
5. 明确 counter-service 从嵌入式库到独立 deployable 的演进路径，并让该路径受 gate 约束。

## 11. 这份文档不做什么

本文档不承担以下职责：

1. 不替代 `AGENTS.md` 与 `codemap.yml` 的仓库级规则说明。
2. 不替代局部 README 的 owner 说明。
3. 不把未实现能力包装成既成事实。
4. 不要求 counter 业务本身承载所有复杂性。

它只负责一件事：

1. 明确告诉 agent 与开发者，当前仓库后端的默认参考锚点在哪里。
2. 明确这条锚点链路已实现了什么、还缺什么、横切生产工具链如何挂在这条链上。

## 12. 一句话结论

`counter-service` 现在不是“已经完工的终态模板”，而是“必须继续优先补齐、并最终承载完整后端生产工具链默认路径的参考锚点”。

后续新增后端能力、精简文档、收敛 gate/CI，都应优先围绕这条链路推进，而不是绕开它另起炉灶。
