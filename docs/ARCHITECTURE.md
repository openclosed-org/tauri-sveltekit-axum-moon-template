下面是压缩后的 **可复制 monorepo 目录树模板**，以及可直接放进仓库文档/Agent 约束中的 **硬性规则集**。

---

# 1) 可复制目录树模板

> 这版是“最终态骨架”，不是“当前最少文件数”。
> 原则是：**先把位置留对，再决定哪些模块默认启用、哪些预埋关闭。**

```text
boilerplate/
│
├── .mise.toml
├── justfile
├── moon.yml
├── Cargo.toml
├── rust-toolchain.toml
├── package.json
├── bun-workspace.yaml
├── biome.json
├── clippy.toml
├── deny.toml
├── typos.toml
├── .editorconfig
├── .gitattributes
├── .gitignore
│
├── .cargo/
│   ├── config.toml
│   └── audit.toml
│
├── .config/
│   └── nextest.toml
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── contracts.yml
│       ├── images.yml
│       ├── gitops-preview.yml
│       ├── release.yml
│       ├── security.yml
│       └── perf.yml
│
├── agent/
│   ├── README.md
│   ├── codemap.yml
│   ├── boundaries.md
│   ├── constraints/
│   │   ├── dependencies.yaml
│   │   ├── patterns.yaml
│   │   ├── contracts.yaml
│   │   ├── storage-policy.yaml
│   │   ├── telemetry-policy.yaml
│   │   ├── authz-policy.yaml
│   │   └── topology-policy.yaml
│   ├── prompts/
│   │   ├── add-service.md
│   │   ├── add-worker.md
│   │   ├── add-bff.md
│   │   ├── add-resource.md
│   │   ├── add-contract.md
│   │   └── split-domain.md
│   ├── checklists/
│   │   ├── schema-change.md
│   │   ├── migration.md
│   │   ├── release.md
│   │   ├── incident.md
│   │   └── topology-change.md
│   └── templates/
│       ├── service/
│       ├── worker/
│       ├── bff/
│       ├── contract/
│       └── platform-model/
│
├── platform/
│   ├── README.md
│   ├── schema/
│   │   ├── service.schema.json
│   │   ├── deployable.schema.json
│   │   ├── resource.schema.json
│   │   ├── workflow.schema.json
│   │   ├── topology.schema.json
│   │   └── policy.schema.json
│   ├── model/
│   │   ├── services/
│   │   │   ├── user.yaml
│   │   │   ├── tenant.yaml
│   │   │   ├── settings.yaml
│   │   │   ├── counter.yaml
│   │   │   ├── admin.yaml
│   │   │   └── indexing.yaml
│   │   ├── deployables/
│   │   │   ├── web-bff.yaml
│   │   │   ├── admin-bff.yaml
│   │   │   ├── edge-gateway.yaml
│   │   │   ├── indexer-worker.yaml
│   │   │   ├── outbox-relay.yaml
│   │   │   ├── projector.yaml
│   │   │   ├── scheduler.yaml
│   │   │   └── sync-reconciler.yaml
│   │   ├── resources/
│   │   │   ├── turso.yaml
│   │   │   ├── nats.yaml
│   │   │   ├── cache.yaml
│   │   │   ├── object-storage.yaml
│   │   │   ├── authn-zitadel.yaml
│   │   │   ├── authz-openfga.yaml
│   │   │   ├── observability.yaml
│   │   │   ├── secrets.yaml
│   │   │   └── wasm-runtime.yaml
│   │   ├── workflows/
│   │   │   ├── tenant-onboarding.yaml
│   │   │   ├── invite-member.yaml
│   │   │   ├── passwordless-login.yaml
│   │   │   ├── sync-reconcile.yaml
│   │   │   └── projection-rebuild.yaml
│   │   ├── policies/
│   │   │   ├── timeout.yaml
│   │   │   ├── retry.yaml
│   │   │   ├── idempotency.yaml
│   │   │   ├── outbox.yaml
│   │   │   ├── tenancy.yaml
│   │   │   ├── authz.yaml
│   │   │   ├── telemetry.yaml
│   │   │   └── release.yaml
│   │   ├── topologies/
│   │   │   ├── local-dev.yaml
│   │   │   ├── single-vps.yaml
│   │   │   ├── split-edge-workers.yaml
│   │   │   ├── k3s-staging.yaml
│   │   │   └── k3s-microservices.yaml
│   │   └── environments/
│   │       ├── dev.yaml
│   │       ├── staging.yaml
│   │       └── prod.yaml
│   ├── generators/
│   │   ├── contracts/
│   │   ├── sdk/
│   │   ├── compose/
│   │   ├── kustomize/
│   │   ├── flux/
│   │   ├── docs/
│   │   └── graph/
│   ├── validators/
│   │   ├── model-lint/
│   │   ├── dependency-graph/
│   │   ├── contract-drift/
│   │   ├── topology-check/
│   │   ├── security-check/
│   │   └── observability-check/
│   └── catalog/
│       ├── services.generated.yaml
│       ├── deployables.generated.yaml
│       ├── resources.generated.yaml
│       ├── topology.generated.md
│       └── architecture.generated.md
│
├── apps/
│   ├── web/
│   │   ├── src/
│   │   │   ├── routes/
│   │   │   ├── lib/
│   │   │   │   ├── api/
│   │   │   │   ├── auth/
│   │   │   │   ├── tenancy/
│   │   │   │   ├── stores/
│   │   │   │   ├── sync/
│   │   │   │   └── components/
│   │   │   ├── hooks.server.ts
│   │   │   └── app.html
│   │   ├── tests/
│   │   └── package.json
│   ├── desktop/
│   │   ├── src/
│   │   │   └── lib/
│   │   │       ├── api/
│   │   │       ├── tauri/
│   │   │       ├── sync/
│   │   │       └── store/
│   │   ├── src-tauri/
│   │   │   ├── capabilities/
│   │   │   ├── src/
│   │   │   │   ├── commands/
│   │   │   │   ├── deep_link/
│   │   │   │   ├── updater/
│   │   │   │   └── main.rs
│   │   └── package.json
│   └── mobile/
│       ├── src/lib/api/
│       ├── src/lib/sync/
│       └── package.json
│
├── servers/
│   ├── web-bff/
│   │   ├── src/
│   │   │   ├── handlers/
│   │   │   ├── presenters/
│   │   │   ├── middleware/
│   │   │   ├── routes/
│   │   │   ├── extractors/
│   │   │   ├── auth/
│   │   │   └── main.rs
│   │   ├── openapi.yaml
│   │   └── Cargo.toml
│   ├── admin-bff/
│   │   ├── src/
│   │   │   ├── handlers/
│   │   │   ├── presenters/
│   │   │   ├── middleware/
│   │   │   ├── routes/
│   │   │   └── main.rs
│   │   ├── openapi.yaml
│   │   └── Cargo.toml
│   ├── edge-gateway/
│   │   ├── src/
│   │   │   ├── authn/
│   │   │   ├── authz/
│   │   │   ├── rate_limit/
│   │   │   ├── routing/
│   │   │   ├── observability/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── internal-rpc/
│       ├── src/
│       └── Cargo.toml
│
├── services/
│   ├── user/
│   │   ├── src/
│   │   │   ├── domain/
│   │   │   ├── application/
│   │   │   ├── policies/
│   │   │   ├── ports/
│   │   │   ├── events/
│   │   │   ├── contracts/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   ├── migrations/
│   │   └── Cargo.toml
│   ├── tenant/
│   │   ├── src/{domain,application,policies,ports,events,contracts}
│   │   ├── tests/
│   │   ├── migrations/
│   │   └── Cargo.toml
│   ├── settings/
│   │   ├── src/{domain,application,policies,ports,events,contracts}
│   │   ├── tests/
│   │   ├── migrations/
│   │   └── Cargo.toml
│   ├── counter/
│   │   ├── src/{domain,application,policies,ports,events,contracts}
│   │   ├── tests/{unit,integration,contract,sync}
│   │   ├── migrations/
│   │   ├── README.md
│   │   └── Cargo.toml
│   ├── admin/
│   │   ├── src/{domain,application,policies,ports,events,contracts}
│   │   ├── tests/
│   │   ├── migrations/
│   │   └── Cargo.toml
│   └── indexing/
│       ├── src/{domain,application,policies,ports,events,contracts}
│       ├── tests/
│       └── Cargo.toml
│
├── workers/
│   ├── indexer/
│   │   ├── src/
│   │   │   ├── sources/
│   │   │   ├── transforms/
│   │   │   ├── sinks/
│   │   │   ├── checkpoint/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── outbox-relay/
│   │   ├── src/
│   │   │   ├── polling/
│   │   │   ├── publish/
│   │   │   ├── dedupe/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── projector/
│   │   ├── src/
│   │   │   ├── consumers/
│   │   │   ├── readmodels/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── scheduler/
│   │   ├── src/
│   │   │   ├── jobs/
│   │   │   ├── dispatch/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── sync-reconciler/
│   │   ├── src/
│   │   │   ├── plans/
│   │   │   ├── executors/
│   │   │   ├── conflict/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── workflow-runner/
│       ├── src/
│       └── Cargo.toml
│
├── packages/
│   ├── kernel/
│   │   ├── src/
│   │   │   ├── ids/
│   │   │   ├── error/
│   │   │   ├── money/
│   │   │   ├── pagination/
│   │   │   ├── tenancy/
│   │   │   ├── time/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── platform/
│   │   ├── src/{config,health,buildinfo,env,release,service_meta,lib.rs}
│   │   └── Cargo.toml
│   ├── contracts/
│   │   ├── http/
│   │   ├── events/
│   │   ├── rpc/
│   │   ├── jsonschema/
│   │   ├── error-codes/
│   │   ├── compat/
│   │   └── sdk-gen/
│   ├── sdk/
│   │   ├── typescript/
│   │   ├── rust/
│   │   └── openapi-clients/
│   ├── runtime/
│   │   ├── ports/
│   │   │   ├── invocation.rs
│   │   │   ├── pubsub.rs
│   │   │   ├── state.rs
│   │   │   ├── workflow.rs
│   │   │   ├── lock.rs
│   │   │   ├── binding.rs
│   │   │   ├── secret.rs
│   │   │   └── queue.rs
│   │   ├── policy/
│   │   │   ├── timeout/
│   │   │   ├── retry/
│   │   │   ├── idempotency/
│   │   │   ├── backpressure/
│   │   │   └── circuit_breaker/
│   │   ├── adapters/
│   │   │   ├── direct/
│   │   │   ├── dapr/
│   │   │   └── memory/
│   │   └── Cargo.toml
│   ├── authn/
│   │   ├── src/{oidc,pkce,session,token,lib.rs}
│   │   └── Cargo.toml
│   ├── authz/
│   │   ├── src/{model,ports,caching,decision,lib.rs}
│   │   ├── adapters/openfga/
│   │   └── Cargo.toml
│   ├── data/
│   │   ├── turso/
│   │   ├── sqlite/
│   │   ├── migration/
│   │   ├── outbox/
│   │   ├── inbox/
│   │   └── common-sql/
│   ├── messaging/
│   │   ├── nats/
│   │   ├── envelope/
│   │   ├── codec/
│   │   └── lib.rs
│   ├── cache/
│   │   ├── src/{api,policies,lib.rs}
│   │   ├── adapters/{moka,valkey,dragonfly}
│   │   └── Cargo.toml
│   ├── storage/
│   │   ├── src/{api,paths,policies,lib.rs}
│   │   ├── adapters/{s3,minio,localfs}
│   │   └── Cargo.toml
│   ├── observability/
│   │   ├── src/{tracing,metrics,logging,baggage,otel,lib.rs}
│   │   └── Cargo.toml
│   ├── security/
│   │   ├── src/{crypto,signing,redaction,pii,lib.rs}
│   │   └── Cargo.toml
│   ├── web3/
│   │   ├── README.md
│   │   ├── traits/
│   │   ├── nostr/
│   │   ├── farcaster/
│   │   ├── at-protocol/
│   │   ├── evm/{base,contracts}
│   │   ├── ton/
│   │   ├── solana/
│   │   └── indexer/
│   ├── wasm/
│   │   ├── wit/
│   │   ├── host/
│   │   ├── guest-sdk/
│   │   ├── components/
│   │   │   ├── sync-strategy/
│   │   │   ├── protocol-transform/
│   │   │   └── policy-hook/
│   │   └── Cargo.toml
│   ├── ui/
│   │   ├── src/{components,layouts,themes}
│   │   └── package.json
│   └── devx/
│       ├── testkit/
│       ├── fixture-loader/
│       ├── contract-test/
│       ├── perf-harness/
│       └── snapshot/
│
├── infra/
│   ├── local/
│   │   ├── compose/
│   │   │   ├── core.yaml
│   │   │   ├── observability.yaml
│   │   │   ├── identity.yaml
│   │   │   └── web3.yaml
│   │   └── seeds/
│   │       ├── demo-tenants/
│   │       └── demo-users/
│   ├── kubernetes/
│   │   ├── bootstrap/
│   │   │   ├── k3s-install.sh
│   │   │   ├── cilium-install.sh
│   │   │   ├── flux-bootstrap.sh
│   │   │   └── cluster-verify.sh
│   │   ├── base/
│   │   │   ├── namespaces/
│   │   │   ├── storageclasses/
│   │   │   ├── rbac/
│   │   │   ├── network-policies/
│   │   │   ├── pod-security/
│   │   │   ├── priority-classes/
│   │   │   └── limit-ranges/
│   │   ├── addons/
│   │   │   ├── cilium/
│   │   │   ├── gateway-api/
│   │   │   ├── cert-manager/
│   │   │   ├── external-dns/
│   │   │   ├── nats/
│   │   │   ├── cache/
│   │   │   ├── object-storage/
│   │   │   ├── openobserve/
│   │   │   ├── zitadel/
│   │   │   ├── openfga/
│   │   │   ├── dapr/
│   │   │   ├── linkerd/
│   │   │   ├── spinkube/
│   │   │   └── metrics/
│   │   ├── rendered/
│   │   │   ├── dev/
│   │   │   ├── staging/
│   │   │   └── prod/
│   │   └── overlays/
│   │       ├── dev/
│   │       ├── staging/
│   │       └── prod/
│   ├── gitops/
│   │   └── flux/
│   │       ├── sources/
│   │       ├── infrastructure/
│   │       ├── applications/
│   │       ├── tenants/
│   │       └── policies/
│   ├── security/
│   │   ├── sops/
│   │   │   ├── .sops.yaml
│   │   │   ├── age/
│   │   │   └── secrets.enc.yaml
│   │   ├── supply-chain/
│   │   │   ├── cosign/
│   │   │   ├── sbom/
│   │   │   └── provenance/
│   │   └── cluster-policies/
│   │       ├── admission/
│   │       ├── image/
│   │       └── runtime/
│   └── images/
│       ├── Dockerfile.rust-service
│       ├── Dockerfile.web
│       ├── Dockerfile.worker
│       └── bake.hcl
│
├── ops/
│   ├── migrations/
│   │   ├── runner/
│   │   ├── plans/
│   │   └── rollback/
│   ├── benchmark/
│   │   ├── http/
│   │   ├── nats/
│   │   ├── sync/
│   │   └── storage/
│   ├── resilience/
│   │   ├── load/
│   │   ├── fault/
│   │   ├── chaos/
│   │   └── recovery/
│   ├── backup-restore/
│   │   ├── turso/
│   │   ├── object-store/
│   │   └── verification/
│   ├── runbooks/
│   │   ├── incident-response.md
│   │   ├── auth-outage.md
│   │   ├── nats-backpressure.md
│   │   ├── sync-conflict.md
│   │   ├── projection-rebuild.md
│   │   └── deploy-rollback.md
│   └── scripts/
│       ├── bootstrap-vps.sh
│       ├── verify-platform.sh
│       ├── smoke.sh
│       └── sync-health-check.sh
│
├── verification/
│   ├── e2e/
│   │   ├── demo-counter/
│   │   ├── multi-tenant/
│   │   ├── settings/
│   │   └── desktop-web-roundtrip/
│   ├── contract/
│   │   ├── backward-compat/
│   │   ├── sdk-roundtrip/
│   │   └── event-schema/
│   ├── topology/
│   │   ├── single-vps/
│   │   ├── split-workers/
│   │   └── k3s/
│   ├── resilience/
│   │   ├── retry/
│   │   ├── idempotency/
│   │   ├── outbox/
│   │   └── failover/
│   ├── performance/
│   │   ├── bff/
│   │   ├── gateway/
│   │   ├── nats/
│   │   └── cache/
│   └── golden/
│       ├── generated-sdk/
│       ├── rendered-manifests/
│       └── diagrams/
│
├── docs/
│   ├── adr/
│   │   ├── 001-platform-model-first.md
│   │   ├── 002-services-are-libraries-not-processes.md
│   │   ├── 003-runtime-abstraction-direct-plus-dapr.md
│   │   ├── 004-k3s-cilium-gateway-api-flux.md
│   │   ├── 005-authn-authz-zitadel-openfga.md
│   │   ├── 006-observability-vector-openobserve.md
│   │   ├── 007-workers-first-async-architecture.md
│   │   └── 008-wasm-extension-plane.md
│   ├── architecture/
│   │   ├── context/
│   │   ├── container/
│   │   ├── component/
│   │   ├── sequence/
│   │   ├── deployment/
│   │   └── topology/
│   ├── platform-model/
│   │   ├── service-model.md
│   │   ├── deployable-model.md
│   │   ├── resource-model.md
│   │   ├── workflow-model.md
│   │   └── topology-model.md
│   ├── contracts/
│   │   ├── http/
│   │   ├── events/
│   │   ├── rpc/
│   │   └── error-codes/
│   ├── operations/
│   │   ├── local-dev.md
│   │   ├── single-vps.md
│   │   ├── k3s-cluster.md
│   │   ├── gitops.md
│   │   └── secret-management.md
│   └── generated/
│       ├── service-catalog/
│       ├── resource-catalog/
│       └── dependency-graphs/
│
├── fixtures/
│   ├── tenants/
│   ├── users/
│   ├── settings/
│   ├── counter/
│   ├── sync-scenarios/
│   ├── authz-tuples/
│   └── seeds/
│
├── tools/
│   ├── web3/
│   │   ├── anvil.sh
│   │   ├── nostr-relay.yml
│   │   ├── ton-local.sh
│   │   └── atproto-local.sh
│   ├── codegen/
│   ├── loadtest/
│   └── diagnostics/
│
├── README.md
├── GOAL.md
├── AGENTS.md
├── CONTRIBUTING.md
└── CHANGELOG.md
```

---

# 2) 全局硬性约束规则

这部分是全仓库最高优先级规则，优先级高于单目录说明。

## 2.1 架构总规则

### 规则 A：`services/*` 是业务能力库，不是进程

- **必须**：每个 `services/*` 可被 `servers/*` 与 `workers/*` 同时复用。
- **禁止**：在 `services/*` 中放 HTTP server、CLI main、消息消费者主循环、容器探针逻辑。
- **验证**：删掉所有 `servers/*` 和 `workers/*` 后，`cargo test -p <service>` 仍可独立通过。

### 规则 B：`platform/model/*` 是平台真理源

- **必须**：服务、部署单元、资源、工作流、拓扑、环境都先在 `platform/model/*` 定义。
- **禁止**：先手改 `infra/kubernetes/rendered/*` 再反推模型。
- **验证**：删除 `infra/kubernetes/rendered/*` 后，执行生成命令能完全恢复。

### 规则 C：`packages/contracts/*` 是协议真理源

- **必须**：HTTP / Event / RPC / DTO / ErrorCode 的变化先改 `packages/contracts/*`。
- **禁止**：直接在 `servers/*` 手写响应结构后再回填契约。
- **验证**：`gen-contracts` 后工作区零 diff。

### 规则 D：所有异步执行单元必须放进 `workers/*`

- **必须**：后台 job、投递器、投影器、同步协调器都是 worker。
- **禁止**：把长轮询、批处理、事件投递塞进 BFF。
- **验证**：BFF 在无消息系统情况下仍能做最小功能响应。

### 规则 E：具体中间件只能出现在 `packages/*/adapters/*`

- **必须**：NATS、Turso、Valkey、Dragonfly、OpenFGA、Dapr、S3 等接入都放在 adapter。
- **禁止**：业务层直接 import 第三方 client SDK。
- **验证**：全仓库 grep vendor client，只允许在 adapters 目录命中。

### 规则 F：生成产物必须可再生

- **必须**：`sdk/`、`rendered/`、`docs/generated/`、`platform/catalog/` 都由命令生成。
- **禁止**：手工长期维护 generated 文件。
- **验证**：CI 中先删生成目录再跑生成流程，最终零 diff。

---

## 2.2 依赖方向总规则

```text
apps/*          -> packages/sdk, packages/ui, packages/authn
servers/*       -> services/*, packages/*
workers/*       -> services/*, packages/*
services/*      -> packages/kernel, packages/platform, packages/contracts, packages/runtime(ports only), packages/authn, packages/authz
packages/*      -> 低层可互依，但不得反向依赖 servers/apps/workers/services
platform/*      -> 不依赖业务实现，只引用模型 schema / 生成器 / 校验器
infra/*         -> 不被业务代码 import
ops/*           -> 不被业务代码 import
verification/*  -> 可依赖全仓库，但只能做测试与验证
docs/*          -> 不作为运行时依赖
```

### 补充限制

- `apps/*` **禁止** 直接依赖 `services/*`。
- `services/*` **禁止** 互相直接依赖；跨域协作通过 contracts / events / runtime ports 完成。
- `servers/*` **允许** 同时依赖多个 `services/*` 做聚合。
- `workers/*` **允许** 同时依赖多个 `services/*` 执行业务工作流。
- `packages/runtime/adapters/dapr` **禁止** 被业务层直接依赖，只能通过 feature 或 wiring 注入。

---

# 3) 各顶级目录硬规则

下面统一使用四段式：

- **职责**
- **必须**
- **禁止**
- **验证**

---

## 3.1 根层

### 职责

统一工具链、工作区、CI、构建策略、代码质量与发布入口。

### 必须

- 所有开发命令从 `justfile` 暴露。
- 所有版本从 `.mise.toml` 与 `rust-toolchain.toml` 锁定。
- 所有工作区成员在 `Cargo.toml` / `bun-workspace.yaml` 中可追踪。
- CI 只调用根层命令，不直调深层脚本。

### 禁止

- 根目录出现临时脚本、业务配置、环境差异逻辑。
- 在子目录私自维护第二套工具版本定义。
- 让 Agent 绕过 `just` 直接发散执行。

### 验证

- `just --list` 列出完整命令。
- `mise doctor` 通过。
- `cargo metadata` 与 `moon project-graph` 成员一致。

---

## 3.2 `agent/`

### 职责

给 Agent 提供边界、禁令、模板、流程与审查清单。

### 必须

- 新增模块前先补 `codemap.yml` 或模板。
- 所有危险操作都要有 checklist。
- 依赖白名单与禁止模式必须机器可读。

### 禁止

- 放业务实现代码。
- 把口头约定留在聊天记录而不落地到约束文件。
- 在模板里暗藏运行时分支逻辑。

### 验证

- 新 Agent 在只读 `agent/` 与 `docs/` 的前提下可完成加模块任务。
- 违反依赖方向的 PR 能被约束规则拒绝。

---

## 3.3 `platform/`

### 职责

描述平台，不实现业务。
决定“服务是什么、怎么部署、依赖什么资源、用什么策略、在什么拓扑运行”。

### 必须

- `model/` 下每个实体都能被 schema 校验。
- `topologies/` 与 `deployables/` 是正交关系。
- `generators/` 只能从模型生成，不倒写模型。
- `catalog/` 作为可审查生成结果保留在 Git。

### 禁止

- 放业务逻辑。
- 直接写第三方基础设施的 SDK 调用。
- 在 `rendered` 产物中手工修补。

### 验证

- `platform validate` 全通过。
- 删除 `catalog/`、`infra/kubernetes/rendered/` 后可重新生成。
- 任意服务是否暴露、是否有 HPA、是否依赖 NATS，都能从模型中回答。

---

## 3.4 `apps/`

### 职责

前端与客户端壳层，只消费 SDK、UI、认证与同步协调。

### 必须

- API 调用只走 `packages/sdk/*` 生成产物。
- 客户端同步只通过 `lib/sync/` 聚合，不散落页面。
- 端特定能力封装在各自 `lib/` 中。

### 禁止

- 直接 import `services/*`。
- 手写与后端重复的 DTO。
- 在页面路由中嵌入复杂业务规则。

### 验证

- 更新后端契约后，前端只需要更新 SDK 即可编译。
- 删除 `servers/*` 源码实现，前端仍能通过 SDK 类型检查。

---

## 3.5 `servers/`

### 职责

同步请求入口层，负责协议适配、聚合、鉴权、限流、视图组装。

### 必须

- 每个 server 只有 `main.rs` 入口。
- HTTP 服务必须带 `openapi.yaml`。
- 所有 handler 只调用 services 的 application API 或 composition 层。
- 所有请求路径都经过 middleware 的 telemetry / auth / tenant 注入。

### 禁止

- 写领域规则。
- 直接操作数据库。
- 自己生成跨服务事件。
- 长时间阻塞任务驻留在 server 中。

### 验证

- `cargo build -p <server>` 独立通过。
- `openapi.yaml` 与 handler 行为可被 contract test 覆盖。
- 业务规则单测不落在 `servers/*`。

---

## 3.6 `services/`

### 职责

领域逻辑与用例逻辑的唯一落点。

### 必须

- 目录固定为：`domain/`、`application/`、`policies/`、`ports/`、`events/`、`contracts/`。
- 所有外部依赖通过 `ports/` 抽象。
- 跨域协作优先走事件或共享契约。
- 每个 service 自带独立 `tests/` 与 `migrations/`。

### 禁止

- 直接依赖其他 `services/*`。
- 直接依赖具体适配器。
- 暴露框架绑定类型到业务核心。
- 混入部署、容器、探针、CLI、HTTP 层概念。

### 验证

- `cargo test -p <service>` 不需要启动 HTTP 服务。
- `rg "axum|tauri|hyper|reqwest"` 在 `services/*/src/domain` 中应为零。
- `ports/` 可被 memory adapter 替换做纯业务测试。

---

## 3.7 `workers/`

### 职责

执行异步任务、流处理、投影、补偿、同步协调。

### 必须

- 每个 worker 只有一个主职责。
- 输入输出边界明确：消费什么、产出什么、幂等键是什么。
- 所有 worker 都要有 checkpoint / dedupe / retry 策略。
- 与消息系统或任务队列相关的协议统一走 `packages/runtime` 或 `packages/messaging`。

### 禁止

- 复用 BFF handler 充当 worker。
- 一个 worker 承担“所有后台任务”。
- 在 worker 中塞入端展示逻辑。

### 验证

- 单独启动某个 worker 不影响其他 worker 的编译。
- 人工重复投递同一消息不会产生业务重复副作用。
- 关闭 worker 再重启后可从 checkpoint 继续。

---

## 3.8 `packages/`

### 职责

提供共享能力、核心抽象、适配器、生成结果与开发工具。

### 必须

- `kernel/` 保持最底层、最稳定。
- `contracts/` 作为协议真理源。
- `runtime/ports` 先于 `runtime/adapters`。
- 所有适配器按能力分组，不按“随便一个 vendor 名字”堆放。
- `sdk/` 只放生成结果。

### 禁止

- 在共享包中写具体业务规则。
- 让高层包反依赖低层业务模块。
- 手动修改 `sdk/` 生成代码。

### 验证

- `cargo build -p packages-kernel` 这类底层包可以独立通过。
- 替换某个 adapter 后，上层业务无需改签名。
- `gen-sdk` 后零 diff。

---

## 3.9 `infra/`

### 职责

声明并交付基础设施。
本地环境、集群基座、GitOps、安全与镜像构建都在此处。

### 必须

- `rendered/` 目录只放生成产物。
- `bootstrap/` 只放集群初始化，不放应用部署脚本。
- `gitops/flux` 作为生产交付面。
- `security/` 统一管理 secrets、签名、供应链产物。

### 禁止

- 在 `infra/` 放业务代码。
- 手工长时间维护应用 workload 清单。
- 用 shell 脚本直接替代 GitOps。
- 把明文 secrets 提交进仓库。

### 验证

- 新环境部署只需：bootstrap cluster -> render manifests -> flux reconcile。
- 删除 rendered 后能完全重建。
- `sops` 与 `age` 路径清晰，CI 能拒绝明文 secrets。

---

## 3.10 `ops/`

### 职责

承载必须存在的运维动作：迁移、压测、演练、备份恢复、巡检。

### 必须

- 所有操作仍由 `just` 统一入口触发。
- 迁移、回滚、演练都需要 runbook。
- 压测与故障演练目录分开。

### 禁止

- 作为业务运行时依赖。
- 无文档的临时脚本永久留存。
- 生产只靠人工 SSH 手工执行。

### 验证

- 任何 runbook 都能定位到对应命令。
- 备份恢复至少能在 staging 演练。
- 故障演练结果可追踪到报告。

---

## 3.11 `verification/`

### 职责

做跨模块、跨拓扑、跨生成层的统一验证。

### 必须

- E2E、契约兼容、拓扑验证、韧性验证、性能基线分目录。
- `golden/` 保存受控基准产物。
- counter、多租户、settings 必须在这里有完整链路用例。

### 禁止

- 用 verification 目录承载生产运行代码。
- 把仅单模块单测混入这里。
- 忽略 generated 产物的回归检查。

### 验证

- 切换 `single-vps` 与 `k3s-microservices` 拓扑后，同一套 E2E 用例仍能跑。
- 契约向后兼容检查可挡住破坏性修改。
- 性能回归有固定阈值。

---

## 3.12 `docs/`

### 职责

沉淀 ADR、平台模型说明、运维文档、生成后的架构资产。

### 必须

- 所有重大架构变化对应 ADR。
- `docs/generated/` 由生成器产出。
- 新成员只读 docs 即可理解系统骨架与拓扑。

### 禁止

- 让架构知识只存在代码注释或口头聊天。
- 让过时文档与现状长期背离。
- 在 docs 中维护第二套真理源。

### 验证

- 任一顶级目录都有对应说明文档。
- 设计变更 PR 至少包含代码或模型变化 + ADR/文档变化之一。
- `generated/` 可由命令重建。

---

## 3.13 `fixtures/`

### 职责

集中管理测试数据、演示种子、同步场景与授权 tuples。

### 必须

- 全部可版本控制。
- 业务演示数据与测试断言一致。
- 多租户、settings、counter、authz 都要有独立 fixtures。

### 禁止

- 混入真实生产数据。
- 在测试代码里临时硬编码大型结构体当 fixture。
- fixture 与 schema 漂移却无人维护。

### 验证

- 新环境可通过 seed 命令得到完整 demo 数据。
- 删除 DB 后从 fixtures 能恢复 E2E 场景。
- authz tuples 可独立加载重放。

---

## 3.14 `tools/`

### 职责

本地协议测试、代码生成辅助、诊断与负载工具。

### 必须

- 所有工具都服务于开发、测试、诊断，不进入生产运行时依赖。
- 每个工具要有用途说明。
- 工具命令仍走 `just` 暴露。

### 禁止

- 在 tools 中放正式业务二进制。
- 用 tools 替代平台生成器或正式 infra。

### 验证

- 删除 `tools/` 不影响生产构建。
- 本地 Web3 / loadtest / diagnostics 彼此隔离。

---

# 4) 关键子目录的硬性规则

---

## 4.1 `services/<name>/domain`

- **职责**：实体、不变量、值对象、领域规则。
- **必须**：纯业务，无 IO。
- **禁止**：网络、数据库、时钟获取、日志框架依赖。
- **验证**：domain 单测不需要 mock HTTP / DB。

## 4.2 `services/<name>/application`

- **职责**：用例编排、命令查询、事务边界。
- **必须**：只依赖 domain + ports。
- **禁止**：直接依赖 adapters。
- **验证**：可用 in-memory ports 跑完整用例测试。

## 4.3 `services/<name>/ports`

- **职责**：定义 repo、cache、publisher、clock、id generator、authz decision 等抽象。
- **必须**：接口语义稳定。
- **禁止**：泄露 vendor 专有类型。
- **验证**：至少存在 memory/mock 实现用于测试。

## 4.4 `servers/*/handlers`

- **职责**：请求解析、调用 use case、响应组装。
- **必须**：薄。
- **禁止**：复杂 if/else 业务分支。
- **验证**：handler 单测主要是协议层，不断言深业务细节。

## 4.5 `workers/*/checkpoint`

- **职责**：记录消费进度、重放边界、恢复点。
- **必须**：明确 checkpoint schema 与更新条件。
- **禁止**：隐式依赖内存状态。
- **验证**：重启恢复测试必须覆盖。

## 4.6 `packages/runtime/ports`

- **职责**：抽象分布式系统能力。
- **必须**：业务视角命名，不用 vendor 视角命名。
- **禁止**：把 Dapr、NATS、Redis 术语直接写进通用接口。
- **验证**：同一个业务逻辑能在 `memory/direct/dapr` 三种实现下跑通。

## 4.7 `packages/contracts/http` / `events`

- **职责**：系统外部契约。
- **必须**：版本、兼容性、错误码策略清晰。
- **禁止**：让 server 自己成为真理源。
- **验证**：backward compat 测试挡住破坏性修改。

## 4.8 `infra/kubernetes/rendered`

- **职责**：由模型生成的工作负载与环境清单。
- **必须**：只读。
- **禁止**：手改。
- **验证**：CI 检测手改漂移。

---

# 5) 命名规则

## 顶级目录命名

- 只允许语义稳定的英文小写短名。
- 顶级目录不得频繁改名。

## deployable 命名

- `*-bff`：同步聚合入口
- `*-gateway`：边缘/入口转发单元
- `*-worker`：异步执行单元
- `*-relay`：投递桥接单元
- `*-projector`：读模型构建单元
- `*-reconciler`：同步/修复/补偿单元
- `*-runner`：工作流/任务执行单元

## service 命名

- 用业务域名，不用技术实现名。
- 禁止 `common`、`utils`、`misc` 这类垃圾桶命名。

## package 命名

- 先按能力命名，再按 vendor 放进 `adapters/`。
- 例：`cache/adapters/dragonfly`，而不是 `packages/dragonfly-cache-utils`。

---

# 6) 每类模块的最小文件要求

## 新增一个 service，最少必须有

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

## 新增一个 server，最少必须有

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

## 新增一个 worker，最少必须有

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

## 新增一个 resource model，最少必须有

```text
platform/model/resources/<name>.yaml
docs/platform-model/<name>-resource.md   # 可选，但建议同步
```

## 新增一个 topology，最少必须有

```text
platform/model/topologies/<name>.yaml
verification/topology/<name>/
docs/operations/<name>.md
```

---

# 7) 推荐放进 `agent/constraints/dependencies.yaml` 的硬规则摘要

下面这段最适合给本地 Agent 直接消费：

```yaml
rules:
  - name: apps-cannot-import-services
    from: "apps/**"
    disallow:
      - "services/**"

  - name: services-cannot-import-other-services
    from: "services/**"
    disallow:
      - "services/**"
    except_same_module: true

  - name: services-cannot-import-concrete-adapters
    from: "services/**"
    disallow:
      - "packages/**/adapters/**"
      - "infra/**"
      - "ops/**"

  - name: servers-can-import-services-and-packages
    from: "servers/**"
    allow:
      - "services/**"
      - "packages/**"

  - name: workers-can-import-services-and-packages
    from: "workers/**"
    allow:
      - "services/**"
      - "packages/**"

  - name: platform-cannot-contain-business-code
    from: "platform/**"
    disallow_runtime_code: true

  - name: rendered-is-readonly
    from: "infra/kubernetes/rendered/**"
    readonly: true

  - name: sdk-is-generated
    from: "packages/sdk/**"
    readonly: true

  - name: docs-generated-is-readonly
    from: "docs/generated/**"
    readonly: true
```

---

# 8) 推荐放进 `README` / `AGENTS` 的一句话规则

这几句非常关键，适合写在仓库入口最前面：

1. **先改 platform model，再改 infra。**
2. **先改 contracts，再改 server handler。**
3. **services 是库，不是进程。**
4. **workers 是一等公民，不是附属脚本。**
5. **vendor 只能进 adapters。**
6. **generated 目录禁止手改。**
7. **拓扑变化靠 topology model，不靠重构业务。**

---

# 9) 第一批必须落地的校验命令

建议你把这批命令固化进 `justfile`：

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

其中最关键的是这 5 个：

- `just gen-platform`
- `just validate-platform`
- `just validate-deps`
- `just verify-generated`
- `just verify-single-vps`

因为这 5 个命令共同保证：
**模型是完整的，依赖方向没坏，生成物没漂移，单 VPS 路径仍然成立。**

---

这份内容下一步最适合直接落成两个文件：

- `docs/architecture/repo-layout.md`
- `agent/codemap.yml`

这样你的本地 Agent 接下来做任何改造，都有统一边界可依。
