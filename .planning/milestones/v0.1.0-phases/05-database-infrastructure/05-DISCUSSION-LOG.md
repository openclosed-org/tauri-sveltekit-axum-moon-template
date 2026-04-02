# Phase 05: Database & Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 05-database-infrastructure
**Areas discussed:** Database Architecture, DatabaseAdapter Pattern, Docker Infrastructure Scope, Optional Infra Components, HTTP Transport

---

## Database Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| libsql only | Single libsql/turso — simple, boilerplate-friendly. STATE.md/REQUIREMENTS already confirmed. | |
| Dual DB (SurrealDB + libsql) | SurrealDB for server-side, libsql for local App. Full capabilities. | ✓ |
| libsql now, SurrealDB later | Phase 5 libsql only, SurrealDB deferred to v2. | |

**User's choice:** Dual DB (SurrealDB + libsql)
**Notes:** 覆盖了 STATE.md 之前的 "libsql/turso over SurrealDB — Accepted" 决定。用户明确要求两者都保留在 boilerplate 中。

---

## DatabaseAdapter Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Trait-per-DB (Repository pattern) | Each DB has its own Port trait. DI via Axum state / Tauri managed. Rust-idiomatic. | ✓ |
| Enum-based Unified | DatabaseBackend enum with variants. Unified DatabaseAdapter trait. | |
| Separate, no abstraction | Each DB writes its own code. No shared trait. Simple but duplicative. | |

**User's choice:** Trait-per-DB (Repository pattern)
**Notes:** Aligns with ARCHITECTURE.md existing design. domain crate defines traits, runtime_* provides implementations.

---

## Docker Infrastructure Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full docker-compose | SurrealDB container + nginx reverse proxy. | |
| SurrealDB container only | Only containerize SurrealDB server. | |
| No Docker this phase | Code layer only, Docker deferred. | |

**User's choice:** No Docker — 纯 Rust 部署方案
**Notes:** 用户明确说 "我不用 docker"。部署方案：cargo build --release → systemd + Cloudflare Tunnel（后续阶段）。nginx 用 Pingora 替代（也推迟）。

---

## Optional Infra Components

| Component | Decision | Notes |
|-----------|----------|-------|
| Moka cache | ✓ 纳入 Phase 5 | 替代 redis，Rust-native 内存缓存 |
| Quinn HTTP/3 | ✓ 纳入 Phase 5 | Axum 传输层升级 |
| Cloudflare Tunnel | 推迟 | 部署/运维层，独立阶段 |
| apalis job queue | 推迟 | 后台任务，独立阶段 |
| R2 storage | 推迟 | 对象存储，独立阶段 |
| Pingora | 推迟 | 反向代理，部署层 |

**Notes:** redb 也被讨论但选择保持 libsql 做本地存储。redis/rathole/vector 从 Phase 5 范围移除。

---

## HTTP Transport

| Option | Description | Selected |
|--------|-------------|----------|
| TCP primary, HTTP/3 optional | Default Axum TCP, h3 as upgrade. Best compatibility. | |
| HTTP/3 primary, HTTP/2 fallback | Quinn QUIC primary, TCP fallback. | ✓ |
| Dual listener | Both TCP + QUIC simultaneously. Most complex. | |

**User's choice:** HTTP/3 primary + HTTP/2 fallback
**Notes:** "axum 很好接入的有直接用的库"。开发环境 TCP 调试，生产 QUIC 优先。

---

## Agent's Discretion

以下领域用户未指定细节，留给 agent 决策：
- SurrealDB 连接池配置
- Moka cache eviction 策略和容量
- Quinn HTTP/3 证书配置
- libsql migration 文件组织
- reqwest middleware 配置
- Turso 同步触发策略

---

## Deferred Ideas

- Cloudflare Tunnel — 部署阶段
- apalis job queue — 独立阶段
- R2 object storage — 独立阶段
- Pingora — 部署阶段
- nginx — 不使用
- redis — 被 Moka 替代
- rathole — 被 Cloudflare Tunnel 替代
- vector observability — 推迟到 v2
