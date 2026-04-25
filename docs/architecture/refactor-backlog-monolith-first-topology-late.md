# 单体优先、拓扑后置重构 Backlog

> 目的：为后续 agent 与开发者提供一份可直接接手的重构执行清单。
>
> 本文档只基于当前仓库真实代码与已落地目录结构给出建议，不把目标态当成当前事实。

## 1. 适用范围

本 backlog 面向以下目录的后续重构工作：

- `agent/**`
- `platform/model/**`
- `platform/schema/**`
- `services/**`
- `servers/**`
- `workers/**`
- `packages/contracts/**`
- `packages/messaging/**`
- `packages/runtime/**`
- `packages/data*/**`
- `packages/observability/**`

## 2. 当前架构结论

当前仓库最真实、最有参考价值的主链不是“完整微服务平台”，而是：

```text
service library
  -> web-bff
  -> CAS + idempotency + unified outbox
  -> outbox-relay
  -> projector
  -> replayable read model
```

当前默认参考锚点是 `counter-service`，证据链包括：

- `services/counter-service/model.yaml`
- `services/counter-service/src/application/service.rs`
- `services/counter-service/src/infrastructure/libsql_adapter.rs`
- `servers/bff/web-bff/src/handlers/counter.rs`
- `servers/bff/web-bff/src/state.rs`
- `workers/outbox-relay/src/main.rs`
- `workers/projector/src/main.rs`
- `docs/operations/counter-service-reference-chain.md`

因此，后续重构的第一原则不是继续扩张目标态抽象，而是把这条真实主链收紧、统一、去重、补齐。

## 3. 核心原则

后续所有重构默认遵循以下原则：

1. 业务语义固定在 `services/*` 库中，不固定在部署拓扑中。
2. 同步入口优先由 `servers/*` 组合；异步入口优先由 `workers/*` 组合。
3. 跨拓扑演进依赖 `shared contracts + unified outbox + replayable projection`，而不是依赖预先实现完整 runtime 抽象。
4. `counter-service` 是唯一默认生产参考链；其他服务在没有形成同等级闭环前，不应被描述为同等参考样例。
5. platform model 必须优先反映当前真实状态，再表达未来目标态。
6. 能通过 sidecar、运行时配置或未来 deployable 开关引入的能力，不应在当前单点主链中过早主链化。

## 4. 立即执行项

以下事项属于“现在不做，未来大概率要大改”的范围，应优先进入重构主线。

### B0. 统一默认参考链

目标：收敛默认架构叙事，避免多个半完成参考链并存。

当前问题：

- `counter-service` 已形成最真实闭环。
- `tenant-service` 的 `model.yaml` 语义很完整，但代码闭环显著弱于 `counter-service`。
- `runtime` ADR 已明确不是默认主路径，但目录与命名仍容易让人误解为核心总抽象。

证据：

- `services/counter-service/src/application/service.rs`
- `services/tenant-service/model.yaml`
- `services/tenant-service/src/application/service.rs`
- `services/tenant-service/src/policies/mod.rs`
- `docs/adr/003-runtime-abstraction-direct-plus-dapr.md`

交付标准：

1. 文档层明确声明 `counter-service` 是唯一默认生产参考链。
2. `tenant-service` 被标记为“语义参考”或“二级参考”，而不是默认复制模板。
3. `runtime` 被标记为“按需基础设施能力包”，不是默认后端主路径。

建议落点：

- `docs/README.md`
- `docs/operations/counter-service-reference-chain.md`
- `services/tenant-service/README.md` 或其 `model.yaml` 注释
- `packages/runtime/README.md` 或 `docs/adr/003-*`

### B1. 统一事件主链与 outbox canonical path

目标：确保领域事件只有一条标准发布路径。

当前问题：

- `packages/messaging/src/outbox/outbox_publisher.rs` 和 `workers/outbox-relay/src/publish/mod.rs` 同时承担 outbox 发布职责。
- `outbox-relay` 当前同时发布到 `EventBus` 与 `runtime::PubSub`，形成双轨。
- `packages/contracts/events/src/lib.rs` 又声明 `AppEvent` 是流经 `EventBus` 的统一事件类型。

证据：

- `packages/messaging/src/outbox/outbox_publisher.rs`
- `workers/outbox-relay/src/publish/mod.rs`
- `packages/contracts/events/src/lib.rs`
- `packages/messaging/src/lib.rs`

交付标准：

1. 明确 `event_outbox -> outbox-relay worker` 是唯一 canonical relay path。
2. 明确 `AppEvent/EventEnvelope` 是唯一领域事件载体。
3. 明确 `EventBus` 与 `runtime::PubSub` 的主从关系。
4. 非 canonical 实现被降级为测试/实验/本地模式，或被移除。

建议做法：

1. 先写一份短 ADR 或补充现有 ADR/README，声明 canonical path。
2. 再调整命名和模块注释，避免继续形成双主链认知。
3. 最后清理多余实现或降级其可见性。

### B2. 引入共享 worker reliability 基础件

目标：提前补齐未来多副本、多 Pod 时必然重构的可靠性语义。

当前问题：

- outbox relay checkpoint 存本地文件。
- projector checkpoint 存本地文件。
- dedupe/idempotency 仍偏本地进程内。
- 当前实现更适合单副本，而不是可平滑扩展的共享语义。

证据：

- `workers/outbox-relay/src/checkpoint/mod.rs`
- `workers/outbox-relay/src/dedupe/mod.rs`
- `workers/outbox-relay/src/idempotency/mod.rs`
- `workers/projector/src/checkpoint/mod.rs`

交付标准：

1. 引入统一的 checkpoint store trait。
2. 引入统一的 worker ownership / lease / dedupe / idempotency 存储抽象。
3. 默认生产实现落在共享存储，而不是本地文件。
4. 文件实现仅保留为本地开发 fallback。

建议落点：

- 新增 `packages/worker-runtime/**` 或 `packages/runtime-worker/**`
- 或在 `packages/messaging/**` / `packages/data-traits/**` 下收口，但必须只有一个归属点

不建议：

- 继续让每个 worker 复制自己的 checkpoint/recovery 基础件
- 直接引入重型协调系统作为当前单点主链前提

### B3. 定死 shared contracts 与 service-local 类型边界

目标：避免未来拆服务时回头重做 DTO、Event、ErrorCode 边界。

当前问题：

- `counter-service` 相对收敛。
- `tenant-service` 中 service-local event 与 target-state workflow 语义较多，和 shared contracts 边界不够统一。

证据：

- `services/counter-service/src/contracts/service.rs`
- `services/tenant-service/src/events/mod.rs`
- `packages/contracts/api/**`
- `packages/contracts/events/src/lib.rs`

统一规则：

1. 跨进程、跨 deployable、跨 HTTP/消息边界的 DTO、Event、ErrorCode 必须进入 `packages/contracts/**`。
2. 仅限 service 内部编排使用的 trait、context、辅助类型可留在 service crate 内。
3. 进入 outbox 的事件必须进入 shared contracts，而不是停留在 service-local events。

交付标准：

1. 在 `agent/codemap.yml` 或 docs 中增加这条显式规则。
2. 对 `tenant-service` 进行一次边界梳理，把需要共享的 event 升格。
3. 补对应 gate 或静态检查脚本，避免继续漂移。

### B4. 修正 deployable model 与真实二进制边界

目标：恢复 platform model 作为真实控制面的可信度。

当前问题：

- `platform/model/deployables/counter-service.yaml` 声称独立 deployable，但 `entry_point` 与 `package` 指向 `web-bff`。
- `edge-gateway` model 描述明显超前于真实实现。

证据：

- `platform/model/deployables/counter-service.yaml`
- `platform/model/deployables/edge-gateway.yaml`
- `servers/gateway/src/main.rs`

交付标准：

1. model 中区分 `current` 与 `target` 状态。
2. 未独立出 binary 的 deployable 不得伪装为已独立运行路径。
3. gateway 的能力描述收敛到已实现范围，或增加显式 `planned_capabilities` 字段。
4. 每个 deployable 都必须显式声明 `current_status` / `target_status` / `planned_capabilities`，即使目标数组为空。

建议做法：

1. 给 deployable schema 增加现实状态字段。
2. 给 validators 增加“entry_point/package 与 deployable status 一致性”检查。

### B5. 把 composition root 与具体 adapter 逐步从 service 内迁出

目标：让 `services/*` 真正成为纯业务能力库。

当前问题：

- `services/counter-service/src/lib.rs` 已明确承认 `infrastructure/` 是临时保留。
- `servers/bff/web-bff/src/state.rs` 直接从 service crate 中 new concrete repository。

证据：

- `services/counter-service/src/lib.rs`
- `servers/bff/web-bff/src/state.rs`
- `services/tenant-service/src/lib.rs`

交付标准：

1. 具体 adapter 不再默认由 service crate 暴露。
2. server/worker 成为唯一 composition root。
3. service 只依赖 port/trait，不承担 wiring 责任。
4. `state.rs` 只保留状态与访问接口；bootstrap/wiring 集中到单独模块。

建议做法：

1. 先从 `counter-service` 开始，把 repository adapter 迁往共享 adapter 层或 server-owned composition 层。
2. 再让 `tenant-service` 对齐同样模式。
3. 在 `services/*/src/lib.rs` 中移除“临时保留 infrastructure”说明。

## 5. 近期执行项

以下事项重要，但可以在立即执行项稳定后推进。

### B6. 收敛 worker 模板，抽统一 worker runtime

目标：避免后续 `indexer`、`scheduler`、`sync-reconciler` 复制可靠性基础件。

建议内容：

- health/readiness helper
- retry/backoff helper
- checkpoint trait 与存储实现
- lag/metrics helper
- dedupe/idempotency helper
- graceful shutdown / cancellation helper

推荐方式：

- 抽轻量共享包，不抽象成“全能 runtime 平台”
- 只吸收 `outbox-relay` 与 `projector` 已被真实使用的共性

### B7. 分层收紧 Rust 工程质量门禁

目标：从“模板宽松态”过渡到“主链生产级”。

当前问题：

- workspace lint 仍允许较多未使用项与死代码。

证据：

- `Cargo.toml` 中 `workspace.lints` 相关配置

建议顺序：

1. 先收紧 `counter-service`、`web-bff`、`outbox-relay`、`projector`、`packages/contracts/*`、`packages/messaging`
2. stub crate 暂时保留宽松
3. 最后再统一全仓收紧

本轮最小闭环：先把 `unused_imports` / `unused_variables` deny 扩到 reference-chain crates 与 validator/gate 入口，再用 focused `cargo check/test/clippy` 兜底。

### B8. 引入更明确的 bootstrap / composition 模式

目标：把 topology 差异封装在组合根，而不是散落在 handlers 或 services。

建议方向：

- `AppBootstrap`
- `BffCompositionRoot`
- `WorkerBootstrap`

交付标准：

1. `main.rs` 更薄。
2. wiring、配置加载、adapter 构造集中到单一模块。
3. embedded / remote / test / future distributed 的差异只改 bootstrap，不改 service 语义。
4. composition root 由 server/worker 自己拥有，不再散落在 handler 辅助函数里临时构造。

## 6. 延后项

以下事项暂不建议主链化，适合保留在 ADR、`mise.toml` 注释、platform model 或目标态文档中。

### D1. 完整 runtime 8-port 体系继续扩张

原因：

- `docs/adr/003-runtime-abstraction-direct-plus-dapr.md` 已明确该体系不是当前默认主链。
- 当前真实主链主要依赖 service ports、contracts、outbox、workers，而不是 runtime 总抽象。

建议：

- runtime 保留为按需基础设施能力包
- 不继续把 invocation/workflow/lock/secret/queue 全面主链化

### D2. Dapr / sidecar / mesh 抽象

原因：

- 它们是部署与运行时策略，不是当前单点主链的必要代码前提。

建议：

- 保留在 ADR、platform model、`mise.toml` 注释、目标态文档中
- 等真实部署路径需要时，再通过 adapter/sidecar 引入

### D3. tenant durable workflow 平台化实现

原因：

- `tenant-service` 当前语义描述明显超前于真实代码链路。
- 现在强行补 durable workflow runner，容易形成新一轮半成品主链。

建议：

- 等 counter async 主链与 worker reliability 收敛后再做

### D4. 全能 edge gateway 能力主链化

原因：

- 当前 gateway 真实实现仍是轻量反向代理。
- 过早把限流、鉴权旁路、复杂 edge policy 推进主链，会分散主链收敛资源。

### D5. 更多 stub service / worker 提前“正规军化”

原因：

- 目前 `auth-service`、`indexer`、`scheduler`、`sync-reconciler` 等仍偏 stub。
- 不适合在当前阶段进入默认参考架构集合。

## 7. 推荐执行顺序

建议按以下顺序推进，避免多条链同时重构：

1. B0 统一默认参考链与文档叙事
2. B1 统一 outbox canonical path 与事件主链
3. B2 引入共享 worker reliability 基础件
4. B3 收敛 contracts 边界与事件提升规则
5. B4 修正 deployable model 的现实状态表达
6. B5 迁移 composition root 与 adapter 边界
7. B6 统一 worker runtime 模板
8. B7 收紧参考主链 lint/gate
9. B8 收口 bootstrap/composition 模式

## 8. 对 agent 的执行要求

后续 agent 在执行本 backlog 时，应遵守：

1. 先沿 `counter-service` 参考链找证据，再扩展到其它模块。
2. 不要把目标态文档直接当成当前代码事实。
3. 不要同时发起 runtime、workflow、gateway、tenant、workers 多线大重构。
4. 每次优先完成一个闭环：文档规则 -> 代码实现 -> gate/验证。
5. 除非本项明确要求，否则不要为了“看起来先进”而新增大而全抽象层。

## 9. 完成定义

当以下条件同时满足时，可视为本轮架构收敛基本达标：

1. 新人或新 agent 只读 `AGENTS.md`、`docs/README.md`、本 backlog、counter reference chain，即可判断默认主链。
2. `event_outbox` 到异步消费链路只有一条明确 canonical path。
3. worker 的 checkpoint/dedupe/idempotency 具备共享语义，不再默认依赖本地文件与单进程内存。
4. platform model 不再把未来 deployable 状态伪装成当前运行事实。
5. `services/*` 与 `servers/*` / `workers/*` 的 wiring 边界清晰，service library 纯度提升。
