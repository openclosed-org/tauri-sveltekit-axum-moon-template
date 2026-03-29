# Phase 06: Google OAuth Authentication - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 06-google-oauth-authentication
**Areas discussed:** OAuth Flow Mechanism, Session Storage Strategy, Token Refresh Architecture, Login Page UX States

---

## OAuth Flow Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Deep link 回调 | tauri-plugin-deep-link 注册自定义 scheme，系统浏览器 → Google 同意 → deep link 回调 app。零服务器依赖。 | ✓ |
| Axum webhook 中转 | Axum 暴露 /webhook/oauth 接收 Google 回调，再通过 Tauri 命令通知前端。需部署。 | |
| 两者都支持 | Deep link 为主，Axum webhook 为 fallback。更灵活但复杂。 | |

**User's choice:** Deep link 回调 (Recommended)

**Follow-up:** Scheme 命名 + PKCE

| Option | Description | Selected |
|--------|-------------|----------|
| tauri://auth-callback + PKCE | 基于 bundle identifier 的 scheme，启用 PKCE 防劫持。桌面 OAuth 最佳实践。 | ✓ |
| localhost callback + no PKCE | 本地 HTTP 回调，不需要 deep link 插件。更简单但不够安全。 | |

**User's choice:** tauri://auth-callback + PKCE

**Notes:** 用户确认 ARCHITECTURE.md 推荐路径。

---

## Session Storage Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| tauri-plugin-store | token 存 Tauri store 文件，已注册即用。简单安全。 | ✓ |
| libsql 本地 DB | token 存 libsql 表，结构化管理。 | |
| tauri-plugin-store + Moka cache | store 持久化 + Moka 运行时缓存。两层。 | |

**User's choice:** 提出了关于跨平台云端 session 的疑问 — 是否 libsql/Turso 更符合企业级规范？

**我的分析：**
- Token 存储 ≠ 业务数据存储，token 应存安全存储
- 跨平台同步应通过 Auth Server 管理 session，不是 DB 复制 token
- libsql + Turso sync 适合业务数据，不适合 token 同步
- 建议 v1 用 tauri-plugin-store，v2 考虑 Auth Server

**Follow-up 确认：**

| Option | Description | Selected |
|--------|-------------|----------|
| tauri-plugin-store, v2 Auth Server | v1 本地存储，v2 引入 Auth Server 做跨平台 session。Deferred idea 记录。 | ✓ |
| 直接上 libsql 存 token | token 存 libsql，为 Turso sync 准备。有安全考量。 | |

**User's choice:** tauri-plugin-store, v2 考虑 Auth Server (Recommended)

---

## Token Refresh Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri 后台定时器 | tokio::spawn 定时刷新，独立于 Axum，桌面 app 不需服务器在线。 | ✓ |
| Axum 中间件拦截 | 每次 API 请求检查有效期，过期自动 refresh。需 Axum 运行。 | |
| 前端 Svelte 定时器 | $effect + setInterval 检查，invoke Tauri 命令刷新。逻辑分散。 | |

**User's choice:** Tauri 后台定时器 (Recommended)

**Follow-up:** 刷新失败行为

| Option | Description | Selected |
|--------|-------------|----------|
| 静默清除 + 下次操作触发登录 | 清 token，标记过期，下次操作跳转登录。不弹窗打断。 | ✓ |
| 立即弹窗通知重新登录 | 刷新失败立即弹 Dialog。更明确但可能打扰。 | |

**User's choice:** 静默清除 + 下次操作触发登录 (Recommended)

---

## Login Page UX States

| Option | Description | Selected |
|--------|-------------|----------|
| Loading + Error + 已登录重定向 | 三态：loading spinner、inline error、token 有效自动跳转。 | ✓ |
| 只有 Loading 和已登录重定向 | 简化，error 交给 toast。 | |
| 更丰富的登录页 | Logo 动画、按钮 hover、渐进式 loading。 | |

**User's choice:** 选项 1 + 选项 3（简单 Lottie 动画）—— "我希望是有一个简单的 lottie 动画"

**Follow-up:** Lottie 动画位置

| Option | Description | Selected |
|--------|-------------|----------|
| Loading spinner 替代 | OAuth 进行中时按钮位置替换为小型 Lottie loading。 | ✓ |
| 登录页背景装饰动画 | Logo 上方或背景轻微动画。 | ✓ |
| 两者都有 | 背景装饰 + loading 时按钮动画。 | ✓ |

**User's choice:** 两者都有

---

## Agent's Discretion

- Deep link scheme 具体命名 — agent 决定（基于 bundle identifier）
- Lottie 动画 JSON 文件选择 — agent 搜索合适免费动画
- 后台定时器调度逻辑 — agent 决定精确实现
- OAuth state 参数生成 — agent 决定

---

## Deferred Ideas

1. **Cloud Session Management (v2)** — Auth Server (Supabase/Clerk/自建) 管理跨平台 session
2. **Server-Side Token Validation (v2)** — Axum 中间件验证 JWT
3. **Axum OAuth Webhook (v2)** — Axum 端接收 OAuth callback 作为 deep link 的 fallback
