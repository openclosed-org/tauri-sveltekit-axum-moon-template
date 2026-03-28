# 技术选型文档 - Template Boilerplate

> 基于 goal.md 需求 + rust-docs.md (TWiR 2024-09 ~ 2026-03) + exa mcp 实时验证
> 核心目标: 跑通 3 个页面(登录、计数器、admin)的开发基建
> 基准日期: 2026-03-28

---

## 一、项目定位与目标

### 1.1 核心需求 (goal.md)

- **页面**: 登录页 + 计数器页 + Admin 管理端 (共 3 页)
- **认证**: Google 一键登录
- **多租户**: tenant_id 数据隔离
- **基建**: MCP 代码索引 + WebSearch + 各领域 Skills
- **平台**: Desktop + Mobile Web (mobile-first, 响应式)
- **测试**: 编译成功 + 测试通过

### 1.2 技术框架

| 层级 | 选型 | 依据 |
|------|------|------|
| 桌面 Shell | **Tauri 2.x** | goal.md 要求 |
| 前端框架 | **SvelteKit 2.x + Svelte 5** | goal.md 要求 |
| 后端 HTTP | **Axum** | goal.md 要求 |
| 构建工具 | **moon** | goal.md 要求 + 已集成 |
| 包管理器 | **bun** | 已配置 |

---

## 二、数据库策略 (核心)

### 2.1 双数据库架构

用户明确要求:
- **服务端**: SurrealDB (独立部署)
- **本 App**: libsql/turso (嵌入式本地存储)
- **云端同步**: turso cloud (可选)

### 2.2 策略模式 (Adapter Pattern)

```rust
// 数据库抽象层 - 策略模式
pub trait DatabaseAdapter {
    async fn query(&self, sql: &str) -> Result<Vec<Row>>;
    async fn execute(&self, sql: &str) -> Result<()>;
    async fn sync(&self) -> Result<()>;  // 可选云同步
}

// 实现变体
pub enum DatabaseBackend {
    SurrealDB(SurrealDBAdapter),   // 服务端
    LibSQL(LibSQLAdapter),          // 本地嵌入式
    Turso(TursoCloudAdapter),      // 云端同步
}
```

### 2.3 依赖配置

| 用途 | Crate | Features |
|------|-------|----------|
| 本地存储 | `tauri-plugin-libsql` v0.1.0 | embedded + encryption |
| 云端同步 | `libsql` + turso-sdk | embedded-replica |
| 服务端 | `surrealdb` v3.x | kv-mem / kv-rocksdb |

```toml
# Cargo.toml
[dependencies]
# 本地存储 (Tauri Plugin)
tauri-plugin-libsql = "0.1.0"

# 云端同步
libsql = { version = "0.4", features = ["sync"] }

# 嵌入式测试 (开发用)
embedded-sqlite = "0.4"

# 服务端数据库 (Axum Server)
surrealdb = { version = "3", features = ["kv-rocksdb"] }
```

---

## 三、前端技术栈 (goal.md)

### 3.1 核心依赖

| 类别 | 包 | 版本 | 状态 |
|------|-----|------|------|
| UI 组件 | `bits-ui` | ^2.16.4 | ✅ 已配 |
| CSS 框架 | `tailwindcss` | ^4.0.0 | ✅ 已配 |
| Svelte | `svelte` | ^5.54.0 | ✅ 已配 |
| SvelteKit | `@sveltejs/kit` | ^2.50.0 | ✅ 已配 |
| 适配器 | `@sveltejs/adapter-static` | ^3.0.0 | ✅ 已配 |

### 3.2 扩展依赖 (goals 要求)

```json
{
  "dependencies": {
    // 图标 - 用户指定 pqoqubbw/icons (动画图标，7.3K ⭐)
    "@pqoqubbw/icons": "latest",
    "@lucide/svelte": "^1.7.0",
    // 动画 - 用户指定 lottieplayer
    "@lottiefiles/svelte-lottie-player": "^0.3.1"
  },
  "devDependencies": {
    // 文档 - 推荐 mdBook (Rust 单二进制)
    "mdbook": "^0.4",
    // 测试
    "vitest": "^3.0.0",
    "vitest-browser-svelte": "^1.0.0",
    "playwright": "^1.50.0"
  },
  "optionalDependencies": {
    // 文档备选 - 仅构建时需要 Node.js
    // "vitepress": "^1.6.4",
    // "sveltepress": "latest"
  }
}
```

### 3.3 图标方案 (exa 验证 + 用户指定)

用户指定: **pqoqubbw/icons** (7.3K ⭐, 基于 Lucide 的动画图标)

| 方案 | 包 | 动画 | Svelte 支持 | 备注 |
|------|-----|------|-------------|------|
| 主选静态 | `@lucide/svelte` | 无 | ✅ Svelte 5 | 基础图标 |
| 主选动画 | `pqoqubbw/icons` | ✅ 379+ | ⚠️ 需封装 | 用户指定 |
| Lottie | `@lottiefiles/svelte-lottie-player` | ✅ | ✅ | 复杂动画 |

**pqoqubbw/icons 使用方式**:
```bash
# 安装
npm install @pqoqubbw/icons
```

```svelte
<!-- Svelte 封装使用 -->
<script>
  import { ArrowRight } from '@pqoqubbw/icons/svelte';
</script>

<ArrowRight class="w-5 h-5 animate-pulse" />
```

### 3.4 文档站点替代方案 (VitePress vs Rust)

**问题**: VitePress 需要 Node.js 运行吗?

**答案**: 
- **开发时** (`npm run docs:dev`): 需要 Node.js 运行 Dev Server
- **构建后** (`npm run docs:build`): 产出纯静态 HTML，可以部署到任何静态托管
- **生产部署**: 不需要 Node.js，只需要 Nginx/Apache 或 CDN

**Rust 替代方案** (无需运行时):

| 工具 | 类型 | 特点 | 适用场景 |
|------|------|------|----------|
| **mdBook** | Rust 官方 | 单二进制，Rust 文档标准 | API 文档/内部分档 |
| **ddoc** | Rust | 快速，简单 | 博客/简单文档 |
| **SveltePress** | SvelteKit | SPA，可交互 | 需要动态组件 |
| **VitePress** | Node.js | 功能最全 | 需要搜索/国际化 |

**推荐**: 使用 **mdBook** 替代 VitePress

```toml
# Cargo.toml
[dev-dependencies]
mdbook = "0.4"

# 构建命令
cargo install mdbook
mdbook build ./docs

# 输出: docs/book/ (纯静态 HTML)
```

mdBook 优势:
- 单二进制，无依赖
- 主题可定制 (可用 rust-lang 主题)
- 构建速度极快
- 可集成到 cargo 命令 |

---

## 四、移动端测试框架 (2026)

### 4.1 测试框架对比

| 框架 | 适用场景 | Tauri 兼容 | 备注 |
|------|---------|-----------|------|
| **Playwright** | Web E2E + Mobile | ✅ 主力推荐 | 支持 viewport 模拟 |
| **Appium** | 原生 iOS/Android | ⚠️ 需额外配置 | 成熟但重 |
| **Maestro** | 轻量移动测试 | ⚠️ 评估中 | 2026 崛起 |
| **Detox** | React Native | ❌ 不适用 | 仅 RN |

### 4.2 Tauri 移动端测试建议

```yaml
# moon.yml
tasks:
  test:mobile:
    command: 'playwright test --config=playwright.mobile.config.ts'
    inputs:
      - '@tsfiles(tests/mobile/**/*.ts)'

  # AndroidEmulator
  test:android:
    command: 'playwright test --device=Pixel5'
    
  # iOS Simulator (macOS only)
  test:ios:
    command: 'playwright test --device=iPhone14'
```

### 4.3 选型决策

- **Web/Mobile Web E2E**: Playwright ✅ (同一套测试，viewport 切换)
- **移动原生 (远期)**: Appium (需要时添加)
- **理由**: Tauri WebView 本质是 Web，Playwright 完全覆盖

---

## 五、MCP / Skills 配置

### 5.1 MCP Servers

| MCP | 用途 | 状态 |
|-----|------|------|
| **Exa Search** | 实时 web 搜索 | ✅ 已配 |
| **Chrome DevTools** | 浏览器自动化 | ✅ 已配 |
| **indxr** | 代码库语义索引 | 🔲 需配置 |
| **rust-analyzer-mcp** | Rust 代码诊断 | 🔲 需配置 |

### 5.2 Skills (各领域)

| 领域 | Skill | 状态 |
|------|-------|------|
| Rust | `rust-skills` (179 rules) | ✅ 已配 |
| Svelte | `svelte-code-writer` | ✅ 已配 |
| Svelte | `svelte-core-bestpractices` | ✅ 已配 |
| Frontend | `frontend-patterns` | ✅ 已配 |
| Backend | `backend-patterns` | ✅ 已配 |
| Database | `postgres-patterns` | ⚠️ 需改为 surrealdb |
| Test | `e2e-testing` | ✅ 已配 |
| Docker | `docker-patterns` | ✅ 已配 |
| API | `api-design` | ✅ 已配 |
| Security | `security-review` | ✅ 已配 |

---

## 六、Monorepo 目录架构

### 6.1 当前结构

```
tauri-sveltekit-axum-moon-template/
├── apps/
│   └── desktop-ui/           # Tauri App
│       ├── src/              # SvelteKit 前端
│       └── src-tauri/        # Tauri Rust 后端
├── crates/
│   ├── domain/               # 领域模型
│   ├── application/          # 应用逻辑
│   ├── runtime_tauri/        # Tauri IPC 运行时
│   ├── runtime_server/       # Axum HTTP 运行时
│   └── shared_contracts/     # 共享 DTO
├── docs/                     # 文档
├── .planning/                # GSD 规划
└── moon.yml                  # 构建配置
```

### 6.2 扩展后结构

```
tauri-sveltekit-axum-moon-template/
├── apps/
│   ├── desktop-ui/           # Tauri App (桌面)
│   │   ├── src/              # SvelteKit 前端
│   │   ├── src-tauri/        # Tauri Rust 后端
│   │   └── package.json
│   ├── mobile-web/           # 移动端 Web (可选 PWA)
│   │   ├── src/
│   │   └── package.json
│   └── docs/                 # VitePress 文档站点
│       ├── src/
│       └── package.json
├── crates/
│   ├── domain/               # 领域模型 (pure Rust)
│   ├── application/          # 应用逻辑
│   ├── adapters/             # 数据库适配器 ⭐ NEW
│   │   ├── libsql_adapter/
│   │   ├── turso_adapter/
│   │   └── surrealdb_adapter/
│   ├── runtime_tauri/        # Tauri IPC
│   ├── runtime_server/       # Axum REST
│   └── shared_contracts/     # 共享 DTO
├── packages/
│   ├── ui/                   # 共享 UI 组件库
│   │   └── src/components/
│   └── utils/                # 共享工具函数
├── services/
│   ├── auth/                 # 认证服务
│   ├── storage/              # 存储服务
│   └── sync/                 # 云同步服务
├── docker/
│   ├── docker-compose.yml    # 开发环境
│   ├── nginx.conf
│   └── Dockerfile
└── moon.yml
```

### 6.3 命名规范 (BEM + Rust conventions)

```bash
# 前端 (SvelteKit)
components/
├── Button.svelte              # 基础组件 (BEM: .btn)
├── Icon.svelte               # 图标封装
├── LoginForm.svelte          # 业务组件
└── counter/
    └── Counter.svelte        # 页面级组件

# 后端 (Rust)
modules/
├── domain/
│   ├── entities/             # User, Tenant, Counter
│   ├── value_objects/        # Email, TenantId
│   └── repositories/         # traits
├── adapters/
│   ├── database/
│   │   ├── libsql_repo.rs
│   │   └── surrealdb_repo.rs
│   └── cache/
│       └── redis_adapter.rs
└── services/
    ├── auth_service.rs
    └── sync_service.rs
```

---

## 七、关键 Cargo 依赖

### 7.1 服务端 (Axum)

```toml
[dependencies]
# HTTP
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.13", features = ["rustls"] }

# Database - SurrealDB 服务端
surrealdb = { version = "3", features = ["kv-rocksdb"] }

# Auth
jsonwebtoken = "9"
cookie = "0.18"

# Serialization
serde = "1"
serde_json = "1"

# Utils
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 7.2 客户端 (Tauri Plugin)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"          # Sesssion 存储
tauri-plugin-fs = "2"
tauri-plugin-deep-link = "2"      # OAuth 回调
tauri-plugin-window-state = "2"
tauri-plugin-libsql = "0.1.0"      # ⭐ 本地数据库

# 数据库客户端
libsql = "0.4"

# 服务端通信
reqwest = { version = "0.13", features = ["rustls"] }
```

---

## 八、Phase 1 依赖清单

### 8.1 package.json (前端核心)

```json
{
  "name": "desktop-ui",
  "packageManager": "bun@1.3.11",
  "type": "module",
  "scripts": {
    "dev": "concurrently \"vite dev\" \"cargo tauri dev\"",
    "build": "vite build && cargo tauri build",
    "test:unit": "vitest run",
    "test:e2e": "playwright test",
    "lint": "biome check .",
    "format": "biome format --write ."
  },
  "dependencies": {
    "bits-ui": "^2.16.4",
    "@lucide/svelte": "^1.7.0",
    "@lottiefiles/svelte-lottie-player": "^0.3.1"
  },
  "devDependencies": {
    "svelte": "^5.54.0",
    "@sveltejs/kit": "^2.50.0",
    "@sveltejs/adapter-static": "^3.0.0",
    "vite": "^8.0.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0",
    "vitest": "^3.0.0",
    "vitest-browser-svelte": "^1.0.0",
    "@playwright/test": "^1.50.0",
    "@biomejs/biome": "^1.9.4",
    "typescript": "^5.5.0"
  },
  "optionalDependencies": {
    "vitepress": "^1.6.4",
    "@jis3r/moving-icons": "latest"
  }
}
```

### 8.2 workspace Cargo.toml

```toml
[workspace]
members = [
    "apps/desktop-ui/src-tauri",
    "crates/*",
]
resolver = "2"

[workspace.dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-fs = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-window-state = "2"
tauri-plugin-libsql = "0.1.0"

# HTTP
reqwest = "0.13"

# Serialization
serde = "1"
serde_json = "1"
```

---

## 九、验证点 (Checklist)

### 9.1 编译验证

- [ ] `cargo check` 通过
- [ ] `vite build` 生成 SPA
- [ ] `cargo tauri build` 生成可执行文件 < 15MB

### 9.2 测试验证

- [ ] `cargo test` Rust 单元测试通过
- [ ] `vitest run` Svelte 组件测试通过
- [ ] `playwright test` E2E 测试通过

### 9.3 功能验证

- [ ] SurrealDB 服务端连接
- [ ] Tauri 本地 libsql 存储
- [ ] Google OAuth 登录流程
- [ ] Multi-tenant 数据隔离
- [ ] 移动端响应式布局

---

## 十、关键参考来源

### 目标文件
- goal.md (用户核心需求)
- rust-docs.md (TWiR 108 项目)

### Exa MCP 验证
- lucide-animated / @lucide/svelte Runes 支持 ✅
- lottie-player / @lottiefiles/svelte-lottie-player ✅
- tauri-plugin-libsql v0.1.0 ✅
- reqwest 0.13 rustls ✅

---

*Last updated: 2026-03-28*
*Maintainer: AI Research*