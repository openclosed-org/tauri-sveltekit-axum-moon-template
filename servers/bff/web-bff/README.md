# Web BFF

> 目的：说明 `web-bff` 作为当前默认同步入口时，负责什么、聚合哪些能力，以及哪些事实不能被过度推断。

## 状态

- status: `reference`
- 角色：默认用户侧同步入口与当前 `counter-service` 参考链的 composition root
- 说明：当前它直接组合 `counter-service`、`tenant-service`、`user-service` 的具体适配实现，因此仍承担一部分临时 wiring 责任

## 责任

1. 提供面向用户界面的 HTTP 入口。
2. 组装当前默认参考链所需的 service、repository 与 authz 依赖。
3. 维持同步命令/查询到 service library 的明确边界。

## 入口

1. `src/main.rs`：Axum 服务入口。
2. `src/state.rs`：当前 composition root，负责组装 `counter-service`、`tenant-service` 与 `user-service` 依赖。
3. `src/handlers/counter.rs`：默认参考链的同步 handler 入口。
4. `src/handlers/tenant.rs` 与 `src/handlers/user.rs`：非默认参考链但仍存在的用户侧聚合入口。

## 当前调试与验证

1. 默认后端调试可优先使用 `APP_AUTH_MODE=dev_headers`，避免 OAuth/Zitadel 成为 `tenant/init` 与 `counter/*` handler 调试前提。
2. 默认 admission 关注 `counter-service + tenant-service + web-bff`，对应 `just check-backend-primary` / `just test-backend-primary`。
3. Zitadel/OpenFGA 保留为可选 auth lane，对应 `just verify-auth-optional` / `just test-auth-optional`。

## Protected Endpoint Error Matrix

当前 `web-bff` 的受保护 API 已统一到 `contracts_errors::ErrorResponse`：

| Endpoint | 200 | 400 | 401 | 403 | 404 | 409 | 415 | 422 | 500 |
|----------|-----|-----|-----|-----|-----|-----|-----|-----|-----|
| `POST /api/tenant/init` | ✓ | Malformed JSON | Missing/invalid JWT; JWT sub mismatch | - | - | - | Missing content-type | Validation failure | DB not initialized; tenant creation failure |
| `POST /api/counter/increment` | ✓ | - | Missing request context; No tenant binding | Authz denied; Tenant claim mismatch | - | CAS conflict | - | - | DB not initialized; Service failure |
| `POST /api/counter/decrement` | ✓ | - | Missing request context; No tenant binding | Authz denied; Tenant claim mismatch | - | CAS conflict | - | - | DB not initialized; Service failure |
| `POST /api/counter/reset` | ✓ | - | Missing request context; No tenant binding | Authz denied; Tenant claim mismatch | - | CAS conflict | - | - | DB not initialized; Service failure |
| `GET /api/counter/value` | ✓ | - | Missing request context; No tenant binding | Authz denied; Tenant claim mismatch | - | - | - | - | DB not initialized; Service failure |
| `GET /api/user/me` | ✓ | - | Missing request context | - | User profile not found | - | - | - | DB not initialized; Repository failure |
| `GET /api/user/tenants` | ✓ | - | Missing request context | - | - | - | - | - | DB not initialized; Repository failure |

错误码说明：

- `400 BadRequest`：请求体 JSON 格式错误（语法层面）
- `401 Unauthorized`：缺少 bearer token、token 无效、缺少 request context、无 tenant binding、JWT sub 与请求 body 不一致
- `403 Forbidden`：authz 检查明确拒绝、tenant claim 与持久化 binding 不一致
- `404 NotFound`：`GET /api/user/me` 当前用户无 profile 记录
- `409 Conflict`：counter CAS 冲突（并发修改）
- `415 UnsupportedMediaType`：`POST /api/tenant/init` 缺少 `application/json` content-type
- `422 ValidationError`：`POST /api/tenant/init` 字段校验失败（如空字段）
- `500 InternalError`：数据库未初始化、仓储访问失败、service 执行失败

说明：

1. `GET /api/user/me` 与 `GET /api/user/tenants` 现在也走统一错误契约，不再返回裸 `{ "error": ... }`。
2. `src/state.rs` 当前已负责 tenant/user/counter 所需的本地 schema 初始化，因此 `user-service` 的读仓储不再依赖手工预建 `user` 表。
3. 所有 protected endpoints 的 OpenAPI 注解已与实际行为对齐，`utoipa::path` 的 `responses` 字段准确反映当前错误矩阵。

## 不要这样用

1. 不要把它写成“纯 HTTP adapter”而忽略当前它还承担 concrete adapter wiring 责任。
2. 不要因为它聚合了多个 service，就把 `tenant-service` 或 `user-service` 误判成与 `counter-service` 同级的默认参考链。
3. 不要因为 `counter-service` 当前通过这里内嵌接入，就把 `counter-service` 独立 deployable 误写成已闭环事实。
