# Phase 8: Agent 双路径 + Prompts + Phase 5 验证 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-02
**Phase:** 08-agent-dualpath-prompts
**Areas discussed:** IPC 双路径实现, Phase 5 验证闭环, Desktop Mode E2E 验证

---

## IPC 双路径实现

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri 2 Channel API | 前端传 Channel 给 invoke，Rust 端通过 channel.send() 逐 token 推送。类型安全，官方推荐 | ✓ |
| Tauri Event emit/listen | Rust 端 app.emit() 发事件，前端 listen() 接收。实现简单但事件是全局的，需要处理多对话并发冲突 | |

**User's choice:** Tauri 2 Channel API (Recommended)
**Notes:** 用户还要求调研 Rust LLM 框架，最终选择 async-openai crate（698K 下载/月，最成熟，原生支持 streaming）。IPC 模块放在 lib/ipc/agent.ts 与 auth.ts 并列。

## Phase 5 验证闭环

| Option | Description | Selected |
|--------|-------------|----------|
| 只验证已有 assets | 验证 playbooks 和 rubrics 的文件存在性、格式正确性、内容可执行性。Skills 和 Prompts 标记为 deferred | ✓ |
| 完整验证所有 4 类 | 要求 skills、prompts、playbooks、rubrics 全部就位才算通过 | |

**User's choice:** 只验证已有 assets (Recommended)
**Notes:** 与 Phase 5 CONTEXT.md D-01/D-03 一致（skills 和 prompts 被跳过）

## Desktop Mode E2E 验证

| Option | Description | Selected |
|--------|-------------|----------|
| 验证 IPC 通路即可 | 验证 Tauri invoke → Rust command → agent service 能通，不要求完整 OpenAI 调用 | |
| 完整对话验证 | 需要真实 API key 验证完整对话流程，包括 streaming 显示 | ✓ |

**User's choice:** 完整对话验证
**Notes:** 需要配置 API key 环境，验证完整链路

## Prompts 模板补全

**User's decision:** 用户亲自写，本阶段跳过
**Notes:** .agents/prompts/ 目录已存在（只有 .gitkeep），用户后续自行创建 add-feature, add-host, refactor-boundary 等内容

---

## the agent's Discretion

- Tauri Channel API 的具体类型签名和错误处理
- `agent_chat` command 的参数设计
- async-openai 的 client 初始化和连接复用策略
- Phase 5 VERIFICATION.md 的具体验证步骤和通过标准
- E2E 测试的具体实现方式

## Deferred Ideas

- Prompts 具体内容（add-feature, add-host, refactor-boundary）— 用户亲自写
- Skills 具体内容 — Phase 5 决定跳过
- Agent tool calling 的写操作 — 未来版本
- Agent 对话的多模态支持 — 未来
