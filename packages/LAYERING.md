# Packages 分层说明

> 本文档明确 `packages/` 下各目录的职责和长期定位。
> 最后更新：2026-04-12（清理后）

## 最终层（长期保留）

| 目录 | 职责 | 依赖规则 |
|-----|------|---------|
| `kernel/` | 最底层稳定类型（TenantId, UserId, Cursor, AppError） | 不依赖任何业务或框架 |
| `runtime/` | 运行时端口抽象 + memory adapters（Invocation, PubSub, State, Workflow, Lock, Binding, Secret, Queue） | 仅依赖 kernel + contracts/events |
| `contracts/` | 协议真理源（api DTOs, auth, Event schemas, Errors） | 仅依赖 workspace-hack |
| `features/` | Feature trait 定义（admin, agent, auth, chat, counter, settings） | 依赖 contracts + workspace-hack |
| `adapters/` | 外部协议适配器（auth/google, hosts/tauri, storage/surrealdb+турso, telemetry/otel+tracing） | 依赖 runtime ports + contracts |
| `shared/utils/` | 纯工具函数（ID 生成、时间格式化、加密工具） | 仅外部依赖 |
| `ui/` | Svelte 组件库 | 不依赖 Rust 业务 |

## 特殊层

| 目录 | 职责 | 状态 |
|-----|------|------|
| `platform/` | 平台能力抽象（ConfigProvider, TelemetryProvider, Clock） | ⚠️ 当前零依赖，保留为未来使用 |
| `core/domain/` | 数据库端口 trait（LibSqlPort, SurrealDbPort）+ TenantId 重导出 | ✅ 服务级数据库抽象，TenantId 统一来自 kernel |
| `core/workspace-hack/` | cargo-hakari 统一依赖优化 | ✅ 构建优化，长期保留 |
| `sdk/` | SDK 生成占位 | ⚠️ 策略已定义，待迁移条件触发 |

## 已清理项

以下目录已删除（零依赖 stub 或空壳）：

| 删除项 | 原因 |
|-------|------|
| `core/state/` | 空占位 |
| `shared/errors/` | 仅 re-export contracts_errors，零依赖 |
| `shared/types/` | 空占位 |
| `shared/config/` | 空占位 |
| `shared/env/` | 空占位 |
| `shared/testing/` | 空占位 |
| `features/feed/` | 零依赖 stub |
| `features/notifications/` | 零依赖 stub |
| `features/payments/` | 零依赖 stub |
| `features/profile/` | 零依赖 stub |
| `features/social-graph/` | 零依赖 stub |
| `features/wallet/` | 零依赖 stub |
| `adapters/auth/{dpop,oauth,passkey}/` | 空目录 |
| `adapters/cache/` | 空 stub |
| `adapters/chains/` | 空目录（base/evm/solana/ton） |
| `adapters/protocols/` | 空目录（atproto/farcaster/nostr） |
| `adapters/hosts/{base-app,browser-extension,farcaster-miniapp,telegram-miniapp}/` | 空目录 |
| `adapters/storage/{extension-storage,indexeddb,sqlite,tauri-store}/` | 空 stub |
| `contracts/{protocols,codegen,generated,ui}/` | 空目录/infrastructure |

## 新增包规则

1. **Foundation 层**（kernel/runtime/contracts）：需要 2+ 个服务或 worker 共同需要
2. **Feature 层**：需要 1+ 个 service 实际实现其 trait
3. **Adapter 层**：需要有真实的外部协议接入代码
4. **禁止**：为未来可能的需求预建空目录
