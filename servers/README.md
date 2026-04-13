# Servers

Composition layer for standalone services. Each server is a **binary** that wires together services, adapters, and domain logic.

## Structure

```
servers/
├── api/          # ✅ Axum HTTP API service（综合入口，含 admin/agent/counter/settings/user 路由）
├── bff/
│   ├── web-bff/     # ✅ Web BFF（已实现 handlers/middleware/tenant 注入）
│   ├── admin-bff/   # ✅ Admin BFF（已实现 dashboard/tenant/health/metrics）
│   └── mobile-bff/  # ⚠️ Mobile BFF（占位，待实现）
├── gateway/      # ⚠️ API gateway（占位 — Pingora 计划，仅有 Cargo.toml + .gitkeep）
├── indexer/      # ⚠️ 与 workers/indexer 职责冲突，待清理或明确定位
└── realtime/     # ⚠️ Realtime service（占位 — WebSocket/SSE 计划）
```

## Responsibility

Servers are **composition roots only**:
- Register middleware (CORS, tracing, tenant validation)
- Wire routes to service implementations
- Initialize connections (DB, cache, HTTP client)
- **No business logic** — all logic lives in `services/*`

## Current State

### API Server (`servers/api/`)
- Primary HTTP API entry point
- OpenAPI spec: `servers/api/openapi.yaml`
- Routes: admin, agent, counter, settings, user
- State: `servers/api/src/state.rs` wires migrations and service handlers
- Tests: `servers/api/tests/http_e2e_test.rs`, `tracing_test.rs`

### BFFs (`servers/bff/`)
- **web-bff**: Full implementation with user/agent/admin/settings handlers, JWT middleware, tenant middleware
- **admin-bff**: Admin dashboard, tenant management, health/metrics endpoints
- **mobile-bff**: Placeholder, no implementation yet

### Gateway (`servers/gateway/`)
- Placeholder only (`.gitkeep` + minimal `Cargo.toml`)
- Future: Pingora-based API gateway

### Indexer (`servers/indexer`)
- ⚠️ **职责冲突**：与 `workers/indexer` 同名
- 当前有 lib.rs + sources/transformers/sinks 结构
- 待确认：应迁移到 workers、还是保留为 server 特有的索引服务？

### Realtime (`servers/realtime/`)
- Placeholder for future WebSocket/SSE realtime service

## Dependency Direction

```
services/*        ←  业务能力库（纯逻辑）
    ↑
servers/*         ←  组合层（路由 + 中间件），不得包含业务逻辑
    ↑
apps/*            ←  前端/客户端，通过 HTTP API 或 SDK 消费
```

**硬规则**：
- ❌ server handler 中不得包含业务规则
- ❌ server 不得直接操作数据库（必须通过 service 端口）
- ❌ 不得在 server 中驻留长时异步任务（应放入 workers/*）
- ⚠️ servers/indexer 与 workers/indexer 职责需清理

## 验证

```bash
# 单个 server 编译
cargo build -p api
cargo build -p web-bff
cargo build -p admin-bff

# 整个服务器编译
cargo build --workspace

# OpenAPI 验证（如果有 contract test）
just test-contract
```

## BFF OpenAPI 策略

当前仅 `servers/api/openapi.yaml` 存在。BFF 的协议文档策略待决定：
- 方案 A：每个 BFF 维护独立 `openapi.yaml`
- 方案 B：BFF 不暴露独立 OpenAPI，仅作为 API 聚合层
- 详见 `docs/architecture-gap-priority-plan.md` §P1-5
