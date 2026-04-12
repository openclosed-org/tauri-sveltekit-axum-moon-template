# 🎯 项目目标与演进路线

> **核心信念**：当前是**模块化单体**，架构设计先行，后续**无痛微服务化**。这是宪法。
> 本项目处于重构过渡期，存在遗留代码和过时文档。本文档是**唯一真理源**，消除一切 Agent 困惑。

---

## 📦 当前架构：模块化单体（Monolith）

**代码状态**：所有业务逻辑已经或应该位于 `services/` 目录下，遵循 Clean Architecture 四层结构。

```
services/
├── counter-service/   ✅ 完整实现（黄金示例 #1）— 租户级，全链路
├── settings-service/  ✅ 完整实现（黄金示例 #2）— 用户级，全链路
├── user-service/      ✅ 完整实现 — domain/application/ports/infrastructure/tests/migrations（缺 HTTP 路由）
├── tenant-service/    ✅ 已迁移 — domain/application/ports/infrastructure/tests/migrations
├── agent-service/     ✅ 已迁移 — domain/application/ports/infrastructure/tests/migrations
├── admin-service/     ✅ 已创建 — application 组合层
├── chat-service/      ❌ 待实现 — 目录是 stub
├── event-bus/         ✅ 部分实现 — EventBus trait + InMemory 适配器 + Outbox
└── ... future services 按需创建
```

**HTTP 服务器**：`servers/api/`（单体 Axum 服务器，聚合所有路由）
**BFF 层**：`servers/bff/web-bff/`（Web 端聚合，已存在但路由不完整）
**桌面端**：`packages/adapters/hosts/tauri/`（Tauri v2 commands）

---

## 🗺️ 演进路线：无痛微服务化

本项目按**目标架构设计**，代码按 Clean Architecture 组织。当需要微服务化时，每个 `services/<name>/` 可直接编译为独立二进制并部署为独立容器，**零业务逻辑修改**。

| 阶段                 | 代码位置                                           | 部署方式                          | 触发条件                              |
| -------------------- | -------------------------------------------------- | --------------------------------- | ------------------------------------- |
| **当前：模块化单体** | `services/` 下各服务为独立 crate，但在同一二进制中 | `servers/api` 单进程              | 现状                                  |
| **未来：微服务拆分** | `services/` 下各服务编译为独立二进制               | 独立 podman 镜像 + K3s Deployment | 团队 > 5人 或 某服务变更频率 > 3次/周 |

**关键原则**：

1. **每个 service 必须可独立 `cargo build -p <name>` 编译通过**
2. **service 之间不得直接依赖**，必须通过 `contracts/events` 通信
3. **新增业务模块一律在 `services/<domain>/` 下创建**，不再往 `packages/core/usecases/` 添加代码
4. **`packages/core/usecases/` 是历史遗留**，所有逻辑应迁移到 `services/`

---

## 🔍 当前服务状态全景

### 完整度矩阵

| 服务           | Domain | Application | Ports | Infrastructure | Tests | Migrations |     HTTP 路由     | Tauri Command | 数据来源                   |
| -------------- | :----: | :---------: | :---: | :------------: | :---: | :--------: | :---------------: | :-----------: | -------------------------- |
| **counter**    |   ✅   |     ✅      |  ✅   |       ✅       |  ✅   |     ✅     | ✅ servers/api + bff |      ✅       | services/counter-service   |
| **settings**   |   ✅   |     ✅      |  ✅   |       ✅       |  ✅   |     ✅     | ✅ servers/api + bff |      ❌       | services/settings-service  |
| **user**       |   ✅   |     ✅      |  ✅   |       ✅       |  ✅   |     ✅     |  ⚠️ stub (已注册)  |      ✅       | services/user-service      |
| **tenant**     |   ✅   |     ✅      |  ✅   |       ✅       |  ✅   |     ✅     | ✅ servers/api + bff |      ✅       | services/tenant-service    |
| **agent**      |   ✅   |     ✅      |  ✅   |       ✅       |  ✅   |     ✅     | ✅ servers/api + bff |      ✅       | services/agent-service     |
| **admin**      |   ✅   |     ✅      |  ❌   |       ❌       |  ❌   |     ❌     | ✅ servers/api + bff |      ✅       | services/admin-service     |
| **chat**       |   ❌   |     ❌      |  ❌   |       ❌       |  ❌   |     ❌     |        ❌         |      ❌       | 无                         |
| **event-bus**  |  N/A   |     N/A     |  ✅   |       ✅       |  ✅   |     ❌     |        ❌         |      ❌       | services/event-bus         |

### 关键发现

1. **counter-service** 是黄金示例 #1 — 租户级（tenant_id 隔离），全链路
2. **settings-service** 是黄金示例 #2 — 用户级（user_sub 隔离），全链路
3. **tenant/agent/admin** ✅ 已从 `packages/core/usecases/` 迁移到 `services/`
4. **user-service** 实现完整，HTTP 路由已注册（stub，待完善实现）
5. **chat-service** 无消费者，目录是 stub（Tauri 无 chat 命令，之前误报）

---

## 📋 新增 Feature 标准工作流

```
1. 定义契约 (packages/contracts/)    — DTO、错误语义、事件 schema
2. 定义 Feature Trait (packages/features/) — service 必须实现的 trait
3. 创建 Service (services/<domain>/)     — Clean Architecture 四层
   ├── domain/       — 纯领域对象（零外部依赖）
   ├── ports/        — 外部依赖抽象（repository trait 等）
   ├── application/  — 用例逻辑编排（依赖 ports trait）
   ├── infrastructure/ — 具体实现（DB/Cache/HTTP client 等）
   ├── contracts/    — re-export packages/contracts/ 的 DTO
   ├── interfaces/   — 工厂函数（可选，供组合层使用）
   └── sync/         — 同步策略（可选，Local-First 项目需要）
4. 编写测试 (services/<domain>/tests/)   — 单元测试（mock）+ 集成测试
5. 编写迁移 (services/<domain>/migrations/) — SQL 建表语句
6. 注册 HTTP 路由 (servers/api/src/routes/ + servers/bff/web-bff/src/handlers/)
7. 注册 Tauri Command (packages/adapters/hosts/tauri/src/commands/)
8. 注册迁移到 AppState (servers/api/src/state.rs)
9. 前端消费 (apps/web/) — 调用 HTTP API 或 Tauri invoke
10. E2E 测试 (apps/web/tests/e2e/)
```

**禁止**：

- ❌ 在 `packages/core/usecases/` 中新增业务逻辑
- ❌ service 之间直接依赖（必须通过 contracts/events）
- ❌ servers/ 中包含业务逻辑（只能做组合/协议转换）
- ❌ apps/ 中调用 services/ 或 adapters/（只能通过 HTTP API 或 SDK）

---

## 🧹 待清理的过时/困惑点

### ✅ 已完成（批次 A + B）

| #   | 问题                                                      | 状态 |
| --- | --------------------------------------------------------- | ---- |
| 1   | README 说 "stub" 但实际有完整实现                         | ✅ 已重写为领域描述 |
| 2   | README 说 "stub" 但实际有 InMemory 实现                   | ✅ 已重写 |
| 3   | tenant/agent 业务逻辑在 usecases/                         | ✅ 已迁移到 services/ |
| 4   | admin 业务逻辑在 usecases/                                | ✅ 已迁移到 services/admin-service/ |
| 5   | AGENTS.md 说 "Phase 0 业务逻辑在 packages/core/usecases/" | ✅ 已更新 |
| 6   | directory_categories.json 指引搜索 usecases               | ✅ 已更新 |
| 7   | `packages/core/usecases/` 历史遗留目录                    | ✅ 已删除 |
| 8   | 所有消费方 `usecases::` import                            | ✅ 全部更新为 services/ |

### 🔄 待处理（P1）

| #   | 问题                                 | 位置                                               | 修复方案                          |
| --- | ------------------------------------ | -------------------------------------------------- | --------------------------------- |
| 9   | user HTTP 路由 stub 待补全           | servers/api/src/routes/user.rs                     | 实现 user-service 查询逻辑        |
| 10  | chat-service 待实现                  | services/chat-service/                             | 新建或确认不需要                  |
| 11  | web-bff 补全 user handler            | servers/bff/web-bff/src/handlers/user.rs           | 镜像 servers/api 路由             |
| 12  | settings Tauri commands              | packages/adapters/hosts/tauri/src/commands/        | 添加 settings 命令                |

### P2 — 可选优化

| #   | 问题                                     | 位置                                    | 修复方案                          |
| --- | ---------------------------------------- | --------------------------------------- | --------------------------------- |
| 13  | agent_service 直接查 counter/tenant 表   | services/agent-service/src/infrastructure/ | 改为通过 service trait 调用       |
| 14  | admin-service 补全独立 domain/ports      | services/admin-service/                 | 从纯组合层扩展为完整四层          |
