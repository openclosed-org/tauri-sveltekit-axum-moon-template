# Services — 领域服务层

> 每个 service 是一个独立 crate，遵循 Clean Architecture，未来可直接编译为独立微服务。

## 目录结构

```
services/
├── counter-service/   ✅ 黄金示例（完整实现）
├── user-service/      ✅ 用户域（用户实体、OAuth、租户绑定）
├── tenant-service/    ⚠️ 租户域（待从 usecases 迁移）
├── agent-service/     ⚠️ Agent 域（待从 usecases 迁移）
├── chat-service/      ❌ 聊天域（待实现）
├── event-bus/         ✅ 事件总线（内存实现 + Outbox）
└── admin-service/     ❌ 管理域（待从 usecases 迁移，目录待创建）
```

## 标准工作流：新增业务模块

```
1. 定义契约        packages/contracts/          — DTO、错误语义
2. 定义 Feature    packages/features/<domain>/  — service 必须实现的 trait
3. 创建 Service    services/<domain>/           — Clean Architecture 四层
   ├── domain/          纯领域对象（零外部依赖）
   ├── ports/           外部依赖抽象（repository trait）
   ├── application/     用例逻辑编排（依赖 ports）
   ├── infrastructure/  具体实现（DB/Cache/HTTP）
   ├── contracts/       re-export packages/contracts/ 的 DTO
   └── lib.rs           公开 barrel
4. 编写测试        services/<domain>/tests/   — 单元 + 集成
5. 编写迁移        services/<domain>/migrations/ — SQL 建表
6. 注册 HTTP 路由  servers/api/src/routes/<domain>.rs
7. 注册 Tauri cmd  packages/adapters/hosts/tauri/src/commands/<domain>.rs
8. 注册迁移        servers/api/src/state.rs 中调用迁移
9. 前端消费        apps/web/src/routes/ 调用 API
```

## 依赖方向

```
packages/contracts/  ←  所有共享类型的单一真理源
        ↑
packages/features/   ←  定义 trait，不得包含实现
        ↑
services/<domain>/   ←  实现 feature trait，依赖 core + contracts
        ↑
servers/             ←  组合层（路由 + 中间件），不得包含业务逻辑
        ↑
apps/                ←  纯展示层，通过 HTTP API 或 SDK 消费
```

**硬规则**：
- ❌ service 之间不得直接依赖（必须通过 contracts/events 通信）
- ❌ servers/ 不得包含业务逻辑
- ❌ 不得在 `packages/core/usecases/` 中新增业务逻辑（历史遗留，待清空）

## 验证

```bash
# 单个 service 编译
cargo build -p counter-service

# 单个 service 测试
cargo test -p counter-service

# 所有 service 编译
cargo check --workspace

# 整个服务器编译
cargo build -p runtime_server
```

详细的目标架构和演进路线见 [docs/GOAL.md](../../docs/GOAL.md)。
