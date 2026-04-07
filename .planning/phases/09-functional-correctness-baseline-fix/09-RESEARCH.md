# Phase 9: 功能正确性基线修复 — Research

**Date:** 2026-04-06  
**Scope:** AUTH-02, AUTH-03, COUNTER-02, AGENT-02, AGENT-03, AGENT-04

## 结论摘要

Phase 9 不需要引入新依赖，优先沿用既有模式完成正确性修复：

1. **认证退出**：在 Settings 页面提供可见退出入口，执行路径采用「服务端/会话失效 + 本地清理兜底」，保证 desktop/browser 都不会复用旧凭据。
2. **计数器一致性**：统一为「进入即拉取 + 以后端返回值为准」，失败必须可见并保持最后成功值。
3. **Agent 新会话**：点击 New Chat 即切换到新会话并清空消息输入上下文，同时保持 API key/base URL/model。
4. **连接测试**：在 Settings 页面增加 Test Connection，输出 API key/Base URL/Model 三项独立 pass/fail + 下一步建议。

---

## 现有代码可复用点

- `apps/client/web/app/src/lib/stores/auth.svelte.ts`：已有 `signOut/checkSession/markExpired`，可扩展为双端失效流程。
- `apps/client/web/app/src/lib/ipc/auth.ts`：已有 Tauri command bridge，可新增 `logout()` IPC。
- `packages/adapters/hosts/tauri/src/commands/auth.rs`：已有 `AuthService::logout` 能力，可直接暴露 Tauri command。
- `apps/client/web/app/src/routes/(app)/counter/+page.svelte`：已有增减重置命令调用链，需补齐错误可见性与一致性规则。
- `apps/client/web/app/src/routes/(app)/agent/+page.svelte`：已有 New Chat + settings 读取路径，需强化新线程语义与设置读取失败提示。
- `apps/client/web/app/src/routes/(app)/settings/+page.svelte`：已有 settings 保存 UI，可加 logout 与 Test Connection 入口。

---

## 实施策略（按需求映射）

### AUTH-02 / AUTH-03

- 在 `runtime_tauri` 新增 `logout` command 并注册到 `src-tauri/lib.rs`。
- 前端 `signOut()` 顺序：
  1) 调用 `logout`（若失败记录错误但不中断）
  2) `clearAuthStore()` 本地清理（必须执行）
  3) 重置 auth state
  4) 跳转公开页面（优先 referrer 的公开路由，否则 `/login`）

### COUNTER-02

- `loadValue()` 失败时显示页面错误条并保持 `count=0`。
- `increment/decrement/reset` 在成功时只用 command 返回值更新 UI，失败时保留当前值并展示错误条。
- 添加刷新后一致性回归用例（修改后 reload，显示值与持久值一致）。

### AGENT-02 / AGENT-03

- `createConversation()` 成功后立刻 `activeConversation=conv.id`，`messages=[]`，输入区可立即发送。
- 设置保持锁定三项：`api_key/base_url/model`。
- `loadSettings()` catch 分支必须设置可操作提示（例如“读取 settings.json 失败，请前往 Settings 重新保存配置”），允许默认值继续流程。

### AGENT-04

- 在 Settings 页面 Save 按钮旁新增 `Test Connection`。
- 测试结果结构化为三项：
  - API key（格式/缺失）
  - Base URL（URL 可达性/响应）
  - Model（指定模型是否可用）
- 失败后不回滚输入值，仅给出下一步建议并允许重试。

---

## 风险与防护

1. **退出只做本地清理导致服务器状态残留**  
   防护：command 暴露 `logout`，前端固定先远端后本地，远端失败不阻断本地清理。

2. **计数器异常被吞掉，造成“看似成功”**  
   防护：禁止空 catch；统一页面内 error banner。

3. **连接测试只给总结果不可诊断**  
   防护：三维结果对象 + 每项 remediation 文案。

4. **在浏览器路径硬依赖 Tauri API**  
   防护：沿用 dual-path（Tauri invoke / browser HTTP fallback）。

---

## Verification Architecture

### 快速反馈（任务级）

- `bun run --cwd apps/client/web/app test:unit -- tests/component/auth.test.ts`
- `bun run --cwd apps/client/web/app test:unit -- tests/component/counter.test.ts`
- `bun run --cwd apps/client/web/app test:unit -- tests/component/agent-phase9.test.ts`

### 计划级回归

- `bun run --cwd apps/client/web/app test:unit`
- `bun run --cwd apps/client/web/app test:e2e --grep "(logout|counter|agent|settings|connection)"`

### 阶段门禁

- `bun run --cwd apps/client/web/app check`
- `bun run --cwd apps/client/web/app lint`
- `bun run --cwd apps/client/web/app test:unit`
- `bun run --cwd apps/client/web/app test:e2e --grep "(login|counter|agent)"`

---

## 不建议手搓（沿用现有）

- 不新增状态管理库（继续 Svelte 5 runes + 现有 store）
- 不新增 HTTP client 库（继续 `fetch` + existing IPC wrapper）
- 不新增测试框架（继续 Vitest + Playwright）
