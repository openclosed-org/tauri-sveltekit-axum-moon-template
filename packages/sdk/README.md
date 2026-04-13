# SDK — Generated API Clients

> ⚠️ 当前状态：占位目录。
>
> **决策**：当前前端直接使用 `apps/web/src/lib/generated/api/*` 作为 API 类型消费路径。
> `packages/sdk/` 作为统一 SDK 真理源的目标保留，但暂不执行迁移。
>
> **迁移条件**（满足任一即触发）：
> 1. 出现第二个前端消费端（mobile app、browser extension 需要相同 API 类型）
> 2. 需要跨端共享的客户端逻辑（auth refresh、tenant injection、retry policy）
> 3. 后端 OpenAPI 契约变更频繁，需要集中管理生成流程
>
> **当前方案**：
> - 前端类型生成：`ts-rs` 从 Rust 服务端代码生成 → `apps/web/src/lib/generated/api/`
> - 生成触发：`just typegen` 或 CI 自动执行
>
> **未来方案**（当迁移条件触发时）：
> - 统一生成目标：`packages/sdk/typescript/`
> - 前端消费：`import { ... } from '@workspace/sdk/typescript'`
> - 生成工具：`platform/generators/sdk/` 从 OpenAPI + ts-rs 统一生成
