# BFF OpenAPI Strategy

> 决策记录：web-bff 暴露的 HTTP API 契约如何生成和验证。

## 决策

web-bff 的 HTTP OpenAPI 契约由 Rust/Axum source 生成：

```text
servers/bff/web-bff/src/handlers/**
  + packages/contracts/api/** DTOs
  + contracts_errors::ErrorResponse
  -> utoipa-axum route collection
  -> packages/contracts/generated/openapi/web-bff.openapi.yaml
```

规则：

1. 不维护手写 canonical OpenAPI YAML。
2. 不从 HTTP API DTO 生成 TypeScript 类型。
3. `packages/contracts/generated/openapi/web-bff.openapi.yaml` 是 generated artifact，只能通过 `just typegen` 更新。
4. `/scalar`、`/openapi.json`、`/openapi.yaml` 使用同一份 runtime OpenAPI object。
5. 文档只做导航，不作为协议事实源。

## SSE Policy

OpenAPI artifact 可以描述 SSE endpoint 的 `text/event-stream` response，但 streaming 语义必须由 runtime code 和测试证明。

如果未来新增 SSE endpoint，必须同时明确：

1. Axum runtime 使用 `axum::response::sse::Sse`。
2. 该 route 不受普通 30s request timeout 错误截断。
3. auth/tenant 边界、client disconnect cancellation、heartbeat/backpressure、以及 `Last-Event-ID` replay 策略。
4. OpenAPI response 使用 `text/event-stream` 并引用稳定事件 payload schema。

## 验证

常规 contract 变更运行：

```bash
just typegen
just drift-check
just verify-contracts strict
```

server contract 变更还应运行：

```bash
cargo test -p web-bff
just check-backend-primary
```
