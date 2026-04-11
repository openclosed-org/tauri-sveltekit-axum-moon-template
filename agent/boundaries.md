# Agent Boundaries — 目录修改边界定义

> **格式**: Markdown 表格 + 示例
> **验证**: `just quality boundary` 必须校验依赖方向

## 目录修改权限矩阵

| 目录 | 可修改 | 禁止修改 | 跨目录限制 | 示例 |
|------|--------|----------|-----------|------|
| `agent/` | ✅ 约束定义、模板、检查清单 | ❌ 业务逻辑代码 | 不可引用 services/, apps/ | 新增 `prompts/add-module.md` ✅ |
| `apps/` | ✅ UI 组件、路由、前端状态 | ❌ 业务逻辑、服务端调用 | 仅引用 packages/contracts/sdk, packages/ui | `apps/web/src/routes/+page.svelte` ✅ |
| `apps/bff/` | ✅ Handler、Adapter、中间件 | ❌ 领域逻辑、数据访问实现 | 仅引用 services/* (via trait), contracts | `apps/bff/web-bff/src/handlers/` ✅ |
| `services/` | ✅ 领域逻辑、用例、端口 | ❌ 其他 services/, 基础设施实现 | 仅引用 packages/core, packages/contracts | `services/user/src/domain/` ✅ |
| `packages/core/` | ✅ trait 定义、基础类型 | ❌ 业务逻辑、具体实现 | 无外部依赖（除 serde/thiserror 等） | `packages/core/kernel/src/tenant_id.rs` ✅ |
| `packages/contracts/` | ✅ DTO、OpenAPI、事件契约 | ❌ 实现代码 | 仅引用 packages/core | `packages/contracts/http/` ✅ |
| `packages/adapters/` | ✅ 基础设施实现 | ❌ 业务逻辑 | 仅引用 packages/core + external crates | `packages/adapters/turso/` ✅ |
| `packages/sdk/` | ❌ 自动生成，禁止手动修改 | ❌ 任何手动编辑 | 由 contracts/ 生成 | `just gen-frontend-sdk` ✅ |
| `packages/ui/` | ✅ 共享 UI 组件 | ❌ 业务逻辑、端特定代码 | 无业务依赖 | `packages/ui/src/components/Button.svelte` ✅ |
| `infra/` | ✅ 基础设施声明 | ❌ 业务代码引用 | 通过 config loader 加载 | `infra/docker/compose/app.yaml` ✅ |
| `ops/` | ✅ 运维脚本、迁移、文档 | ❌ 业务代码依赖 | 通过 API/配置调用 | `ops/migrations/runner/` ✅ |
| `fixtures/` | ✅ 测试数据、种子 | ❌ 生产数据 | 仅测试使用 | `fixtures/tenants/default.json` ✅ |
| `docs/` | ✅ 架构决策、契约文档 | ❌ 代码 | 无 | `docs/adr/001-*.md` ✅ |

## 依赖方向（不可违反）

```
contracts/     ← 所有共享类型的单一真理源
features/      ← 定义 trait + 类型，不得包含实现，不得依赖 usecases
usecases/      ← 实现 features 定义的 trait
adapters/      ← 外部世界翻译层，不得承载业务逻辑
apps/ / bff/   ← 组合层，不得包含业务逻辑
```

## 验证命令

```bash
just quality boundary    # 依赖方向必须零违规
cargo build -p <service> # 服务必须可独立构建
just gen-openapi && git diff --exit-code  # OpenAPI 必须稳定
```
