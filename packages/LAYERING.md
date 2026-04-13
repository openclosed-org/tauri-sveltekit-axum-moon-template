# Packages 分层说明

> 本文档明确 `packages/` 下各目录的职责和长期定位。

## 最终层（长期保留）

| 目录 | 职责 | 依赖规则 |
|-----|------|---------|
| `kernel/` | 最底层稳定类型（ids, error, money, pagination, tenancy, time） | 不依赖任何业务或框架 |
| `platform/` | 平台能力抽象（config, health, buildinfo, env, service_meta） | 仅依赖 kernel |
| `runtime/` | 运行时端口抽象 + memory adapters（invocation, pubsub, state, workflow, lock, binding, secret, queue） | 仅依赖 kernel |
| `contracts/` | 协议真理源（HTTP DTOs, Event schemas, RPC, Error codes, SDK gen） | 仅依赖 kernel |
| `adapters/` | 外部协议适配器（auth, cache, chains, hosts, protocols, storage, telemetry） | 依赖 runtime ports + contracts |
| `ui/` | Svelte 组件库 | 不依赖 Rust 业务 |

## 过渡层（待收敛）

| 目录 | 当前内容 | 长期去向 | 状态 |
|-----|---------|---------|------|
| `core/domain/` | 端口 trait 定义（LibSQLPort, SurrealDBPort） | → `packages/runtime/ports/` 或保留为服务级 ports | ⚠️ 与 runtime/ports 功能重叠，待统一 |
| `core/state/` | 占位（.gitkeep） | 待实现或移除 | 🚧 空壳 |
| `core/workspace-hack/` | cargo-hakari 统一依赖优化 | 保留为构建优化 | ✅ 长期保留 |
| `features/*` | Feature trait 定义（admin, agent, auth, chat, counter 等 12 个） | → 由各 `services/*/contracts/` 承担 | ⚠️ 与服务级 contracts 重叠，待收敛 |
| `shared/*` | 共享工具（config, env, errors, testing, types, utils） | → `kernel/` 或 `contracts/` 吸收 | ⚠️ 职责模糊 |

## 收敛原则

1. **不为了收敛而大规模重构** — 等待真正出现依赖冲突或职责混乱时逐步迁移
2. **新增代码走最终层** — 新包优先放在 kernel/platform/runtime/contracts/adapters
3. **过渡层不再扩展** — features/shared 下不新增子目录
