# AGENTS.md

> 目标：为本仓库的 agent 协作提供最小但足够的总控协议。
> 回答和汇报 **MUST** 使用中文
> 当前默认上下文只关注后端主链，不把 `apps/**` 作为默认学习入口。

## 1. 默认阅读顺序

进入后端任务时，默认按以下顺序建立上下文：

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. `docs/architecture/repo-layout.md`
6. `docs/operations/counter-service-reference-chain.md`

如果任务是文档规划、审计、瘦身或 gate/CI 收敛，再继续读取：

1. `docs/README.md`
2. `docs/backend-infrastructure-audit-checklist.md`
3. `docs/counter-service-reference-chain-checklist.md`
4. `docs/document-pruning-ab-checklist.md`
5. `docs/gate-ci-decoupling-checklist.md`

## 2. 真理源优先级

判断现状时，按以下优先级取证：

1. 代码、schema、validator、gate、脚本
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. `docs/architecture/repo-layout.md`
6. `docs/operations/counter-service-reference-chain.md`
7. `docs/adr/**` 与 `.agents/skills/*/SKILL.md`

硬规则：

1. 文档与代码冲突时，以代码和可执行验证为准。
2. 不得仅凭目标态文档推断某个文件或模块一定存在。
3. 对当前状态的结论必须能回指到真实文件、目录或命令输出。

## 3. 总控职责

planner 负责：

1. 理解用户目标。
2. 审计受影响目录。
3. 根据 `routing-rules.yml` 决定是否派发 subagent。
4. 按依赖顺序推进修改与验证。
5. 收敛改动、风险、验证结果。

planner 不负责：

1. 凭空设计不存在的模块。
2. 绕过边界直接把多域逻辑揉成一个补丁。
3. 用散文替代 gate、schema、validator、script。

## 4. 路由与边界

路径到 subagent 的默认映射以 `agent/manifests/routing-rules.yml` 为准。高频摘要如下：

1. `platform/model/**`、`platform/schema/**`、`infra/**`、`ops/**` -> `platform-ops-agent`
2. `packages/contracts/**`、`docs/contracts/**` -> `contract-agent`
3. `services/**` -> `service-agent`
4. `servers/**` -> `server-agent`
5. `workers/**` -> `worker-agent`
6. `apps/**`、`packages/ui/**` -> `app-shell-agent`
7. `docs/architecture/**`、`AGENTS.md`、`agent/**`、根级配置 -> `planner`

多域同时变更时，默认派发顺序以 `routing-rules.yml` 为准：

1. `platform-ops-agent`
2. `contract-agent`
3. `service-agent`
4. `server-agent` / `worker-agent`
5. `app-shell-agent`
6. 最终总验证

不要为了形式完整而机械 fan-out 给所有 agent。只有在目录边界、职责边界或验证边界真的不同的时候才拆分。

## 5. 全局硬约束

1. 中文沟通；代码、命令、配置键、日志、协议字段保持原文。
2. 先读再改，先证据后判断，先搜索后猜测。
3. 优先最小闭环，不做无关重构，不顺手修 unrelated 问题。
4. 未执行的验证不能声称通过。
5. 不确定就明确标注，不把猜测包装成结论。
6. 不允许通过删除、跳过、吞错、伪造成功状态来“解决”问题。
7. 生成物目录只读，必须改源头再重新生成。

## 6. 后端默认开发姿势

当前仓库的默认开发轨道是后端优先，核心判断如下：

1. `apps/**` 不是默认上下文，除非任务明确涉及前端壳层。
2. `counter-service` 是默认后端参考锚点，不是最小 demo。
3. 新的后端能力默认应先对照 `counter-service` 参考链，再决定是否扩展新模式。
4. 高价值生产链路不能因为业务简单而从默认路径消失。

`counter-service` 当前承载两条并行链路：

1. 业务主链：`service -> contracts -> server -> outbox -> relay -> projector`
2. 工程横切链：`platform model -> secrets -> deploy -> GitOps -> promotion -> drift -> runbook`

如果某个生产能力尚未挂接到这条 reference chain，就还不能视为仓库默认工程惯性。

## 7. 文档策略

默认只保留两类文档进入主上下文：

1. A 类：仓库级目标态、规则、边界、真理源、gate/CI 收敛规则。
2. B 类：局部 owner 文档、reference chain、局部运行与验证说明。

执行规则：

1. 不稳定、重复、未来态过强的架构散文不应占据默认入口。
2. 删除或归档文档前，先确认其中的生产工具链信息已经进入 A 类文档、B 类 owner 文档，或 `counter-service` reference chain。
3. 文档收敛的目标是减少漂移，不是隐藏复杂度。

## 8. 目录与读写约束

重点目录职责以 `agent/codemap.yml` 为准。默认只记以下稳定边界：

1. `platform/**`：平台模型、schema、validator、generator、catalog
2. `services/**`：业务能力与状态边界，service-local semantics 在 `services/<name>/model.yaml`
3. `servers/**`：同步入口与协议适配
4. `workers/**`：异步执行、projection、replay、恢复
5. `packages/contracts/**`：共享协议真理源
6. `infra/**`：基础设施声明、交付、GitOps、secrets
7. `verification/**`：跨层验证
8. `docs/**`：A/B 文档，不是业务逻辑实现层

禁止手工编辑的生成物目录：

1. `packages/sdk/**`
2. `infra/kubernetes/rendered/**`
3. `docs/generated/**`
4. `platform/catalog/**`

默认不要读取或搜索以下目录：

1. `node_modules/**`
2. `target/**`
3. `.moon/cache/**`
4. `.cocoindex_code/**`
5. `.jj/**`

## 9. 工具使用原则

1. 优先直接读取文件系统，使用 `glob`、`read`、`grep` 获取证据。
2. 不依赖已废弃索引或缓存目录判断文件是否存在。
3. 搜索、diff、验证优先使用工具和脚本，不手写结论。
4. 多个独立搜索可以并行，但不要为了省一句话牺牲可读性。

## 10. 验证与风险升级

subagent 对应 gate 以 `agent/manifests/gate-matrix.yml` 为准，最终总验证默认落在 `just verify`。

以下情况必须显式升级风险：

1. 需求与当前架构或 ADR 明显冲突。
2. 改动横跨多个核心目录或公共契约。
3. 需要新增关键依赖、改变默认交付路径或修改关键链路。
4. 测试、gate 或脚本不足以验证本次修改。
5. 任务涉及 4 个以上 subagent 且存在复杂依赖。
6. `counter-service` reference chain 暴露出新的链路缺口，但本次又无法补齐。

## 11. 辅助脚本

以下脚本用于路由、作用域门禁和交接验证：

1. `bun run scripts/route-task.ts`
2. `bun run scripts/run-scoped-gates.ts <subagent>`
3. `bun run scripts/verify-handoff.ts <subagent>`

这些脚本是总控协议的执行补充，但它们不替代阅读真实代码与 diff。
