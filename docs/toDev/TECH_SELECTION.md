# 技术选型文档 - Template Boilerplate

> 基于 rust-docs.md (TWiR 2024-09 ~ 2026-03) + 用户需求
> 核心目标: 跑通 3 个页面(登录、计数器、admin)的开发基建
> 基准日期: 2026-03-28

---

## 一、项目定位

**当前状态**: Template boilerplate，只有 3 个页面，几乎无业务逻辑
**核心需求**: 开发环境跑通、工具基建就绪、测试通过
**不做**:不过度设计业务逻辑、不预加未来功能

---

## 二、核心栈 (已确认)

| 层级 | 选型 | 依据 |
|------|------|------|
| 桌面 Shell | **Tauri 2.x** | rust-docs.md L35 确认 v2.0 |
| 前端 | **SvelteKit 2.x + Svelte 5** | Runes 稳定 |
| 后端 | **Axum 0.8.x** | 主流 HTTP 框架 |
| 构建 | **moon** | 已集成 |
| 包管理 | **bun** | 已配置 |

---

## 三、基建选型 (基于 rust-docs.md)

### 3.1 数据库 - 核心依赖

| 方案 | 选型 | 依据 | 状态 |
|------|------|------|------|
| 主数据库 | **SurrealDB** (embedded) | 用户明确要求，需跑通 | ⚠️ 需验证 |
| Embedded KV | **redb** | rust-docs.md L155 纯 Rust 3.0.0 | 可选备选 |
| SQLite 重写 | **Limbo** | rust-docs.md L143 Turso 主推 | 观察 |

**SurrealDB 嵌入式配置**:
```toml
# Cargo.toml
surrealdb = { version = "3", features = ["kv-mem"] }
```
支持 embedded 模式，可在 Tauri 中直接使用。

### 3.2 代码索引 MCP

| 工具 | 用途 | 选型 | 来源 |
|------|------|------|------|
| 代码语义搜索 | 理解整个代码库 | **indxr** | rust-docs.md L599 |
| 代码诊断 | rust-analyzer 集成 | **rust-analyzer-mcp** | 外部生态 |
| 项目分析 | Crate 索引 | **cratedex** | 外部生态 |

**indxr** (rust-docs.md 收录):
- Fast codebase indexer for AI coding agents
- MCP server 集成
- 支持 semantic + BM25 混合搜索

### 3.3 Web 能力 MCP

| 工具 | 用途 | 选型 |
|------|------|------|
| 网页搜索 | 实时信息获取 | **Exa MCP** |
| 网页浏览 | 自动化测试 | **Chrome DevTools MCP** |

### 3.4 HTTP 客户端

| 工具 | 用途 | 选型 | 来源 |
|------|------|------|------|
| HTTP 请求 | 外部 API 调用 | **reqwest 0.13** | rust-docs.md L287 |

**reqwest 0.13 特性**:
```toml
# rustls 默认，无需 OpenSSL
reqwest = { version = "0.13", default-features = false, features = ["rustls"] }
```

### 3.5 搜索/索引

| 工具 | 用途 | 选型 | 来源 |
|------|------|------|------|
| 全文搜索 | 文档检索 | **Tantivy** | rust-docs.md L167 |
| 向量搜索 | AI 场景 | Qdrant | 外部生态 |

** Tantivy ** (rust-docs.md 收录):
- Quickwit 开发，成熟稳定
- 完全 Rust，无 Java 依赖

### 3.6 测试框架

| 类型 | 选型 | 来源 |
|------|------|------|
| Rust 单元 | **cargo test** + **rstest** | 官方 |
| Rust 集成 | **tokio test** | 官方 |
| Svelte 组件 | **vitest** + **vitest-browser-svelte** | Svelte 官方 |
| E2E | **Playwright** | 行业标准 |

### 3.7 容器/基础设施 (评估中)

| 工具 | 用途 | 状态 |
|------|------|------|
| OCI 运行时 | youki (7.3K ⭐) | 评估中，暂不采用 |
| 容器管理 | docker-compose | 开发环境必需 |

---

## 四、Tauri 相关 Cargo 依赖

基于 rust-docs.md 收录项目和官方文档:

### 4.1 核心插件

```toml
[dependencies]
# 官方核心插件
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-fs = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-window-state = "2"
```

### 4.2 数据库

```toml
# 主数据库 - SurrealDB embedded
surrealdb = { version = "3", features = ["kv-mem"] }

# 备选 - 纯 Rust embedded KV
redb = "3.0.0"

# 备选 - SQLite Rust 重写 (观察)
# limbo = "0.1"  # rust-docs.md L143
```

### 4.3 HTTP/网络

```toml
# HTTP 客户端 (rust-docs.md L287 确认 0.13)
reqwest = "0.13"

# WebSocket
tokio-tungstenite = "0.24"

# UUID
uuid = { version = "1", features = ["v4", "serde"] }
```

### 4.4 序列化

```toml
# JSON
serde = "1"
serde_json = "1"
tokio-postgres = "0.5"  # 如果用 Postgres
```

### 4.5 认证

```toml
# JWT
jsonwebtoken = "9"

# Session
cookie = "0.18"
```

---

## 五、Frontend (SvelteKit) 依赖

基于用户要求和 rust-docs.md:

```json
{
  "dependencies": {
    "bits-ui": "^2.16.4",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.50.0",
    "svelte": "^5.54.0",
    "vite": "^8.0.0",
    "vitest": "^3.0.0",
    "vitest-browser-svelte": "^1.0.0",
    "@playwright/test": "^1.50.0"
  },
  "optionalDependencies": {
    "vitepress": "^1.0.0",
    "lucide-svelte": "^0.500.0"
  }
}
```

---

## 六、MCP / Skills 清单

### 6.1 MCP Servers (已配置/需配置)

| MCP | 用途 | 状态 |
|-----|------|------|
| Exa Search | 实时 web 搜索 | ✅ 已配 |
| Chrome DevTools | 浏览器自动化 | ✅ 已配 |
| indxr | 代码索引 | 🔲 需配置 |
| rust-analyzer | 代码诊断 | 🔲 需配置 |

### 6.2 Skills (已加载)

来自 `/skill list`:

- **rust-skills** (179 rules)
- **svelte-code-writer**
- **svelte-core-bestpractices**
- **frontend-patterns**
- **backend-patterns**
- **postgres-patterns** ⚠️ 需改为 surrealdb
- **docker-patterns**
- **api-design**
- **security-review**
- **testing** (via run-tests)

---

## 七、Phase 1 具体依赖清单

### 7.1 package.json (前端)

dependencies:
- `bits-ui@^2.16.4` - UI 组件
- `tailwindcss@^4.0.0` - CSS 框架

devDependencies:
- `@sveltejs/adapter-static@^3.0.0` - Tauri SPA 适配
- `@sveltejs/kit@^2.50.0`
- `svelte@^5.54.0`
- `vite@^8.0.0`
- `vitest@^3.0.0`
- `vitest-browser-svelte@^1.0.0`
- `@playwright/test@^1.50.0`

可选 (注释状态):
- `vitepress` - 文档
- `lucide-svelte` - 图标
- `lottie-web` - 动画

### 7.2 Cargo.toml (Rust 端)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-fs = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-window-state = "2"

# SurrealDB embedded
surrealdb = { version = "3", features = ["kv-mem"] }

# HTTP
reqwest = "0.13"
tokio = { version = "1", features = ["full"] }

# Serialization
serde = "1"
serde_json = "1"

# Auth
jsonwebtoken = "9"
cookie = "0.18"

# Utils
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
rstest = "0.22"
mockito = "1.6"
```

### 7.3 moon.yml 任务配置

```yaml
tasks:
  lint:
    command: 'biome check .'
    inputs:
      - '@globs(*)'
  
  test:unit:
    command: 'cargo test --lib'
    inputs:
      - '@rustfiles(crate/**)'
  
  test:comp:
    command: 'vitest run'
    inputs:
      - '@tsfiles(src/**/*.ts)'
  
  test:e2e:
    command: 'playwright test'
    inputs:
      - '@globs(tests/**/*.ts)'

  dev:
    command: 'conc "vite dev" "cargo tauri dev"'
    deps:
      - task: 'build'
```

---

## 八、风险与验证

### 8.1 SurrealDB 验证点

- [ ] embedded 模式内存占用 (Tauri bundle 大小)
- [ ] 与 Tauri plugin 共存稳定性
- [ ] 多租户数据隔离实现复杂度
- [ ] v3.0 稳定性测试

### 8.2 替代方案

如 SurrealDB 不适合:
- **redb** (rust-docs.md 收录 L155) - 纯 KV，简单场景
- **libsql** - Turso 主推，已有 tauri-plugin-libsql

---

## 九、结论

| 类别 | 选型 | 依据 |
|------|------|------|
| 数据库 | **SurrealDB embedded** | 用户明确要求，6月前验证 |
| 代码索引 | **indxr** | rust-docs.md L599，MCP 集成 |
| HTTP | **reqwest 0.13** | rust-docs.md L287 |
| 搜索 | **Tantivy** | rust-docs.md L167 |
| 测试 | vitest + Playwright | Svelte 官方推荐 |

**核心不变**: Tauri 2 + SvelteKit + Axum + moon

需要继续生成 Phase 1 的具体 Cargo.toml 和 package.json 依赖配置吗？