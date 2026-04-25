# 后端执行计划

> 目标：在不改变后端优先、`counter-service` 作为默认参考锚点的前提下，把当前项目推进到“单节点近生产可验证”的稳定阶段，并明确哪些能力继续推进、哪些能力只保留到 `.mise.toml` 注释里。
>
> 状态：active plan
>
> 适用场景：按阶段推进后端主链、控制平台扩张节奏、避免未来工具选型散落到多份文档污染 agent 上下文。
>
> 重要：这是一份执行计划，不是当前实现状态真理源。涉及当前事实判断时，以代码、`docs/README.md`、`docs/operations/counter-service-reference-chain.md` 与 validators/gates 为准。

## 1. 计划定位

这份计划不是未来愿景稿，也不是大而全的 gap 散文。

它只回答 6 个问题：

1. 当前项目处于什么阶段。
2. 接下来必须继续推进什么。
3. 接下来明确不要推进什么。
4. 每个阶段的完成标准是什么。
5. 每个阶段主要修改哪些目录。
6. 未来生态工具选型应该记录到哪里。

## 2. 当前工作判断

当前项目状态在计划层应统一理解为：

1. `counter-service` 已经不是 demo，而是当前最接近生产参考链的默认锚点。
2. 业务主链已经形成：`web-bff -> counter-service -> outbox -> relay -> projector`。
3. 工程横切链已经有真实落点：`platform model -> SOPS -> Kustomize -> Flux -> overlay -> validators`。
4. 当前最合理目标不是直接追多节点 HA，而是先把“单节点近生产可验证”收敛完整。
5. 未来多节点会影响代码形状的分布式语义，其中真正会影响重构成本的部分需要尽早定住。
6. 不会改变当前代码形状的平台能力，不应继续扩张为新的活文档和新上下文入口。

一句话：

1. 继续推进会决定代码形状的语义能力。
2. 继续推进单节点也能真实验证的生产主链。
3. 停止扩张只会增加平台复杂度、但当前不会降低重构成本的能力。

## 3. 硬边界

### 3.1 必须继续推进

以下内容必须继续推进，因为后补代价高：

1. 身份、租户、actor、resource 的统一建模。
2. 事件 envelope、correlation、causation、trace 字段规范。
3. NATS subject taxonomy、event naming、consumer naming。
4. Outbox / idempotency / replay / checkpoint 的统一语义。
5. `projector` 的 durable / replay 目标边界。
6. 数据 ownership 边界，避免把 shared DB 路径误当长期默认架构。
7. 单节点近生产下 shared remote DB 的真实运行链路。

### 3.2 只做到单节点近生产

当前阶段目标统一卡在这里：

1. 单节点 K3s 或等价单节点集群可运行。
2. `web-bff`、`outbox-relay-worker`、`projector-worker` 可独立进程运行。
3. SOPS / Kustomize / Flux / overlay / shared remote DB 路径可验证。
4. OTel / Collector / OpenObserve 的最小闭环可验证。
5. 供应链最小基线可验证：SBOM、扫描、签名、压测。

### 3.3 当前明确不推进

以下内容当前不作为执行计划目标：

1. Service mesh 正式引入。
2. External Secrets / Vault 正式接入。
3. 多节点 NATS / JetStream HA。
4. 多节点 Turso / libSQL HA。
5. progressive delivery controller 正式接入。
6. cert-manager 正式接入。
7. Kyverno / Gatekeeper enforce。
8. 任何新的“未来选型说明文档”。

## 4. 唯一登记处

未来所有生态选型，无论是否现在启用，都遵守以下规则：

1. 可由开发机安装的 CLI 工具，统一只登记到 `.mise.toml`。
2. 未来可能使用、但当前不启用的 CLI，统一只写在 `.mise.toml` 注释区。
3. 未来可能使用、但不适合通过 `mise` 安装的服务型组件，也统一只写在 `.mise.toml` 注释区。
4. 不再新增单独的“未来工具选型说明.md”。
5. `docs/` 只保留执行计划、reference chain、owner 文档和必要的运行说明。

这意味着：

1. `.mise.toml` 是未来生态选型的唯一登记处。
2. `docs/` 不是未来生态选型清单仓库。
3. 当前计划只描述“现在做什么”，不重复维护完整未来工具目录。

## 5. 阶段计划

### Phase 1：语义定桩

目标：冻结未来分布式演进代价最高的语义边界。

本阶段要完成：

1. 统一 `actor / tenant / subject / resource` 建模。
2. 统一事件 envelope 元数据。
3. 统一 `trace_id / span_id / correlation_id / causation_id / tenant_id / actor_id` 传播规则。
4. 统一 NATS subject taxonomy。
5. 明确 `projector` 的 durable / replay 语义目标。
6. 明确 shared DB 只是当前单节点近生产承载手段，不是长期默认共享库架构。

主要修改目录：

1. `packages/contracts/events/**`
2. `packages/messaging/**`
3. 按需涉及 `packages/runtime/**` 中仍被当前主链真实使用的部分
4. `services/counter-service/**`
5. `servers/bff/web-bff/**`
6. `workers/projector/**`

建议验证：

1. `cargo test -p counter-service -p outbox-relay-worker -p projector-worker`
2. `just boundary-check`
3. `just verify-replay strict`

完成标准：

1. `counter` 链所有同步与异步路径都使用统一 envelope 元数据。
2. replay / idempotency / correlation 字段已收口，不再散落。
3. `projector` 的 live path 和 replay path 边界明确。
4. auth 平台即使未部署，actor/resource/tenant 模型也已固定。

停止线：

1. 本阶段不接 Zitadel 实例。
2. 本阶段不接 OpenFGA 实例。
3. 本阶段不做 mesh、HA、外部 secret。

### Phase 2：单节点近生产主链闭环

目标：把 `counter-service` 推进到单节点近生产可验证状态。

本阶段要完成：

1. 核实 shared remote DB secret 的真实值。
2. 启用 `outbox-relay-worker` 独立副本。
3. 启用 `projector-worker` 独立副本。
4. 让 `web-bff -> remote/shared DB -> relay -> projector` 成为默认 cluster path。
5. 弱化旧 `.env + compose` 单机叙事，把单节点 K3s 作为近生产主线。

主要修改目录：

1. `infra/security/sops/**`
2. `infra/k3s/**`
3. `infra/gitops/flux/**`
4. `workers/outbox-relay/**`
5. `workers/projector/**`
6. 必要时 `docs/operations/**`

建议验证：

1. `just validate-platform`
2. `just validate-topology`
3. `just verify-replay strict`
4. `just boundary-check`
5. 对独立 worker 做最小 smoke 验证

完成标准：

1. 单节点环境下 `web-bff` 使用 shared remote DB 路径。
2. `outbox-relay-worker` 可独立运行并真实发布。
3. `projector-worker` 可独立运行并真实 replay / live tail。
4. health / ready / restart recovery 可验证。

停止线：

1. 本阶段不做多节点 K3s。
2. 本阶段不做 JetStream 集群复制。
3. 本阶段不做 Turso 多节点复制。

### Phase 3：观测闭环

目标：形成最小但真实可用的观测主链，而不是继续堆叠观测平台。

本阶段要完成：

1. 正式补齐 OTel Collector 层。
2. 收敛 Rust tracing 到 OTel 的统一出口。
3. 接通 OpenObserve。
4. 保留 Vector 作为日志处理与路由层。
5. 为 `counter-service` 参考链定义最小查询与排障路径。

主要修改目录：

1. `packages/observability/**`
2. `ops/observability/otel/**`
3. `ops/observability/vector/**`
4. `infra/docker/compose/observability.yaml`
5. 少量 `servers/**`、`workers/**`

建议验证：

1. `just test`
2. 最小链路 smoke：HTTP 请求到 projector apply 的 trace / log 可追
3. 观测栈本地启动验证

完成标准：

1. 一次请求可跨 `handler -> service -> outbox -> relay -> projector` 关联定位。
2. log / trace 的 correlation 维度统一。
3. OpenObserve 可作为默认查询入口。

停止线：

1. 不引入第二套观测栈。
2. 不追求复杂 dashboard 全家桶。
3. 不做多集群观测设计。

### Phase 4：性能与供应链最小闭环

目标：让“单节点近生产”具备最小可验证的容量与供应链基线。

本阶段要完成：

1. 引入 `k6` 基线压测。
2. 引入 `Trivy` 漏洞扫描。
3. 引入 `Syft` 生成 SBOM。
4. 引入 `Cosign` 镜像签名的最小路径。
5. 把这些能力收口到 `just` / CI / 本地统一命令入口。

主要修改目录：

1. `.mise.toml`
2. `justfiles/**`
3. `.github/**` 或等价 CI 路径
4. 必要的 `docs/operations/**`

建议验证：

1. `just` 入口可触发最小压测
2. 镜像或构建产物可完成扫描
3. SBOM 可生成
4. 签名链路至少可在本地或 CI 演练

完成标准：

1. 供应链检查不再完全依赖人工。
2. 压测脚本成为 repo 一等公民。
3. `.mise.toml` 成为 installable CLI 的统一入口。

停止线：

1. 本阶段不做 Kyverno enforce。
2. 本阶段不做 admission attestation。
3. 本阶段不做 rollout controller。

### Phase 5：身份与授权正式接入

目标：在语义定桩完成后，再把 auth 平台接进默认链路。

> 2026-04 当前仓库内进度：`counter` 路径已真实走 `AuthzPort`，`web-bff` 已支持按配置切换 `MockAuthzAdapter` / `OpenFGA` adapter，且 `tenant/init` 会写入最小授权元组。
> 仍未闭环的部分是外部系统事实：真实 OpenFGA store / model provisioning、Zitadel OIDC/JWKS 对接与凭据、以及相应环境级 secret/cluster 验证。

本阶段要完成：

1. 先接 OpenFGA 模型与 adapter。
2. 再接 Zitadel token / claims 到 actor 映射。
3. 保留 `MockOAuthProvider` 作为 dev fallback。
4. 明确哪些授权数据进 token，哪些通过 FGA check。
5. 至少打通一个真实多租户授权闭环。

主要修改目录：

1. `packages/authn/**`
2. `packages/authz/**`
3. `services/auth-service/**`
4. `servers/bff/web-bff/**`
5. 对应 `infra/**` 路径

建议验证：

1. auth integration tests
2. 最小 multi-tenant authz smoke
3. `just boundary-check`

完成标准：

1. actor / tenant / resource / relation 已接入真实链路。
2. OpenFGA / Zitadel 从“选型”变成“集成”。
3. `counter-service` 与至少一个多租户场景可跑通授权检查。

停止线：

1. 本阶段不追求全量 IAM 平台能力。
2. 本阶段不追求 auth 平台自身 HA。

## 6. 延后项

以下能力统一视为延后项，只登记到 `.mise.toml` 注释区，不进入当前活计划：

1. Gateway API 正式清单迁移。
2. cert-manager。
3. Kyverno / Gatekeeper。
4. External Secrets / Vault。
5. Linkerd / Cilium service mesh。
6. Flagger / Argo Rollouts。
7. 多节点 NATS / JetStream。
8. 多节点 Turso / libSQL。

延后规则：

1. 可以记录选型结论。
2. 不进入当前阶段目标。
3. 不扩展为新的活文档。
4. 不新增散乱 ADR 以外的未来说明稿。

## 7. 执行顺序

当前推荐执行顺序只有这一条：

1. `Phase 1` 语义定桩
2. `Phase 2` 单节点近生产主链闭环
3. `Phase 3` 观测闭环
4. `Phase 4` 性能与供应链最小闭环
5. `Phase 5` 身份与授权正式接入

不建议插队：

1. 不要先做 mesh。
2. 不要先做 ESO / Vault。
3. 不要先做 rollout controller。
4. 不要先做多节点 HA。

## 8. 文档与上下文约束

为了避免继续污染 agent 默认上下文，后续执行必须遵守：

1. 当前计划是 `docs/` 下唯一活跃的分阶段执行计划。
2. 未来 installable / non-installable 生态选型只登记到 `.mise.toml` 注释区。
3. `docs/operations/counter-service-reference-chain.md` 只记录真实现状，不维护未来工具愿景。
4. `docs/operations/*.md` 只保留当前运行与交付所需说明。
5. 不再新增平行的“工具选型清单”或“未来平台路线图”。

## 9. 一句话结论

当前项目接下来的正确推进方向，不是继续横向扩平台，而是把 `counter-service` 收敛成：

1. 语义已定桩。
2. 单节点近生产可验证。
3. 观测与供应链最小闭环成立。
4. 未来平台能力只在 `.mise.toml` 注释里登记，暂不扩张为新的活文档。
