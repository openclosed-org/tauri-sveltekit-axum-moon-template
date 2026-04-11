# Agent Harness — AI Agent 协作约束

> **设计原则**: Agent 只能在此定义的边界内生成代码。
> 业务逻辑代码**禁止**出现在此目录。

## 核心约束

1. **Agent 首要阅读入口**: 新 Agent 会话必须首先加载此文件
2. **模块边界**: 参见 `codemap.yml` — 每个模块的允许/禁止依赖、允许引入的 external crates、必须遵循的代码模式
3. **目录修改边界**: 参见 `boundaries.md` — 明确哪些目录可改、哪些禁止跨改、哪些只读
4. **约束规则**: 参见 `constraints/` — 依赖白/黑名单、禁止代码模式、契约变更流程、存储策略约束
5. **生成模板**: 参见 `templates/` — 新增模块、BFF 端点的代码生成模板
6. **检查清单**: 参见 `checklists/` — 关键操作的验证清单

## 快速规则

| 操作 | 必须参考 |
|------|----------|
| 新增模块 | `prompts/add-module.md` + `templates/module/` |
| 新增 API 端点 | `prompts/add-endpoint.md` + `templates/bff-endpoint/` |
| 新增同步策略 | `prompts/add-sync-strategy.md` |
| 物理拆分服务 | `prompts/split-service.md` |
| 契约变更 | `checklists/schema-change.md` |
| 数据库迁移 | `checklists/migration.md` |
| 同步冲突处理 | `checklists/sync-conflict.md` |
| 发版 | `checklists/release.md` |

## 验证

- CI 必须校验 `codemap.yml` 语法 + 语义
- `just quality boundary` 必须校验依赖方向
- Agent 生成代码必须在约束边界内，CI 必须拦截越界行为
