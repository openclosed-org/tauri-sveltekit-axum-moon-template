# packages/contracts/ — Structure

> **Status**: Cleaned and stabilized (2026-04-12)

## Current Structure

```
packages/contracts/
├── api/          (contracts_api)     — HTTP API DTOs (HealthResponse, ChatMessage, CounterResponse, AdminDashboardStats, etc.)
├── auth/         (contracts_auth)    — Authentication types (TokenPair, OAuthCallback, UserProfile, UserSession)
├── events/       (contracts_events)  — Event types (TenantCreated, CounterChanged, ChatMessageSent, AppEvent enum)
└── errors/       (contracts_errors)  — Error types (ApiError, ErrorCode enum, ErrorResponse, ApiResult<T>)
```

## 已清理项

以下空目录已删除：
- `protocols/` — 空
- `codegen/` — 空
- `generated/` — 仅 .gitignore
- `ui/` — 空

## 规则

1. **所有对外协议** 必须在 `contracts/` 下有对应的 crate
2. **新增契约** 优先放入 `api/` 或 `events/`，仅当形成独立子域时才建新 crate
3. **错误码** 统一在 `errors/` 中定义，各 port/feature 不重复定义错误码
