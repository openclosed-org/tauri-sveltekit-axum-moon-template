# Repo Layout

> 目标：用一份足够稳定的目录说明，帮助 agent 和开发者快速进入本仓库的后端开发轨道。
>
> 本文档不是完整愿景稿，也不是逐目录操作手册；它只保留当前稳定、可执行、可验证的结构信息。

## 1. 如何使用这份文档

进入后端任务时，建议按以下顺序阅读：

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. 本文档
6. `docs/operations/counter-service-reference-chain.md`

如果本文档与代码现状冲突，以代码、schema、validator、gate、脚本为准。

## 2. 当前默认视角

当前仓库默认采用后端视角理解结构：

1. 默认不把 `apps/**` 作为学习入口。
2. 默认后端参考锚点是 `counter-service`。
3. 目录说明的重点是边界、真理源和变更顺序，而不是未来想象中的完整形态。

## 3. 稳定分层原则

这些原则是当前目录结构中最稳定的部分：

1. `services/**` 表示业务能力与状态边界，不承载进程入口。
2. `servers/**` 表示同步请求入口与协议适配。
3. `workers/**` 表示异步执行、投递、projection、replay、恢复。
4. `packages/contracts/**` 是共享协议真理源。
5. `platform/model/**` 表示平台级元数据、deployable、topology、workflow、environment、全局规则。
6. `services/<name>/model.yaml` 表示 service-local semantics，不应被拆散回 `platform/model/state/**`。
7. `infra/**` 表示 secrets、deploy、GitOps、environment overlays 和交付路径。
8. `verification/**` 表示跨层验证，而不是业务实现本身。

## 4. 顶层目录总览

以下目录是当前最值得记住的稳定骨架：

| 路径 | 角色 | 当前默认关注度 |
| --- | --- | --- |
| `agent/` | 路由、边界、门禁的机器可读规则 | 高 |
| `platform/` | schema、model、validator、generator | 高 |
| `services/` | 业务能力与状态边界 | 高 |
| `servers/` | 同步入口层 | 高 |
| `workers/` | 异步执行层 | 高 |
| `packages/contracts/` | 共享协议 | 高 |
| `infra/` | secrets、deploy、GitOps、环境承载 | 高 |
| `verification/` | 跨层验证与回放 | 中 |
| `docs/` | A/B 文档入口与参考链 | 中 |
| `apps/` | 前端与客户端壳层 | 低 |
| `ops/` | 运维执行与 runbook | 中 |
| `fixtures/` | 测试与样例数据 | 中 |

## 5. 真理源分层

当前仓库可以按三层理解：

### 5.1 规则与模型层

优先作为判断依据的路径：

1. `agent/**`
2. `platform/schema/**`
3. `platform/model/**`
4. `services/*/model.yaml`
5. `packages/contracts/**`
6. `docs/adr/**`

### 5.2 实现层

主要承载业务实现与运行时适配：

1. `services/*/src/**`
2. `servers/**`
3. `workers/**`
4. `packages/**` 中非生成物目录
5. `infra/**` 中非 rendered 目录

### 5.3 生成层

以下目录应视为只读生成物：

1. `packages/sdk/**`
2. `platform/catalog/**`
3. `infra/kubernetes/rendered/**`
4. `docs/generated/**`

这些目录出现问题时，应回到 schema、model、generator 或 source contracts 修复，而不是手工补丁。

## 6. 默认变更顺序

当一个后端能力跨多层修改时，默认顺序如下：

1. 先确认是否需要改 `platform/schema/**`
2. 再改 `platform/model/**` 或 `services/<name>/model.yaml`
3. 再改 `packages/contracts/**`
4. 再改 `services/**`
5. 再改 `servers/**` 和 `workers/**`
6. 最后改 `infra/**`、生成物和补充文档

这不是绝对强制的脚本顺序，但它是当前最稳妥的回归控制顺序。

## 7. 后端默认参考路径

理解本仓库后端时，默认从 `counter-service` 开始，而不是从最复杂的 service 倒推抽象。

推荐参考路径：

1. `platform/model/services/counter-service.yaml`
2. `services/counter-service/model.yaml`
3. `packages/contracts/**` 中的 counter 相关契约
4. `services/counter-service/src/**`
5. `servers/bff/web-bff/src/handlers/counter.rs`
6. `workers/outbox-relay/**`
7. `workers/projector/**`
8. `infra/security/**`、`infra/k3s/**`、`infra/gitops/**` 中与 counter 链路相关的部分

这条路径同时覆盖两类问题：

1. 最小业务主链如何实现。
2. 一个后端能力如何进入 secrets、deploy、GitOps、promotion、drift、runbook 轨道。

更细的真实状态、已闭环部分和未闭环部分，以 `docs/operations/counter-service-reference-chain.md` 为准。

## 8. 关键目录说明

### 8.1 `platform/`

当前 `platform/` 的核心角色是表达和验证平台级规则，而不是替代各 service 自己的语义模型。

重点子目录：

1. `platform/schema/`：模型 schema
2. `platform/model/services/`：service 元数据
3. `platform/model/deployables/`：deployable 定义
4. `platform/model/topologies/`：承载组合
5. `platform/model/environments/`：环境差异
6. `platform/model/state/`：全局 state 默认规则与 ownership map

### 8.2 `services/`

每个 service 目录至少应被理解为两部分：

1. `model.yaml`：该 service 的局部语义真理源
2. `src/**`：domain、application、ports、infrastructure 等实现

当前默认参考样例是 `services/counter-service/`。

### 8.3 `servers/`

`servers/` 承担协议适配、认证上下文、错误映射、DTO 组合等同步入口职责，不应反向拥有服务领域逻辑。

### 8.4 `workers/`

`workers/` 承担异步推进职责。当前默认参考链包括：

1. `workers/outbox-relay/`
2. `workers/projector/`

其中结构已经有参考价值，但真实完成度仍应以代码和 reference chain 文档核对，不能直接假定为最终生产模板。

### 8.5 `infra/`

`infra/` 不是附属目录，而是后端默认工程路径的一部分，尤其包括：

1. `infra/security/`：SOPS 与 secrets
2. `infra/gitops/`：Flux / GitOps
3. `infra/k3s/`、`infra/kubernetes/`：overlay、base、rendered manifests
4. `infra/local/`：本地环境承载

如果这些能力没有通过 `counter-service` reference chain 挂回默认路径，就说明默认工程轨道仍未收敛完成。

## 9. 非默认上下文

以下内容当前不应作为默认学习入口：

1. 大量未来态架构散文
2. 未挂接到 `counter-service` 参考链的独立生产工具链说明
3. 与当前代码状态明显脱节的目录说明
4. `apps/**` 的实现细节

这些内容可以按需阅读，但不应替代默认主线。

## 10. 一句话结论

这份 `repo-layout.md` 的作用，是告诉后续 agent 和开发者：

1. 仓库的稳定层次是什么。
2. 后端默认从哪里入手。
3. 变更应按什么顺序推进。
4. `counter-service` 为什么是默认锚点，而不是普通 demo。
