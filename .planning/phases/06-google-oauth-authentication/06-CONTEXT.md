# Phase 06: Google OAuth Authentication - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

用户通过 Google OAuth 登录桌面应用，session 跨 app 重启持久化，token 自动刷新。登录成功后跳转 Counter 页面。已登录用户访问登录页自动重定向。

**不包含：** 云端 session 管理（v2）、email/password auth（v2）、Axum 端 server-side token 验证（v2）。

</domain>

<decisions>
## Implementation Decisions

### OAuth Flow Mechanism
- **D-01:** Deep link callback — 注册自定义 scheme，Google 授权后通过 deep link 回调 app 捕获 auth code
- **D-02:** PKCE flow — 启用 Proof Key for Code Exchange，防止 auth code 劫持
- **D-03:** 系统浏览器打开 Google 同意页面（不是 in-app WebView），符合 OAuth 2.0 for Desktop Apps 最佳实践

### Session Storage
- **D-04:** tauri-plugin-store 持久化 token — access_token, refresh_token, user info, expiry 全部存在 Tauri store 文件中（已注册，即用）
- **D-05:** 不将 token 存入 libsql — token 不应与业务数据混存，不同步到云端

### Token Refresh
- **D-06:** Tauri 后台定时器管理刷新 — 启动时读取 store，计算 token 剩余有效期，在过期前 5 分钟通过 tokio::spawn 自动调 Google token endpoint 刷新
- **D-07:** 刷新失败静默处理 — 清除 store 中的 token，标记 session 过期，用户下次操作时检测并跳转登录页（不弹窗打断）
- **D-08:** reqwest::Client（AppState 已有）用于调用 Google token endpoint

### Login Page UX
- **D-09:** Loading 状态 — 点击 Google 按钮后按钮位置替换为小型 Lottie loading 动画（LottiePlayer 在 package.json 中已预加载，需取消注释激活）
- **D-10:** Error 状态 — OAuth 失败时在按钮下方显示 inline error message
- **D-11:** 已登录重定向 — 页面加载时检测 store 中有效 token，自动跳转 /counter
- **D-12:** 背景装饰 — 登录页有轻微的 Lottie 背景动画（品牌视觉提升）

### Agent's Discretion
- Deep link 自定义 scheme 的具体命名（建议基于 bundle identifier）
- Lottie 动画的具体 JSON 文件（搜索合适的免费 loading + 装饰动画）
- 后台定时器的精确调度逻辑（如首次刷新延迟、重试策略）
- OAuth state 参数生成和验证细节

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 6 — Phase goal, success criteria (5 items), dependencies (Phase 3, Phase 4)
- `.planning/REQUIREMENTS.md` §AUTH-01 through §AUTH-04 — OAuth login, deep link callback, session persistence, auto-refresh

### Architecture & Patterns
- `.planning/research/ARCHITECTURE.md` §Pattern 5 (Tauri Plugin Architecture, lines 383-414) — Plugin lifecycle, custom auth plugin pattern
- `.planning/research/ARCHITECTURE.md` §Integration Patterns (lines 825-838) — Deep link callback via tauri-plugin-deep-link, IPC for auth calls
- `.planning/research/ARCHITECTURE.md` §IPC Recommendation (lines 98-129) — Tauri IPC as primary channel, Axum only for external callbacks

### Workspace Configuration
- `Cargo.toml` — jsonwebtoken = "10.3.0" already declared, tauri-plugin-deep-link = "2" declared
- `apps/desktop-ui/src-tauri/src/lib.rs` — Tauri builder with 4 plugins (shell, dialog, store, libsql); store ready for session

### Existing Code
- `apps/desktop-ui/src/routes/(auth)/login/+page.svelte` — Login page layout with placeholder Google button
- `apps/desktop-ui/package.json` — @lottiefiles/lottie-player (preloaded, needs activation)

### Prior Phase Context
- `.planning/phases/05-database-infrastructure/05-CONTEXT.md` — AppState (SurrealDB + Moka + reqwest), Domain Port traits, tauri-plugin-libsql registered

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **tauri-plugin-store** — 已注册到 Tauri builder，直接用于 token 持久化，无需额外配置
- **tauri-plugin-deep-link** — Cargo.toml 已声明，需注册到 Tauri builder + 配置 capabilities
- **jsonwebtoken** — workspace deps 已声明，用于 JWT 解析和验证
- **reqwest::Client** — AppState 中已有，可直接调 Google token endpoint
- **LottiePlayer** — package.json 中预加载（需取消注释），用于 loading + 装饰动画
- **Button 组件** — $lib/components/ui/Button.svelte，primary variant 已就绪
- **$state rune** — Svelte 5 响应式状态，管理前端 auth 状态

### Established Patterns
- Tauri IPC (`invoke()`) — 前后端通信标准，auth 相关调用走 IPC 不走 HTTP
- Domain Port traits — auth 可以定义 AuthPort trait 在 domain crate
- workspace.dependencies 统一管理 — 新依赖必须走此模式
- Svelte 5 runes — 所有前端状态用 $state/$derived/$effect

### Integration Points
- `apps/desktop-ui/src-tauri/src/lib.rs` — 需要注册 tauri-plugin-deep-link
- `apps/desktop-ui/src/routes/(auth)/login/+page.svelte` — 需要接入真实 OAuth 逻辑
- `apps/desktop-ui/src/routes/(app)/+layout.svelte` — 需要 auth guard 检测
- `apps/desktop-ui/src/lib/ipc/auth.ts` — 需要创建（ARCHITECTURE.md 已规划此路径）
- Tauri capabilities — 需要配置 deep link 权限

</code_context>

<specifics>
## Specific Ideas

- Google Cloud Console 需要创建 OAuth 2.0 Desktop Client，配置 redirect URI 为自定义 scheme
- Login page 的 Lottie 动画建议用品牌色系的几何/粒子动画（保持 Linear/Vercel 美学）
- Loading 动画可以是简单的 pulsing circle 或 dot wave（轻量级，不抢视觉焦点）
- "已登录"检测逻辑放在 +page.svelte 的 onMount 中，避免闪烁
- AUTH-04 (auto-refresh) 的实现可以用 `time::interval` + `tokio::select!` 模式

</specifics>

<deferred>
## Deferred Ideas

### Cloud Session Management (v2)
- v2 考虑引入 Auth Server（Supabase Auth / Clerk / 自建）做跨平台 session 管理
- 跨设备 token 同步应由 Auth Server 管理，不是 DB 复制 token
- libsql + Turso sync 适合同步业务数据，不适合同步 token

### Server-Side Token Validation (v2)
- Axum 中间件验证 JWT token — 当前 v1 所有 auth 逻辑在 Tauri 侧
- Axum 端 OAuth webhook callback — 当前 v1 用 deep link，v2 可加 webhook 作为 fallback

### Reviewed Todos (not folded)
None.

</deferred>

---

*Phase: 06-google-oauth-authentication*
*Context gathered: 2026-03-29*
