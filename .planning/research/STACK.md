# Stack Research

**Domain:** Agent-Native Cross-Platform Application Engineering Base (Tauri + SvelteKit + Axum)
**Researched:** 2026-04-01
**Confidence:** HIGH (sourced from authoritative blueprint docs)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Tauri v2 | 2.x latest | Desktop app shell | Rust 原生，安全性高，插件生态成熟，确定选型不再考虑替代 |
| SvelteKit 2 + Svelte 5 | 2.x / 5.x | Web frontend framework | 编译时优化，Runes 响应式，性能领先（1200 RPS），确定选型 |
| Axum | 0.8.x | HTTP backend framework | Tokio 生态，类型安全，性能优秀，确定选型 |
| Bun | 1.x | JS/TS runtime + package manager | 性能极快，兼容 npm 生态 |
| moon | latest | Monorepo task orchestration | 专注 monorepo，Rust 编写，确定选型 |
| proto | latest | Multi-language toolchain manager | 多语言支持，Rust 编写 |
| Just | latest | Human/agent task entry | 简洁易用，跨平台，收敛常用操作入口 |

### Supporting Libraries

| Library | Purpose | When to Use |
|---------|---------|-------------|
| serde / serde_json | Serialization | All Rust data boundary code |
| tokio | Async runtime | All server/worker async code |
| tower / tower-http | Middleware stack | Axum middleware (CORS, Trace, Timeout) |
| tracing / tracing-subscriber | Structured logging | All Rust crates, structured JSON output |
| jsonwebtoken | JWT auth | Auth adapter, tenant middleware |
| moka | In-memory cache | Server-side caching layer |
| libsql / rusqlite_migration | Embedded database | Local app storage, vector extension |
| SurrealDB | Multi-model database | Server-side primary DB |
| async-openai | OpenAI API client | Agent conversation feature |
| oauth2 | OAuth 2.0 flows | Google Auth adapter |
| zod | TS schema validation | Frontend form/API validation |
| @testing-library/svelte | Svelte component testing | Vitest unit tests |
| Playwright | E2E testing | Cross-browser E2E tests |
| rstest | Rust test utilities | Rust unit/integration tests |
| ts-rs | Rust→TS type generation | Contracts/typegen pipeline |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| moon | Task orchestration | `repo:setup`, `repo:verify`, `repo:typegen` etc. |
| Just | Task entry | `just dev`, `just verify`, `just typegen` |
| proto | Toolchain pinning | `.prototools` for Rust/Bun/Node versions |
| cargo fmt / clippy | Rust quality | `cargo fmt --check`, `clippy -D warnings` |
| Vitest | JS/TS unit tests | happy-dom environment |
| Playwright | E2E tests | Desktop project, mock OAuth via deep-link |
| release-plz + git-cliff | Release automation | CI/CD, changelog generation |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| React/Vue | 确定选型 SvelteKit 2 + Svelte 5 | SvelteKit 2 + Svelte 5 |
| Express/Fastify | 确定选型 Axum | Axum |
| Electron | Tauri 更优的性能和安全性 | Tauri v2 |
| npm scripts as primary | 不可发现，不可编排 | moon + Just |
| Heavy AI frameworks (LangChain etc.) | 过重，不适合 Rust 生态 | 自研轻量级 agent + async-openai |
| Hand-written mirror types | 导致类型漂移 | contracts → typegen pipeline |

## Sources

- docs/blueprints/agent-native-starter-v1/00-index.md — Technology selection philosophy
- docs/blueprints/agent-native-starter-v1/03-toolchain-and-taskgraph.md — Tool responsibilities
- docs/blueprints/agent-native-starter-v1/06-engineering-standards-rust-tauri-svelte.md — Version baselines
- docs/blueprints/agent-native-starter-v1/13-subdomain-selection-guide.md — Sub-domain selection criteria

---
*Stack research for: v0.2.0 架构蓝图对齐与核心功能实现*
*Researched: 2026-04-01*
