# 🔄 重构 Handoff 文档

> **生成时间**: 2026-04-11
> **目的**: 为下一个 Agent 提供完整上下文，无需重新阅读历史对话即可继续工作。
> **状态**: 批次 A/B/C/D 已完成，剩余 P1 任务待执行。

---

## 1. 当前状态快照

### 1.1 架构定位

| 维度 | 现状 |
|------|------|
| **架构风格** | 模块化单体（Modular Monolith） |
| **代码组织** | Clean Architecture 四层（domain/ports/application/infrastructure） |
| **部署** | 单二进制（`servers/api`），BFF 层已存在（`servers/bff/web-bff`） |
| **演进目标** | 无痛微服务化 — 每个 `services/<name>/` 可直接编译为独立容器 |

### 1.2 服务完整度矩阵

| 服务 | 四层完整 | Tests | Migrations | HTTP servers/api | HTTP bff | Tauri |
|------|:---:|:---:|:---:|:---:|:---:|:---:|
| counter | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| settings | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| tenant | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| agent | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| admin | ⚠️ 组合层 | ❌ | ❌ | ✅ | ✅ | ✅ |
| user | ✅ | ✅ | ✅ | ⚠️ stub | ❌ | ✅ |
| chat | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| event-bus | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |

### 1.3 依赖方向（宪法级，不可违反）

```
packages/contracts/  ←  单一真理源（HTTP/Event/DTO）
        ↑
packages/features/   ←  定义 trait，不得包含实现
        ↑
services/<domain>/   ←  实现 feature trait，Clean Architecture 四层
        ↑
packages/core/       ←  系统级抽象（TenantId/Error/Config/Clock）+ 基础设施端口
        ↑
packages/adapters/   ←  外部世界翻译层（Turso/SurrealDB/Telemetry/Auth）
        ↑
servers/             ←  组合层（路由 + 中间件），不得包含业务逻辑
        ↑
apps/                ←  纯展示层，通过 HTTP API 或 SDK 消费
```

**硬规则**：
- ❌ service 之间不得直接依赖（必须通过 contracts/events 通信）
- ❌ servers/ 不得包含业务逻辑
- ❌ 不得在 `packages/core/usecases/` 中新增业务逻辑（已删除）
- ❌ apps/ 不得调用 services/ 或 adapters/

### 1.4 两个黄金示例

| 示例 | 服务 | 隔离维度 | 完整链路 |
|------|------|---------|---------|
| #1 | counter-service | **租户级**（tenant_id） | domain → ports → application → infrastructure → HTTP(api+bff) → Tauri → 前端 |
| #2 | settings-service | **用户级**（user_sub） | domain → ports → application → infrastructure → HTTP(api+bff) → tests → migrations |

---

## 2. 已完成工作（本次会话）

### 批次 A：文档修复（消除 Agent 困惑）
- [x] 创建 `docs/GOAL.md` — 统一 Phase 定义、服务状态全景图、标准工作流
- [x] 重写 `AGENTS.md` §0.3 — 删除 "Phase 0 代码在 usecases" 过时描述
- [x] 重写 `agent/directory_categories.json` — 搜索指引指向 services/
- [x] 重写 `services/README.md` — 开发工作流文档
- [x] 重写 7 个 `services/*/README.md` — 删除 Phase checklist，改为领域描述
- [x] 删除 `packages/core/usecases/` 整个目录

### 批次 B：代码迁移（从 usecases → services）
- [x] **tenant-service**: domain/entity, ports/repository, application/service, infrastructure/LibSql+SurrealDb, tests, migrations
- [x] **agent-service**: domain, ports(LlmProvider/ToolExecutor), application, infrastructure(LibSqlAgentRepository), tests, migrations
- [x] **admin-service**: application/AdminDashboardService（组合 tenant + counter）
- [x] 更新 14 处消费方 import（servers/api, servers/bff, Tauri commands, desktop app, tests）
- [x] 更新 6 个 Cargo.toml（根 + servers/api + servers/bff + Tauri + desktop）

### 批次 C：补全缺失链路
- [x] 创建 `servers/api/src/routes/user.rs`（stub，已注册到 api_router）
- [x] 创建 `servers/api/src/routes/settings.rs`（完整实现）
- [x] 创建 `servers/bff/web-bff/src/handlers/settings.rs`（完整实现）
- [x] 注册 settings 路由到 servers/api + servers/bff + OpenAPI doc

### 批次 D：settings-service 教学示例
- [x] 完整 Clean Architecture 四层
- [x] 与 counter-service 的对比：tenant_id vs user_sub 隔离
- [x] 单元测试（3 个：默认值、更新、用户隔离）
- [x] SQL 迁移 + 注册到 AppState
- [x] HTTP 路由（GET/PUT /api/settings）+ API Key 脱敏
- [x] BFF handlers 镜像

---

## 3. 待执行任务（P1 优先级）

### 3.1 user HTTP 路由补全

**现状**: `servers/api/src/routes/user.rs` 已注册但返回 placeholder 消息。

**需要做的**:
1. `user-service` 已有 domain/application/ports/infrastructure
2. 在 `user-service/src/infrastructure/` 添加 LibSqlUserRepository（已有，在 user-service 的现有实现中）
3. 在路由中构建 UserService 并调用查询方法
4. 在 `servers/bff/web-bff/src/handlers/user.rs` 镜像实现
5. 前端 `/user` 页面消费（可选，当前无此页面）

**参考**: 照抄 `counter.rs` 路由结构，替换为 user 的 service 调用。

### 3.2 chat-service 决策

**现状**: `services/chat-service/` 是 stub 目录，无消费者。

**决策树**:
- 如果项目不需要独立聊天功能 → 删除 stub 目录，移除 workspace 引用
- 如果需要 → 按 counter-service 模板补全四层 + HTTP + Tauri

**建议**: 当前 agent 的聊天/对话功能已通过 agent-service 的 `chat_stream` 覆盖。chat-service 可能是为未来独立的实时消息功能预留。**建议标记为 "预留"，不做实现。**

### 3.3 settings Tauri commands

**现状**: settings-service 有完整后端，但 Tauri 端无对应命令。

**需要做的**:
1. 创建 `packages/adapters/hosts/tauri/src/commands/settings.rs`
2. 实现 `settings_get` + `settings_update` 命令
3. 注册到 `mod.rs` + `generate_handler!`
4. 更新 Tauri Cargo.toml 添加 settings-service 依赖
5. 前端 settings 页面检测 Tauri vs Web 环境调用不同路径

**参考**: `commands/counter.rs` 的模式。

### 3.4 admin-service 补全（可选）

**现状**: admin-service 只有 application 组合层，无独立 domain/ports/infrastructure。

**需要做的**（如果决定补全）:
1. 定义 AdminDashboardStats 到 domain/
2. 定义 AdminDashboardRepository port
3. 实现 LibSqlAdminDashboardRepository
4. 但这与当前 admin 作为纯组合层的职责定位冲突
5. **建议**: 保持组合层定位，不扩展为完整四层

### 3.5 agent-service 直接查表问题（P2）

**现状**: `services/agent-service/src/infrastructure/libsql_adapter.rs` 中 `execute_tool_by_name` 直接查询 `counter` 和 `tenant` 表。

**修复**: 改为注入 CounterService 和 TenantLister trait，通过 trait 调用而非直接 SQL。

---

## 4. 新增 Feature 标准工作流（10 步）

每次接到新需求，按此顺序执行：

```
1. 定义契约
   → packages/contracts/api/src/lib.rs
   → 添加 DTO struct（ToSchema + TS export）

2. 定义 Feature Trait
   → packages/features/<domain>/src/lib.rs
   → trait + error enum

3. 创建 Service 目录
   → cp -r services/counter-service services/<domain>-service/
   → 替换所有 counter → <domain>

4. 实现 Domain 层
   → src/domain/entity.rs（纯领域对象，零外部依赖）
   → src/domain/error.rs

5. 实现 Ports 层
   → src/ports/mod.rs（Repository trait）

6. 实现 Application 层
   → src/application/service.rs（用例编排，依赖 ports trait）

7. 实现 Infrastructure 层
   → src/infrastructure/libsql_adapter.rs（LibSQL 实现）

8. 编写 Tests + Migrations
   → tests/unit/<domain>_service_test.rs（mock repository）
   → migrations/001_create_<domain>.sql

9. 注册 HTTP 路由
   → servers/api/src/routes/<domain>.rs
   → servers/bff/web-bff/src/handlers/<domain>.rs
   → 注册到 routes/mod.rs + bff lib.rs
   → 注册迁移到 state.rs

10. Tauri Commands（桌面端需要）
    → packages/adapters/hosts/tauri/src/commands/<domain>.rs
    → 注册到 mod.rs + generate_handler!

11. 前端消费
    → apps/web/src/routes/<domain>/+page.svelte
    → 调用 HTTP API 或 Tauri invoke
```

---

## 5. 关键文件索引

### 5.1 必须读取的入口文件

| 场景 | 文件 |
|------|------|
| 理解架构 | `docs/GOAL.md`, `docs/ARCHITECTURE.md`, `docs/spec.md` |
| Agent 协作 | `AGENTS.md`, `agent/directory_categories.json` |
| 新增服务参考 | `services/counter-service/` (黄金示例 #1), `services/settings-service/` (黄金示例 #2) |
| 运行验证 | `Justfile`, `moon.yml`, `.mise.toml` |
| 依赖版本 | `Cargo.toml` (根) |

### 5.2 禁止读取的目录

| 目录 | 原因 |
|------|------|
| `node_modules/` | 第三方依赖 |
| `target/` | Rust 构建产物 |
| `.moon/cache/` | moon 缓存 |
| `.cocoindex_code/` | 索引缓存 |
| `.jj/` | VCS 内部数据 |
| `packages/core/usecases/` | **已删除**，不应再存在 |

### 5.3 常用验证命令

```bash
# 编译检查
cargo check --workspace

# 单个 service 编译
cargo build -p counter-service
cargo build -p settings-service
cargo build -p tenant-service
cargo build -p agent-service
cargo build -p admin-service

# 单个 service 测试
cargo test -p counter-service
cargo test -p settings-service
cargo test -p tenant-service
cargo test -p agent-service

# 服务器编译
cargo build -p runtime_server        # servers/api
cargo build -p web-bff               # servers/bff/web-bff

# 前端检查
bun run -C apps/web check

# Just 命令
just check          # cargo check --workspace
just test           # 运行测试
just build          # moon run repo:build
```

---

## 6. 下一个 Agent 应该做什么

### 推荐优先级

#### 第一优先级：编译验证
```bash
cargo check --workspace
```
确认本次大规模迁移没有引入编译错误。如有错误，逐个修复。

#### 第二优先级：settings Tauri commands
这是最小改动且直接完善全链路的任务：
1. 创建 `packages/adapters/hosts/tauri/src/commands/settings.rs`
2. 实现 `settings_get` + `settings_update`
3. 注册到 `mod.rs`
4. 更新 Tauri Cargo.toml
5. `cargo check -p runtime_tauri` 验证

#### 第三优先级：user HTTP 路由补全
当前返回 placeholder message。实现真正的 user-service 查询。

#### 第四优先级（可选）：agent-service 直接查表修复
将 `execute_tool_by_name` 中的直接 SQL 查询改为 trait 注入调用。

### 不推荐立即做的

- ❌ chat-service 实现（无消费者，agent-service 已覆盖对话功能）
- ❌ admin-service 扩展为四层（保持组合层定位）
- ❌ 大规模前端改造（settings 页面仍使用 localStorage，迁移到后端 API 是独立任务）

---

## 7. 技术债务清单

| 债务 | 影响 | 修复优先级 |
|------|------|-----------|
| agent-service 直接查 counter/tenant 表 | 服务间耦合 | P2 |
| user HTTP 路由是 stub | 功能不完整 | P1 |
| settings Tauri commands 缺失 | 桌面端无法使用 settings | P1 |
| admin-service 无独立 domain/ports | 组合层无隔离 | P2（可接受） |
| event-bus 无 migrations 注册 | 事件表不存在 | P2 |
| user-service 缺 HTTP 路由在 bff | bff 不完整 | P1 |
| settings 前端仍用 localStorage | 未消费后端 API | P1（前端任务） |

---

## 8. 项目目标与成功标准

详见 `docs/GOAL.md` 和 `docs/spec.md`。

**核心目标**：构建一个 Tauri 2 + SvelteKit + Axum 跨端 monorepo boilerplate，支持：
- 模块化单体 → 无痛微服务化
- 本地优先 + 云端同步
- Agent 100% 开发维护

**成功标准**：
- 新模块生成到 CI 通过 ≤ 5 分钟
- `cargo check --workspace` 零错误
- 新增 feature 遵循 10 步标准工作流
- 无 Agent 困惑点（README/AGENTS.md/directory_categories 与实际代码一致）
