# Error Code Catalog

> All error codes are defined in `packages/contracts/errors/src/lib.rs`.
> This is the single source of truth for API error responses.

## Error Response Format

All API errors follow this format:

```json
{
  "error": {
    "code": "<ErrorCode>",
    "message": "<human-readable description>",
    "details": null
  }
}
```

## HTTP Status Code Mapping

### 4xx Client Errors

| Error Code | HTTP Status | Description | When to Use |
|------------|------------|-------------|-------------|
| `bad_request` | 400 | The request was malformed or invalid | Missing required fields, invalid JSON, wrong content type |
| `unauthorized` | 401 | Authentication required or token invalid | Missing/expired/invalid JWT, no session cookie |
| `forbidden` | 403 | User lacks required permissions | Valid auth but insufficient role/permissions |
| `not_found` | 404 | Resource does not exist | Requested ID doesn't exist, invalid route |
| `conflict` | 409 | Request conflicts with current state | Duplicate resource, concurrent modification |
| `validation_error` | 422 | Request data fails validation | Invalid field values, failed business rules |
| `rate_limited` | 429 | Too many requests | Exceeded rate limit, retry after delay |

### 5xx Server Errors

| Error Code | HTTP Status | Description | When to Use |
|------------|------------|-------------|-------------|
| `internal_error` | 500 | Unexpected server error | Unhandled exception, panic recovery |
| `service_unavailable` | 503 | Service temporarily unavailable | DB connection lost, dependency down |
| `database_error` | 500 | Database operation failed | Query error, constraint violation, connection error |
| `cache_error` | 500 | Cache operation failed | Redis/Valkey connection error, cache miss critical |
| `external_service_error` | 502 | External service call failed | OAuth provider down, API gateway timeout |

## Detailed Error Descriptions

### bad_request

**HTTP Status**: 400 Bad Request

**Causes**:
- Malformed JSON in request body
- Missing required query parameters
- Invalid content type header
- Unknown endpoint or method

**Example**:
```json
{
  "error": {
    "code": "bad_request",
    "message": "Request body is not valid JSON",
    "details": null
  }
}
```

---

### unauthorized

**HTTP Status**: 401 Unauthorized

**Causes**:
- No authentication token provided
- Token has expired
- Token signature verification failed
- Session has been invalidated

**Example**:
```json
{
  "error": {
    "code": "unauthorized",
    "message": "Invalid or expired authentication token",
    "details": null
  }
}
```

**Client Action**: Redirect to login or refresh token

---

### forbidden

**HTTP Status**: 403 Forbidden

**Causes**:
- User lacks required role (e.g., `admin`)
- User not a member of requested tenant
- Resource access denied by OpenFGA policy

**Example**:
```json
{
  "error": {
    "code": "forbidden",
    "message": "User does not have admin privileges",
    "details": null
  }
}
```

**Client Action**: Show permission denied message

---

### not_found

**HTTP Status**: 404 Not Found

**Causes**:
- Requested resource ID does not exist
- Resource was deleted
- Invalid resource ID format

**Example**:
```json
{
  "error": {
    "code": "not_found",
    "message": "Conversation 'conv_invalid' not found",
    "details": null
  }
}
```

---

### conflict

**HTTP Status**: 409 Conflict

**Causes**:
- Duplicate resource creation (e.g., tenant slug already exists)
- Concurrent modification detected
- State mismatch

**Example**:
```json
{
  "error": {
    "code": "conflict",
    "message": "A tenant with slug 'acme' already exists",
    "details": null
  }
}
```

**Client Action**: Inform user of conflict, suggest alternatives

---

### validation_error

**HTTP Status**: 422 Unprocessable Entity

**Causes**:
- Request field fails format validation
- Business rule violation (e.g., negative counter)
- Missing required field

**Example**:
```json
{
  "error": {
    "code": "validation_error",
    "message": "Invalid email format: 'not-an-email'",
    "details": {
      "field": "email",
      "value": "not-an-email",
      "rule": "email_format"
    }
  }
}
```

---

### rate_limited

**HTTP Status**: 429 Too Many Requests

**Causes**:
- Exceeded requests per minute limit
- Exceeded concurrent connections limit
- Global rate limit reached

**Example**:
```json
{
  "error": {
    "code": "rate_limited",
    "message": "Rate limit exceeded. Try again in 30 seconds",
    "details": {
      "retry_after": 30
    }
  }
}
```

**Headers**: `Retry-After: 30`

---

### internal_error

**HTTP Status**: 500 Internal Server Error

**Causes**:
- Unhandled exception in service code
- Unexpected panic in middleware
- Configuration error

**Example**:
```json
{
  "error": {
    "code": "internal_error",
    "message": "An unexpected error occurred. Please try again later.",
    "details": null
  }
}
```

**Note**: Internal details are NEVER exposed to clients in production.

---

### service_unavailable

**HTTP Status**: 503 Service Unavailable

**Causes**:
- Database connection pool exhausted
- NATS connection lost
- Required dependency not responding
- Maintenance mode

**Example**:
```json
{
  "error": {
    "code": "service_unavailable",
    "message": "Service temporarily unavailable. Please try again later.",
    "details": null
  }
}
```

---

### database_error

**HTTP Status**: 500 Internal Server Error

**Causes**:
- SQL constraint violation
- Database connection error
- Migration not applied
- Query timeout

**Example**:
```json
{
  "error": {
    "code": "database_error",
    "message": "Failed to execute database query",
    "details": null
  }
}
```

---

### cache_error

**HTTP Status**: 500 Internal Server Error

**Causes**:
- Valkey/Redis connection lost
- Cache serialization error
- Cache eviction failure

**Example**:
```json
{
  "error": {
    "code": "cache_error",
    "message": "Cache service unavailable",
    "details": null
  }
}
```

---

### external_service_error

**HTTP Status**: 502 Bad Gateway

**Causes**:
- OAuth provider API down
- Third-party API timeout
- Invalid response from external service

**Example**:
```json
{
  "error": {
    "code": "external_service_error",
    "message": "OAuth provider is currently unavailable",
    "details": null
  }
}
```

---

## Service-Specific Error Types

These error types are used internally by services and are NOT part of the public API error codes. They may be mapped to public error codes before returning to clients.

### AuthError (feature-auth)

| Variant | Maps To | Description |
|---------|---------|-------------|
| `Network` | `external_service_error` | Network call to OAuth provider failed |
| `Config` | `internal_error` | OAuth configuration missing/invalid |
| `InvalidCallback` | `bad_request` | OAuth callback URL malformed |
| `TokenExchange` | `external_service_error` | Token exchange with provider failed |
| `SessionExpired` | `unauthorized` | User session has expired |
| `Database` | `database_error` | Session storage operation failed |

### CounterError (feature-counter)

| Variant | Maps To | Description |
|---------|---------|-------------|
| `Database` | `database_error` | Counter DB operation failed |
| `NotFound` | `not_found` | Counter for tenant not found |

### AgentError (feature-agent)

| Variant | Maps To | Description |
|---------|---------|-------------|
| `Database` | `database_error` | Agent DB operation failed |
| `Api` | `external_service_error` | LLM API call failed |
| `Config` | `internal_error` | Agent configuration invalid |
| `NotFound` | `not_found` | Conversation not found |

### ChatError (feature-chat)

| Variant | Maps To | Description |
|---------|---------|-------------|
| `Database` | `database_error` | Chat DB operation failed |
| `NotFound` | `not_found` | Conversation or message not found |
| `PermissionDenied` | `forbidden` | User not authorized for this conversation |
| `StreamError` | `external_service_error` | SSE streaming error |

### EventBusError

| Variant | Maps To | Description |
|---------|---------|-------------|
| `PublishFailed` | `service_unavailable` | Failed to publish event |
| `SubscribeFailed` | `service_unavailable` | Failed to subscribe to events |
| `HandlerError` | `internal_error` | Event handler panic or error |

### PubSubError

| Variant | Maps To | Description |
|---------|---------|-------------|
| `PublishFailed` | `service_unavailable` | NATS publish failed |
| `SubscribeFailed` | `service_unavailable` | NATS subscribe failed |
| `UnsubscribeFailed` | `internal_error` | Error unsubscribing from topic |
| `HandlerError` | `internal_error` | Message handler error |
| `ConnectionError` | `service_unavailable` | NATS connection lost |

---

## Error Handling Best Practices

### For API Clients

1. **Always check the `error.code` field**, not just HTTP status
2. **Handle `rate_limited`** with exponential backoff using `retry_after`
3. **Handle `unauthorized`** by redirecting to login
4. **Handle `conflict`** by showing user-friendly conflict resolution
5. **Log `internal_error`** with correlation ID for support

### For Service Developers

1. **Use specific error codes** — prefer `validation_error` over `bad_request` when applicable
2. **Never expose internal details** — map internal errors to public codes
3. **Include correlation IDs** in error responses for tracing
4. **Use the `details` field** for structured error information (field-level validation, retry-after, etc.)
5. **Log the full error** with context before mapping to public code
