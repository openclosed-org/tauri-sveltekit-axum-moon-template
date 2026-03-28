# 技术选型文档 - Template Boilerplate

> 基于 goal.md 需求 + rust-docs.md + 版本验证 (2026-03-28)
> 核心目标: 跑通 3 个页面(登录、计数器、admin)的开发基建

---

## 一、项目定位与目标

### 1.1 核心需求 (goal.md)

- **页面**: 登录页 + 计数器页 + Admin 管理端 (共 3 页)
- **认证**: Google 一键登录
- **多租户**: tenant_id 数据隔离
- **基建**: MCP 代码索引 + WebSearch + 各领域 Skills
- **平台**: Desktop + Mobile Web (mobile-first, 响应式)
- **测试**: 编译成功 + 测试通过

### 1.2 技术框架 (版本已验证)

| 层级       | 选型          | 版本   | 验证日期   |
| ---------- | ------------- | ------ | ---------- |
| 桌面 Shell | **Tauri**     | 2.10.3 | 2026-03-28 |
| 前端框架   | **SvelteKit** | 2.55.0 | 2026-03-28 |
| 前端语言   | **Svelte**    | 5.55.0 | 2026-03-28 |
| 后端 HTTP  | **Axum**      | 0.8.8  | 2026-03-28 |
| 构建工具   | **moon**      | 2.x    | 已集成     |
| 包管理器   | **bun**       | 1.3.x  | 已配置     |

---

## 二、数据库策略 (核心)

### 2.1 双数据库架构

用户明确要求:

- **服务端**: SurrealDB (独立部署)
- **本 App**: libsql/turso (嵌入式本地存储)
- **云端同步**: turso cloud (可选)

### 2.2 策略模式 (Adapter Pattern)

```rust
pub trait DatabaseAdapter {
    async fn query(&self, sql: &str) -> Result<Vec<Row>>;
    async fn execute(&self, sql: &str) -> Result<()>;
    async fn sync(&self) -> Result<()>;
}

pub enum DatabaseBackend {
    SurrealDB(SurrealDBAdapter),
    LibSQL(LibSQLAdapter),
    Turso(TursoCloudAdapter),
}
```

### 2.3 依赖配置 (版本已验证)

| 用途     | Crate                 | 版本  | 验证日期   |
| -------- | --------------------- | ----- | ---------- |
| 本地存储 | `tauri-plugin-libsql` | 0.1.0 | 2026-02-24 |
| 云端同步 | `libsql`              | 0.4.x | 2026-03-28 |
| 服务端   | `surrealdb`           | 3.0.5 | 2026-03-28 |

---

## 三、前端技术栈 (版本已验证)

### 3.1 核心依赖

```json
{
  "scripts": {
    "dev": "concurrently \"vite dev\" \"cargo tauri dev\"",
    "build": "vite build && cargo tauri build",
    "test:unit": "vitest run",
    "test:mobile": "maestro test tests/flow/",
    "test:e2e": "playwright test",
    "lint": "biome check ."
  },
  "dependencies": {
    "bits-ui": "2.16.4",
    "tailwindcss": "4.2.2",
    "@lucide/svelte": "1.7.0",
    "@pqoqubbw/icons": "latest",
    "@lottiefiles/svelte-lottie-player": "0.3.1"
  },
  "devDependencies": {
    "@sveltejs/kit": "2.55.0",
    "svelte": "5.55.0",
    "@sveltejs/adapter-static": "3.0.0",
    "vite": "8.0.3",
    "vitest": "4.1.2",
    "vitest-browser-svelte": "latest",
    "@playwright/test": "1.58.2",
    "maestro": "latest",
    "@biomejs/biome": "1.9.4",
    "typescript": "5.5.0"
  },
  "optionalDependencies": {
    "vitepress": "2.0.0-alpha.17"
  }
}
```

### 3.2 图标方案

用户指定: **pqoqubbw/icons** (7.3K ⭐)

| 方案     | 包                                  | 动画    |
| -------- | ----------------------------------- | ------- |
| 主选静态 | `@lucide/svelte`                    | 无      |
| 主选动画 | `@pqoqubbw/icons`                   | ✅ 379+ |
| Lottie   | `@lottiefiles/svelte-lottie-player` | ✅      |

### 3.3 文档站点

**用户指定**: VitePress (构建后纯静态 HTML，不占云服务器资源)

---

## 四、移动端测试框架 (用户指定 Maestro)

| 框架        | 版本   | 适用场景              |
| ----------- | ------ | --------------------- |
| **Maestro** | 2.0.0  | Mobile E2E (用户指定) |
| Playwright  | 1.58.2 | Web E2E + Mobile      |

---

## 五、MCP / Skills 配置

### 5.1 MCP Servers

| MCP               | 用途           | 状态      |
| ----------------- | -------------- | --------- |
| Exa Search        | 实时 web 搜索  | ✅ 已配   |
| Chrome DevTools   | 浏览器自动化   | ✅ 已配   |
| indxr             | 代码库语义索引 | 🔲 需配置 |
| rust-analyzer-mcp | Rust 代码诊断  | 🔲 需配置 |

### 5.2 Skills

| 领域     | Skill              |
| -------- | ------------------ |
| Rust     | rust-skills        |
| Svelte   | svelte-code-writer |
| Frontend | frontend-patterns  |
| Backend  | backend-patterns   |
| Test     | e2e-testing        |
| Docker   | docker-patterns    |
| API      | api-design         |
| Security | security-review    |

---

## 六、Monorepo 目录架构

```
tauri-sveltekit-axum-moon-template/
├── apps/
│   ├── desktop-ui/           # Tauri App
│   └── docs/                 # VitePress 文档
├── crates/
│   ├── domain/
│   ├── application/
│   ├── adapters/             # 数据库适配层
│   ├── runtime_tauri/
│   ├── runtime_server/
│   └── shared_contracts/
├── packages/
├── docker/
└── moon.yml
```

---

## 七、Cargo 依赖 (版本已验证)

### 7.1 服务端

```toml
[dependencies]
tauri = "2.10.3"
axum = "0.8.8"
tokio = { version = "1.50.0", features = ["full"] }
reqwest = { version = "0.13.2", features = ["rustls"] }
surrealdb = "3.0.5"
serde = "1.0.228"
serde_json = "1"
jsonwebtoken = "10.3.0"
uuid = { version = "1.23.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[workspace.dependencies]
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-fs = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-window-state = "2"
tauri-plugin-libsql = "0.1.0"
```

---

## 八、Release / Changelog / Git Hooks 工具链

### 8.1 Changelog 生成

| 工具 | 版本 | 类型 | 适用场景 |
|------|------|------|----------|
| **git-cliff** | 2.12.0 | Rust | ⭐ 主推 - 支持 Conventional Commits |
| conventional-changelog | - | Node.js | npm 生态 |

```toml
# Cargo.toml
[dev-dependencies]
git-cliff = "2.12.0"

# 生成 changelog
git-cliff --output CHANGELOG.md
```

### 8.2 Release 自动化

| 工具 | 版本 | 类型 | 适用场景 |
|------|------|------|----------|
| **release-plz** | 0.3.157 | Rust | ⭐ 主推 - 自动版本 + Changelog |
| changesets | - | Node.js | npm/pnpm 生态 |

```toml
# Cargo.toml
[dev-dependencies]
release-plz = "0.3.157"
```

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    branches: [main]
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: release-plz/release-pr@v0.3
```

### 8.3 Git Hooks

由于是 monorepo (Node.js + Rust 混合)，需要兼容方案:

| 工具 | 版本 | 类型 | 适用场景 |
|------|------|------|----------|
| **husky** | 9.1.7 | Node.js | ⭐ npm/bun 脚本 |
| **cargo-husky** | 1.5.0 | Rust | Rust 代码检查 |
| **pre-commit** | - | Python | 跨语言通用 |
| **fasthooks** | 0.1.0 | Rust | 新兴，声称 27x 更快 |

**推荐方案 (moon repo)**:

```yaml
# .husky/pre-commit (husky)
#!/bin/sh
npm run lint:front && npm run lint:back

# 或使用 pre-commit (全局)
pip install pre-commit

# .pre-commit.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-check
        name: cargo check
        entry: cargo check
        language: system
        pass_filenames: false
        stages: [pre-commit]
```

### 8.4 完整配置示例

```yaml
# moon.yml
tasks:
  lint:
    command: 'biome check .'
    inputs:
      - '@globs(*)'

  # Release流程
  release:dry-run:
    command: 'release-plz update --dry-run'

  release:pr:
    command: 'release-plz release-pr'

  changelog:
    command: 'git-cliff -o CHANGELOG.md'
    inputs:
      - '@gitcommits'
```

---

## 九、版本检查汇总

### 已验证版本 (2026-03-28)

**NPM**:

- vitepress: 2.0.0-alpha.17 ✅
- @sveltejs/kit: 2.55.0 ✅
- svelte: 5.55.0 ✅
- tailwindcss: 4.2.2 ✅
- vite: 8.0.3 ✅
- vitest: 4.1.2 ✅
- @playwright/test: 1.58.2 ✅

**Rust**:

- tauri: 2.10.3 ✅
- surrealdb: 3.0.5 ✅
- reqwest: 0.13.2 ✅
- axum: 0.8.8 ✅
- tokio: 1.50.0 ✅
- serde: 1.0.228 ✅

---

_Last updated: 2026-03-28_
_Version Checked: 2026-03-28_
