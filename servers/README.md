# Servers

Composition layer for standalone services. Each server is a **binary** that wires together usecases, adapters, and domain logic.

## Structure

```
servers/
├── api/        # ✅ Axum HTTP API service (running)
├── gateway/    # ⚠️ API gateway (stub — Pingora in Phase 2)
└── realtime/   # ⚠️ Realtime service (stub — WebSocket/SSE in Phase 1)
```

## Responsibility

Servers are **composition roots only**:
- Register middleware (CORS, tracing, tenant validation)
- Wire routes to usecase implementations
- Initialize connections (DB, cache, HTTP client)
- **No business logic** — all logic lives in `packages/core/usecases/`

## Current: API Server

The Axum API server (`servers/api/`) is the primary service:
- Health endpoints: `/healthz`, `/readyz`
- Counter API: `/api/counter/*`
- Agent API: `/api/agent/*`
- Admin API: `/api/admin/*`
- Tenant management: `/api/tenants/*`

Run with: `just dev-api`
