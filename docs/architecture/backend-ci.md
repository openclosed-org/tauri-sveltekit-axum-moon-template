# Backend CI Architecture

> 目标：只监控后端代码，但不把所有后端模块同权耦合到一个慢速工作流里。  
> 原则：**全量纳管、分层阻塞、主线优先、替补独立、实验非阻塞**。

---

## 1. 设计目标

本仓库的定位不是单一业务系统，而是一个可长期演进的后端 boilerplate：

- 以后端 Rust/Axum 架构为主
- 长期验证轻量、高性能、云原生友好的生态组合
- 当前业务逻辑较轻，重点是模块边界、依赖关系、适配器替换能力和拓扑演进能力

因此，CI 的目标不是“每次把所有后端 crate 同权跑一遍”，而是：

- 主线后端准入必须快、稳、强阻塞
- 次线治理能力必须持续受控
- 替补适配器必须持续可用，但不能拖慢主线
- 生态实验必须制度化，但不能污染日常开发反馈

---

## 2. 分层模型

当前后端 CI 分为四层。

### 2.1 Primary Admission

当前真实交付路径，对 PR 合并和主干稳定性负责。

包含：

- `packages/core/domain`
- `packages/kernel`
- `packages/platform`
- `packages/runtime`
- `packages/contracts/*`
- `packages/shared/utils`
- `packages/features/*`
- `packages/adapters/storage/turso`
- `packages/adapters/telemetry/*`
- `packages/adapters/auth/google-backend`
- `services/*`
- `servers/bff/web-bff`
- `servers/bff/admin-bff`

要求：

- 必须对 PR 提供快速、稳定、强阻塞反馈
- 必须覆盖 `fmt`、`clippy`、`check`、`test`、BFF smoke tests
- 不引入前端、Tauri、SvelteKit 相关检查

### 2.2 Governance And Secondary Runtime

负责平台治理和次线部署单元，不与主线 BFF 准入耦合。

包含：

- `servers/gateway`
- `workers/*`
- `platform/validators/*`
- `platform/generators`

要求：

- 仍然属于后端质量门禁
- 必须继续受 `security`、静态检查、构建和测试约束
- 失败时要明确标识为治理或次线问题，而不是主线服务问题

### 2.3 Alternatives

替补适配器与未来候选方案，持续巡检但不和当前主线同权阻塞。

当前包含：

- `packages/adapters/storage/surrealdb`
- 与其直接耦合的 `domain`、`tenant-service`

要求：

- 替补模块不能失管
- 但替补模块不得拖慢 Turso 主线准入
- 替补线失败只说明候选链路有问题，不代表主线交付失效

### 2.4 Experiments

针对后端生态演进的编译漂移和未来组合验证。

当前包含：

- 主线、次线、替补三个层级的全量编译检查
- `stable` + `beta` toolchain 组合

要求：

- 仅通过 `schedule` 或 `workflow_dispatch` 触发
- 不阻塞日常 PR 主流程
- 用于尽早发现生态升级、工具链变化或替补链路腐烂

---

## 3. Workflow Mapping

当前 workflow 的职责如下。

> **与 gate 的关系**：
> - 本文件描述的 CI 分层仅面向后端 workflow（`.github/workflows/*.yml`）。
> - 根 gate 组合（`scripts/gate.ts`、`just gate-*`、lefthook）由 integration 轨道负责接线。
> - 本轨道不应因接线需求而修改根 gate 组合或前端 workflow。

### 3.1 `.github/workflows/ci.yml`

名称：`CI`

职责：Primary Admission

Jobs：

- `Primary Static Checks`
- `Primary Tests`
- `BFF Smoke Tests`
- `Backend CI`

这是主线准入入口。  
如果此 workflow 失败，应优先视为“当前真实后端交付路径失败”。

### 3.2 `.github/workflows/quality-gate.yml`

名称：`Quality Gate`

职责：Governance And Secondary Runtime

Jobs：

- `Security Audit`
- `Governance Static Checks`
- `Secondary Runtime Validation`
- `Quality Gate Summary`

这是平台治理、网关、worker 和 validators 的质量门禁。  
如果此 workflow 失败，不应与主线 BFF 问题混淆。

### 3.3 `.github/workflows/coverage.yml`

名称：`Coverage`

职责：主线覆盖率统计

只覆盖 Primary Admission 范围。  
不覆盖 SurrealDB 替补线，不覆盖实验线。

### 3.4 `.github/workflows/backend-alternatives.yml`

名称：`Backend Alternatives`

职责：替补适配器巡检

当前用于：

- `storage_surrealdb`
- 与其直接相关的后端模块

### 3.5 `.github/workflows/backend-experiments.yml`

名称：`Backend Experiments`

职责：实验线与编译漂移验证

特点：

- nightly 定时执行
- 支持手动触发
- `stable` 必须通过
- `beta` 允许预警式失败

---

## 4. 触发原则

所有 workflow 都必须遵守以下原则：

- 只监听后端路径
- 不引入 `apps/**`、Tauri、SvelteKit、Web-only 检查
- 主线 workflow 只监听主线范围
- 替补 workflow 只监听替补范围
- 实验 workflow 不进入 PR 强阻塞路径

这意味着：

- 改主线服务，不应无条件触发替补线
- 改替补适配器，不应拖慢主线准入
- 改平台治理模块，应聚焦治理线反馈

> **当前档位**：L2（第二档）稳定阶段。
> 后端 CI 职责是维持第二档的可解释性与稳定性；第三档闭环验证（contract drift strict、generated drift strict）由 integration 轨道决定是否纳入。

---

## 5. 准入语义

### 5.1 什么叫“主线通过”

满足以下条件时，认为当前真实后端主链路通过准入：

- `CI` workflow 通过
- `Coverage` 生成成功
- 相关 `BFF Smoke Tests` 成功

### 5.2 什么叫“治理通过”

满足以下条件时，认为平台治理与次线运行时通过：

- `Quality Gate` workflow 通过

### 5.3 什么叫“替补健康”

满足以下条件时，认为替补链路可继续保留为候选方案：

- `Backend Alternatives` 通过

### 5.4 什么叫“实验稳定”

满足以下条件时，认为当前 boilerplate 对外部生态变化保持可接受兼容性：

- `Backend Experiments` 的 `stable` 通过
- `beta` 失败只作为预警，不阻塞日常开发

---

## 6. 为什么 SurrealDB 不在主线里

当前项目的实际主存储链路是 `Turso`。  
`SurrealDB` 是未来候选方案，不是当前真实交付路径。

因此：

- `storage_turso` 属于 Primary Admission
- `storage_surrealdb` 属于 Alternatives

这样做的原因不是忽略 SurrealDB，而是避免：

- 主线被候选方案拖慢
- 一个未上场适配器让主线持续红灯
- 覆盖率和安全审计结论被非主线模块污染

替补并未失管，而是被单独治理。

---

## 7. 后续扩展规则

未来新增后端模块时，按以下规则归类。

### 7.1 进入 Primary Admission

满足任一条件：

- 当前真实拓扑会部署
- 属于当前主请求链路
- 属于当前主存储、主认证、主观测实现
- 改动后直接影响 BFF 或核心 service 准入

### 7.2 进入 Governance And Secondary Runtime

满足任一条件：

- 属于平台治理、校验、生成能力
- 属于网关、worker、非主同步入口
- 当前会部署，但不是最核心的主开发反馈路径

### 7.3 进入 Alternatives

满足任一条件：

- 是未来候选适配器
- 是主实现的替代方案
- 当前不在真实拓扑里，但未来可能扶正

### 7.4 进入 Experiments

满足任一条件：

- 主要用于生态试验
- 需要跨 toolchain 验证
- 不应阻塞日常 PR，但需要长期观察

---

## 8. 维护约束

维护这套 CI 时，必须遵守以下约束：

- 不得把前端路径重新混入后端 workflow
- 不得把替补模块直接塞回主线 workflow，除非其已成为真实主链路
- 不得让实验线变成 PR 必经路径
- 新增后端模块时，必须先明确它属于主线、治理、替补还是实验
- 若模块角色发生变化，先更新本文档，再调整 workflow

---

## 9. 当前状态基线

本文档对应的 CI 基线为：

- `CI` 通过
- `Quality Gate` 通过
- `Coverage` 通过
- `Backend Alternatives` 通过
- `Backend Experiments` 通过

这意味着当前分层模型已经跑通，可作为后续演进的稳定起点。
