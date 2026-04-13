# BFF OpenAPI Strategy

> 决策记录：BFF 层是否需要独立 OpenAPI 文档。

## 决策

**BFF 不维护独立 OpenAPI**。理由：

1. BFF 是 API 聚合层，不产生新契约
2. 所有真实业务契约来自 `servers/api/openapi.yaml` 和 `packages/contracts/*`
3. 维护多份 OpenAPI 会造成契约漂移风险

## 替代方案

- **文档化路由映射**：每个 BFF 在 `docs/contracts/bff/<name>-routes.md` 记录路由转发关系
- **契约验证**：BFF handler 的 request/response 类型直接复用 `servers/api` 的 ts-rs 生成类型
- **前端类型**：前端消费 `apps/web/src/lib/generated/api/*`（由 ts-rs 从服务端代码生成）

## 例外

如果 BFF 产生了**新的业务语义**（不仅仅是转发），例如：
- BFF 聚合多个 API 后返回新的组合视图
- BFF 实现端特有的业务逻辑

则该 BFF 需要补充独立 OpenAPI。当前 web-bff 和 admin-bff 均不属于此情况。

## 验证

- BFF handler 的 input/output 类型与 upstream API 一致
- 前端类型变更后，BFF 编译不中断
- 不存在 BFF 特有的、未在上游 API 中定义的业务规则
