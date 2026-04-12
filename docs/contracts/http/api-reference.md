# HTTP API Documentation

> Auto-generated from `packages/contracts/` and `servers/*/openapi.yaml`.

## Base URLs

| Environment | URL |
|-------------|-----|
| Local Dev (Web BFF) | `http://localhost:3000` |
| Local Dev (Admin BFF) | `http://localhost:3001` |
| Local Dev (Web App) | `http://localhost:5173` |
| Production | `https://api.example.com` |

## Authentication

All authenticated endpoints require a valid JWT access token passed via:
- Cookie: `session` (set during OAuth login)
- Header: `Authorization: Bearer <token>`

## Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Human-readable error description",
    "details": null
  }
}
```

See [Error Codes](../error-codes.md) for the complete catalog.

---

## Health & Readiness

### GET /healthz

**Authentication**: Not required
**Purpose**: Liveness probe — is the server running?

**Response** `200 OK`:
```json
{
  "status": "ok"
}
```

### GET /readyz

**Authentication**: Not required
**Purpose**: Readiness probe — are all dependencies (DB, cache, etc.) available?

**Response** `200 OK`:
```json
{
  "status": "ok"
}
```

**Response** `503 Service Unavailable`:
```json
{
  "error": {
    "code": "service_unavailable",
    "message": "Database connection failed",
    "details": null
  }
}
```

---

## Authentication

### POST /auth/oauth/authorize

**Authentication**: Not required
**Purpose**: Initiate OAuth login flow. Redirects to OAuth provider.

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | Yes | OAuth provider name (e.g., `google`, `github`) |

**Response** `302 Found`: Redirect to OAuth provider

**OAuth Callback**: `GET /auth/oauth/callback?code=xxx&state=yyy`

---

## User Service

### GET /api/user/me

**Authentication**: Required
**Purpose**: Get current user's profile

**Response** `200 OK`:
```json
{
  "sub": "user_01hxyz...",
  "email": "user@example.com",
  "name": "John Doe",
  "picture": "https://..."
}
```

**Response** `401 Unauthorized`:
```json
{
  "error": {
    "code": "unauthorized",
    "message": "Invalid or expired token",
    "details": null
  }
}
```

### GET /api/user/tenants

**Authentication**: Required
**Purpose**: Get all tenants the user belongs to

**Response** `200 OK`:
```json
[
  {
    "tenant_id": "tenant_01hxyz...",
    "name": "Acme Corp",
    "role": "owner"
  }
]
```

---

## Tenant Service

### POST /api/tenant/init

**Authentication**: Required
**Purpose**: Initialize a new tenant

**Request Body**:
```json
{
  "user_sub": "user_01hxyz...",
  "user_name": "John Doe"
}
```

**Response** `201 Created`:
```json
{
  "tenant_id": "tenant_01hxyz...",
  "role": "owner",
  "created": "2026-04-12T10:00:00Z"
}
```

**Response** `409 Conflict`:
```json
{
  "error": {
    "code": "conflict",
    "message": "Tenant with this slug already exists",
    "details": null
  }
}
```

---

## Counter Service

### POST /api/counter/increment

**Authentication**: Required
**Purpose**: Increment the counter for the current tenant

**Response** `200 OK`:
```json
{
  "value": 42
}
```

### POST /api/counter/decrement

**Authentication**: Required
**Purpose**: Decrement the counter

**Response** `200 OK`:
```json
{
  "value": 41
}
```

### POST /api/counter/reset

**Authentication**: Required
**Purpose**: Reset the counter to zero

**Response** `200 OK`:
```json
{
  "value": 0
}
```

### GET /api/counter/value

**Authentication**: Required
**Purpose**: Get the current counter value

**Response** `200 OK`:
```json
{
  "value": 42
}
```

---

## Settings Service

### GET /api/settings

**Authentication**: Required
**Purpose**: Get current settings (API key is redacted)

**Response** `200 OK`:
```json
{
  "api_key": "***REDACTED***",
  "base_url": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

### PUT /api/settings

**Authentication**: Required
**Purpose**: Update settings

**Request Body**:
```json
{
  "api_key": "sk-...",
  "base_url": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

**Response** `200 OK`:
```json
{
  "api_key": "***REDACTED***",
  "base_url": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

---

## Agent Service

### GET /api/agent/conversations

**Authentication**: Required
**Purpose**: List user's conversations

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | int | No | Page size (default 20) |
| `cursor` | string | No | Pagination cursor |

**Response** `200 OK`:
```json
[
  {
    "id": "conv_01hxyz...",
    "title": "My Conversation",
    "created_at": "2026-04-12T10:00:00Z",
    "updated_at": "2026-04-12T11:00:00Z"
  }
]
```

### POST /api/agent/conversations

**Authentication**: Required
**Purpose**: Create a new conversation

**Request Body**:
```json
{
  "title": "New Conversation"
}
```

**Response** `201 Created`:
```json
{
  "id": "conv_01hxyz...",
  "title": "New Conversation",
  "created_at": "2026-04-12T10:00:00Z",
  "updated_at": "2026-04-12T10:00:00Z"
}
```

### GET /api/agent/conversations/{id}/messages

**Authentication**: Required
**Purpose**: Get messages in a conversation

**Response** `200 OK`:
```json
[
  {
    "id": "msg_01hxyz...",
    "conversation_id": "conv_01hxyz...",
    "role": "user",
    "content": "Hello!",
    "created_at": "2026-04-12T10:00:00Z"
  }
]
```

### POST /api/agent/chat (SSE)

**Authentication**: Required
**Purpose**: Send a chat message and stream response via Server-Sent Events

**Request Body**:
```json
{
  "conversation_id": "conv_01hxyz...",
  "content": "What is the weather?",
  "api_key": "sk-...",
  "base_url": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

**Response** `200 OK` (SSE stream):
```
data: {"type": "chunk", "content": "The weather is..."}
data: {"type": "chunk", "content": "sunny today."}
data: {"type": "done"}
```

---

## Admin BFF

### GET /api/admin/dashboard

**Authentication**: Required (admin role)
**Purpose**: Get admin dashboard statistics

**Response** `200 OK`:
```json
{
  "tenant_count": 15,
  "counter_value": 42,
  "last_login": "2026-04-12T10:00:00Z",
  "app_version": "0.1.0"
}
```

### GET /api/admin/tenants

**Authentication**: Required (admin role)
**Purpose**: List all tenants

**Response** `200 OK`:
```json
[
  {
    "tenant_id": "tenant_01hxyz...",
    "name": "Acme Corp",
    "owner": "user_01hxyz..."
  }
]
```

### GET /api/admin/metrics

**Authentication**: Required (admin role)
**Purpose**: Get system metrics

**Response** `200 OK`:
```json
{
  "total_requests": 10000,
  "active_users": 150,
  "error_rate": 0.01
}
```
