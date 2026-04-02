# Phase 04: Backend Dependencies & Build Optimization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 04-backend-dependencies-build-optimization
**Areas discussed:** Axum 服务器架构, Binary 优化策略, 依赖分层与版本策略, 基础设施预加载

---

## Axum 服务器架构

| Option | Description | Selected |
|--------|-------------|----------|
| 完整中间件栈 | CORS + tower-http trace/timeout + request-id | ✓ |
| 最简中间件 | 仅 trace + timeout，不含 CORS | |
| CORS only | 仅 CORS，不含 trace/timeout | |

| Option | Description | Selected |
|--------|-------------|----------|
| 集中式 Router | runtime_server 内部 define create_router() | |
| 模块化 Router | 每模块导出 router()，顶层合并 | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| /healthz 端点 | GET /healthz 返回 {status:ok} | |
| /health 端点 | GET /health 返回详细状态 | |
| 分层健康检查 | /healthz + /readyz | ✓ |

**Follow-up (middleware):** tower-http 全套 + 自定义 request-id 中间件 (tower-http 0.6+)

**Follow-up (router structure):** mod.rs per module — crates/runtime_server/src/routes/health.rs, api.rs, mod.rs

---

## Binary 优化策略

| Option | Description | Selected |
|--------|-------------|----------|
| abort on panic | Release panic=abort | |
| unwind (default) | panic=unwind, 更安全 | |
| abort in release, unwind in debug | Profile 级别区分 | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| cargo-size 集成 | moon task: cargo bloat --release --crates | ✓ |
| CI 门控检查 | CI 中检查 binary <= 15MB | |
| 不监控 | 配置好不管了 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 profile | 所有平台共享同一 release profile | |
| 平台特定 profile | 平台级微调 | ✓ |

**Follow-up (platform strip):** Windows: strip debug symbols 保留 symbol table, macOS/Linux: strip all

---

## 依赖分层与版本策略

| Option | Description | Selected |
|--------|-------------|----------|
| tower + tower-http + hyper | workspace 声明三者统一管理 | ✓ |
| 仅 tower-http | workspace 只声明 tower-http | |
| 不走 workspace | runtime_server 直接声明 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 精确版本共享, 宽松 crate | workspace 精确版本, crate workspace=true | ✓ |
| SemVer 范围 | workspace 用 ^x.y | |
| 全精确版本 | 所有依赖精确版本号 | |

| Option | Description | Selected |
|--------|-------------|----------|
| axum-json + axum-extra | json 特性 + cookie/query | ✓ |
| axum 默认特性 | 不额外启用 | |
| axum 全特性 | json, ws, query, multipart 全部启用 | |

---

## 基础设施预加载

| Option | Description | Selected |
|--------|-------------|----------|
| 注释块 + 注释说明 | Cargo.toml 中 # 注释 + 用途说明 | ✓ |
| 独立 [section] 注释 | workspace 加注释段落 | |
| 不做预加载 | 不预留任何未来依赖 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 5-8 全部 | DB驱动, tunnel, 可观测性, OAuth 等 | ✓ |
| 仅 Phase 5 | 仅下一阶段 | |
| 核心依赖 | 仅数据库 + HTTP | |

---

## Agent's Discretion

- tower-http TimeoutLayer 默认超时值
- CorsLayer 具体 allow_origins 配置
- request-id 中间件实现细节
- moon task 中 cargo-bloat 具体参数
