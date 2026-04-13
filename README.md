# tauri-sveltekit-axum-moon-template

> **状态**：平台模型骨架已落地，业务层仍在迁移收敛中。
> **目标**：Tauri 2 + SvelteKit + Axum 跨端 monorepo，支持单 VPS → K3s → 微服务拓扑无缝切换。

---

## 快速导航

| 你要找什么 | 去哪里 |
|-----------|-------|
| **当前实际状态** | [`docs/CURRENT-STATE.md`](docs/CURRENT-STATE.md) ← 单一真相源 |
| **目标架构与硬约束** | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| **差距分析与优先级** | [`docs/architecture-gap-priority-plan.md`](docs/architecture-gap-priority-plan.md) |
| **目录布局说明** | [`docs/architecture/repo-layout.md`](docs/architecture/repo-layout.md) |
| **重构进度** | [`.refactoring-state.yaml`](.refactoring-state.yaml) |
| **开发命令入口** | `just --list` |

---

## 目录总览

```
├── agent/          # Agent 约束、模板、checklist
├── apps/           # 前端与客户端 (web, desktop, browser-extension)
├── docs/           # ADR、架构图、运维文档、生成产物
├── fixtures/       # 测试种子数据 (tenants, sync-scenarios, seeds)
├── infra/          # 基础设施声明与交付 (docker, k3s, gitops, security)
├── justfiles/      # Just 任务模块
├── ops/            # 运维执行 (migrations, runbooks, benchmark)
├── packages/       # 共享抽象、适配器、运行时、契约
├── platform/       # 平台模型层（真理源：services/deployables/resources/topologies）
├── scripts/        # 辅助脚本 (typegen, boundary-check, doctor)
├── servers/        # 同步请求入口 (api, bff/*, gateway, indexer, realtime)
├── services/       # 纯业务能力库 (counter, user, tenant, agent, chat, auth, settings, admin, event-bus)
├── tools/          # 本地辅助工具 (web3)
├── verification/   # 跨模块验证 (contract, e2e, resilience, golden, topology, performance)
└── workers/        # 异步执行层 (indexer, outbox-relay, projector, scheduler, sync-reconciler)
```

---

## 当前完成度

| 层级 | 完成度 | 说明 |
|-----|-------|------|
| `platform/` | ✅ 高 | schema/model/generators/validators/catalog 已完整 |
| `workers/` | ✅ 高 | 5 个独立 worker 已建立，含 runtime ports 集成 |
| `services/` | ⚠️ 中 | counter/user/auth/settings 较完整；tenant/agent 已迁移但 README 未更新；chat/admin 已实现但文档缺失 |
| `servers/` | ⚠️ 中 | api + bff/* 有真实实现；gateway/realtime 仍为占位；indexer 与 workers 职责冲突待清理 |
| `packages/` | ⚠️ 中 | kernel/platform/runtime/contracts 已就位；core/features/shared 为过渡层；sdk/ 为空占位符 |
| `apps/` | ⚠️ 中 | web/desktop/extension 已建立；前端使用 app-local generated client 而非 packages/sdk |
| `infra/` | ⚠️ 中 | docker compose/k3s base/gitops/sops 已建立；rendered 产物待生成 |
| `verification/` | ⚠️ 中 | contract/resilience/golden 有基础；e2e/performance/topology 覆盖较弱 |
| `docs/` | ✅ 高 | 8 个 ADR + C4 架构图 + 运维文档 + 合同文档已完整 |

---

## 开发入口

```bash
# 查看所有可用命令
just --list

# 工具链检查
mise doctor

# Rust 质量门禁
just quality

# 平台模型验证
just validate-platform
just validate-deps

# 开发启动
just dev-api          # 启动 API server
just dev-web          # 启动 Web 前端
just dev-desktop      # 启动 Desktop 客户端
```

---

## 核心架构原则

1. **平台模型优先** — `platform/model/*` 是真理源，一切由其生成
2. **契约先于实现** — `packages/contracts/*` 先改，再改实现
3. **Services 是库，不是进程** — 可被 servers/workers 同时复用
4. **Workers 一等公民** — 所有异步执行单元在 `workers/`
5. **Runtime Ports 高于中间件** — 业务只依赖抽象，adapter 落地
6. **生成物禁止手改** — sdk/rendered/catalog 必须可删可再生

---

## 重要提醒

**本文档描述的是当前实际状态，不是最终目标架构。**
- 目标架构详见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- 当前与目标的差距详见 [`docs/architecture-gap-priority-plan.md`](docs/architecture-gap-priority-plan.md)
- Agent 开发前**必须**先读 `docs/CURRENT-STATE.md`，不得仅凭 ARCHITECTURE.md 推断现状
