<!-- GSD:project-start source:PROJECT.md -->
## Project

**geek taste**

`geek taste` 是一个面向重度 GitHub 观察者与 AI coding 采用者的**跨端技术雷达工作台**。用 TopK 榜单发现值得关注的新趋势，用 Repo 订阅跟踪值得处理的可用更新，用 Agent 资源雷达发现与当前语言/框架相关的 MCP / Skills / Agent 生产力资源。核心输出不是"更多信息"，而是**高信噪比、可行动、低打扰的技术信号**。

**Core Value:** **信号质量高于一切。** 如果 everything else fails，用户仍然能在 30 秒内完成判断：今天有哪些仓库/资源值得看。

### Constraints

- **平台**: v1 正式支持 macOS + Windows 桌面端（Tauri）；Web/Linux/移动仅保留技术准备
- **数据源**: v1 MUST 使用 GitHub 官方 API（Repos/Search/Releases/Tags/Events）；禁止多源爬取
- **前端架构**: SvelteKit 必须使用 static-adapter SPA 模式；Tauri 不支持 server-based frontend
- **数据库**: SQLite 为 v1 主权威数据库；Turso 仅在跨设备同步/云调度时引入
- **API 速率**: 认证 5000 req/hour，Search 30 req/min；scheduler MUST 做端点级限流与退避
- **安全**: GitHub token MUST NOT 明文存入 SQLite；MUST 使用 OS 安全存储
- **同步**: 不得依赖 webhook（需要 owner/admin 权限）；Events API 不是实时接口
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## 核心框架
### Tauri v2（桌面壳）
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `tauri` | 2.6.x | 桌面应用框架 | v2 已于 2024-09 稳定发布，当前 2.6.1；插件生态成熟；跨 macOS + Windows 原生 WebView，包体 ~4MB 远小于 Electron |
| `tauri-build` | 2.x | 构建工具链 | 与 tauri 同版本对齐 |
#### Tauri v2 迁移状态（v1→v2）
- v2 已完全稳定，不再接受 v1 新功能
- v2.6.x 为最新稳定版
- 官方提供 `tauri migrate` 自动迁移工具
- **本项目为 greenfield，无需迁移，直接使用 v2**
- Stronghold 插件将在 v3 被废弃 → **不推荐依赖 Stronghold**
#### Tauri v2 必需插件
| 插件 | 版本 | 用途 |
|------|------|------|
| `tauri-plugin-notification` | 2.x | 桌面通知（digest 推送） |
| `tauri-plugin-shell` | 2.x | 打开外部 URL（仓库链接等） |
| `tauri-plugin-dialog` | 2.x | 文件选择/确认对话框 |
| `tauri-plugin-store` | 2.x | 轻量 KV 持久化（非敏感配置） |
| `tauri-plugin-updater` | 2.x | 自动更新 |
| `tauri-plugin-window-state` | 2.x | 窗口位置/大小记忆 |
### SvelteKit + Svelte 5（前端 UI）
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `svelte` | 5.54.x | UI 编译器 | Runes 响应式系统；编译产物极小（~1.6KB gzipped）；明确优于隐式响应式 |
| `sveltekit` | 2.50.x | 应用框架 | 路由、load 函数、TypeScript 集成 |
| `@sveltejs/adapter-static` | latest | SPA 模式适配器 | Tauri 官方推荐；`fallback: 'index.html'` 实现 SPA 路由 |
| `@sveltejs/vite-plugin-svelte` | latest | Vite 集成 | `vitePreprocess()` 用于 TypeScript/SCSS |
#### SPA 模式配置
#### Svelte 5 Runes 说明
- `$state()` 替代 `let` 声明式响应式
- `$derived()` 替代 `$:` 响应式声明
- `$effect()` 替代 `onMount`/`afterUpdate`
- `$props()` 替代 `export let`
- 响应式可以在 `.svelte.ts` 文件中使用（跨组件状态共享）
- **不需要外部状态管理库**
## 数据层
### SQLite — 主数据库
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `rusqlite` | 0.38.0 | SQLite 封装 | 桌面/嵌入式场景的行业标准；薄封装、零抽象泄漏；完整 SQLite 功能集可直接使用 |
| `rusqlite_migration` | 2.4.1 | Schema 迁移 | 使用 `user_version` 而非额外表，性能最优；内联 SQL 定义简洁；与 rusqlite 版本严格对齐 |
| `serde_rusqlite` | latest | 行→结构体映射 | 减少手动 `.get()` 映射样板代码 |
#### 为什么是 rusqlite 而不是 diesel / sqlx
| 方案 | 推荐？ | 原因 |
|------|--------|------|
| **rusqlite** | ✅ 推荐 | SQLite 专用；同步 API 对桌面应用足够；编译快、抽象薄、调试友好 |
| diesel | ❌ | 编译时 SQL DSL 增加学习成本和编译时间；异步支持是外挂的；对于桌面嵌入式 SQLite 场景过度设计 |
| sqlx | ❌ | 异步开销在桌面应用中无必要；编译时 SQL 检查需要连接数据库；query builder 偏向 Web 后端场景 |
| SeaORM | ❌ | ActiveRecord 模式不适合领域驱动的四层架构；异步 ORM 对桌面应用无收益 |
#### 如果需要异步 SQLite 连接池
| 技术 | 用途 |
|------|------|
| `deadpool-sqlite` | 将 rusqlite 包装为异步连接池，通过 `spawn_blocking` 实现 |
### 为什么不用 Turso / libSQL
- v1 单机模式不需要远程数据库
- Turso 的嵌入式副本（embedded replicas）需要 write-to-cloud 路径，增加架构复杂度
- 当出现跨设备同步需求时（v1.1+）再引入
- **不提前引入，不做过度准备**
## HTTP 与 GitHub API
### HTTP 客户端
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `reqwest` | 0.13.x | HTTP 客户端 | Rust 生态标准；默认 rustls（跨平台一致）；JSON 序列化/反序列化内置；tokio 异步原生 |
| `tokio` | 1.x | 异步运行时 | Rust 异步事实标准；Tauri 内部已使用 tokio；cron 调度器依赖 |
### GitHub API 客户端
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `octocrab` | 0.49.5 | GitHub REST API 客户端 | Rust 生态最成熟的 GitHub 客户端；强类型语义 API + 底层 HTTP 扩展；支持 ETag 条件请求（v0.49.3+）；分页处理内置 |
#### octocrab 关键能力
- **搜索 API**: `octocrab.search().repositories(q).send().await?`
- **Releases**: `octocrab.repos(owner, repo).releases().list().send().await?`
- **Tags**: `octocrab.repos(owner, repo).list_tags().send().await?`
- **Events**: `octocrab.repos(owner, repo).list_events().send().await?`
- **条件请求**: v0.49.3+ 支持 HTTP 缓存与 ETag
- **分页**: `Page<T>` 结构体带 `next()` 链接
- **速率限制**: 内置 `ratelimit` API
#### REST-first 策略确认
## 安全存储（Token）
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `keyring` | 3.6.3（稳定） | OS 安全存储 | 跨平台 Keychain/Credential Manager/Secret Service 集成；Tauri 社区首选方案 |
#### 特性配置
- `apple-native`: macOS Keychain / iOS Keychain
- `windows-native`: Windows Credential Manager
- `sync-secret-service`: Linux D-Bus Secret Service（同步版本，兼容性更好）
#### 为什么不是 Stronghold
| 方案 | 推荐？ | 原因 |
|------|--------|------|
| **keyring** | ✅ 推荐 | 直接使用 OS 原生安全存储；跨平台已验证；Tauri 官方维护者已明确推荐 |
| tauri-plugin-stronghold | ❌ | 官方已声明 v3 将废弃；需要用户输入密码或额外存储加密密钥；社区反馈调试困难 |
## 调度与定时
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `tokio::time` | 1.x（内置） | 简单轮询定时器 | 对固定间隔轮询（如每 12h 同步）足够简单可靠 |
| `tokio-cron-scheduler` | 0.15.1 | Cron 表达式调度 | 300 万下载量；成熟稳定；支持 tokio 异步；适合需要灵活 cron 调度的场景 |
| `chrono` | 0.4.x | 时间处理 | Rust 时间标准库；tokio-cron-scheduler 和 octocrab 均依赖 |
#### 调度策略建议
- **简单轮询**（每 N 小时同步订阅）：用 `tokio::time::interval`
- **复杂调度**（按 cron 表达式触发 TopK 快照/摘要生成）：用 `tokio-cron-scheduler`
- v1 大多数场景用 `tokio::time::interval` 就够了，`tokio-cron-scheduler` 作为增强引入
## 错误处理与序列化
| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `serde` | 1.x | 序列化/反序列化 | Rust 生态标准；JSON/TOML/结构体转换；octocrab/reqwest/rusqlite 均依赖 |
| `serde_json` | 1.x | JSON 处理 | GitHub API 响应解析；IPC 数据传输 |
| `anyhow` | 1.x | 应用层错误 | Application Layer 的错误包装；简化错误传播 |
| `thiserror` | 2.x | 领域层错误 | Domain Layer 的类型化错误定义；derive 宏简化 |
| `tracing` | 0.1.x | 结构化日志 | Rust 异步日志标准；替代 log crate；支持 span 和 structured fields |
| `tracing-subscriber` | 0.3.x | 日志输出配置 | 控制台/文件输出 |
#### 错误分层策略
## 完整依赖安装
### Cargo.toml 核心依赖
# Tauri
# Tauri 插件
# 异步运行时
# HTTP
# GitHub API
# SQLite
# 安全存储
# 序列化
# 错误处理
# 时间
# 调度
# 日志
### package.json 前端依赖
## 不使用的方案及原因
| 被排除方案 | 原因 |
|-----------|------|
| **Electron** | 包体巨大（150MB+ vs Tauri ~4MB）；Node.js 运行时冗余 |
| **Deno/Bun** | Tauri 绑定 Node.js 生态；切换运行时无收益且增加不确定性 |
| **GraphQL（v1）** | REST-first 策略已确定；批量优化留待 v1.1+ |
| **SurrealDB** | 域模型不需图数据库；SQLite 确定性和成熟度更重要 |
| **Turso（v1）** | 单机不需要远程数据库；过度准备 |
| **diesel / sqlx / SeaORM** | 桌面 SQLite 嵌入式场景中 rusqlite 是最优解 |
| **Stronghold** | 官方已宣布 v3 废弃；keyring 直接使用 OS 安全存储更可靠 |
| **GraphQL client（async-graphql 等）** | v1 不使用 GraphQL |
| **Zustand/Pinia/Svelte Stores** | Svelte 5 Runes 提供内置跨组件状态，无需外部库 |
| **Axum（v1）** | v1 不需要 HTTP 服务器；预留至 v1.1 云同步 |
## Cargo Workspace 结构（对应架构规范）
## 兼容性注意事项
| 关注点 | 状态 | 说明 |
|--------|------|------|
| Tauri v2 + SvelteKit 2 | ✅ 官方支持 | Tauri 文档有专门集成指南 |
| rusqlite `bundled` 特性 | ✅ 推荐启用 | 内置 SQLite 编译，避免系统库版本不一致 |
| reqwest 默认使用 rustls | ✅ 跨平台一致 | 无需系统 OpenSSL 依赖 |
| keyring 跨平台 | ✅ macOS/Windows/Linux | 需分别启用 `apple-native`/`windows-native`/`sync-secret-service` |
| Svelte 5 Runes + Tauri IPC | ✅ 无冲突 | Runes 是编译时特性，不影响运行时 IPC |
| tokio 版本对齐 | ⚠️ 需注意 | 确保所有依赖使用 tokio 1.x；octocrab/reqwest/tokio-cron-scheduler 均使用 tokio 1.x |
## 置信度评估
| 领域 | 置信度 | 原因 |
|------|--------|------|
| Tauri v2 | HIGH | 官方文档 + Context7 + crates.io 验证；v2.6.x 为当前稳定版 |
| SvelteKit + Svelte 5 | HIGH | Tauri 官方集成指南验证；Svelte 5.54.0 GA 已发布超过 1 年 |
| rusqlite | HIGH | 多篇 2026 年对比文章一致推荐；crates.io 版本确认 |
| octocrab | HIGH | GitHub 仓库活跃；v0.49.5 已支持 HTTP 缓存；1.3k stars |
| keyring | HIGH | crates.io 活跃更新；keyring-rs GitHub 有 Tauri v2 示例项目 |
| reqwest | HIGH | 4 亿下载量；v0.13.2 为最新稳定版 |
| 调度方案 | MEDIUM | tokio-cron-scheduler 成熟但无大规模 Tauri 桌面应用案例验证；`tokio::time::interval` 更简单稳妥 |
| Stronghold 不推荐 | HIGH | Tauri 官方维护者公开声明将废弃；社区普遍反馈不佳 |
## 来源
- [Tauri v2 官方文档](https://v2.tauri.app/) — HIGH
- [Tauri + SvelteKit 集成指南](https://v2.tauri.app/start/frontend/sveltekit/) — HIGH
- [rusqlite crates.io](https://crates.io/crates/rusqlite) — HIGH
- [octocrab CHANGELOG](https://github.com/XAMPPRocky/octocrab/blob/master/CHANGELOG.md) — HIGH
- [reqwest crates.io](https://crates.io/crates/reqwest) — HIGH
- [keyring-rs GitHub](https://github.com/open-source-cooperative/keyring-rs) — HIGH
- [Rust ORMs 2026 对比](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3) — HIGH
- [rusqlite_migration](https://crates.io/crates/rusqlite_migration) — HIGH
- [Reddit: Stronghold 不推荐](https://www.reddit.com/r/rust/comments/1qgb3rq/) — HIGH
- [keyring-demo Tauri v2 项目](https://github.com/open-source-cooperative/keyring-demo) — HIGH
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
