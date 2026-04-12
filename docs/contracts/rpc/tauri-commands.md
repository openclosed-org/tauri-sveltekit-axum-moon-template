# RPC Documentation (Tauri Commands)

> This document describes the Tauri command interface used by the Desktop application.
> Commands are invoked from the frontend via `@tauri-apps/api`'s `invoke()` function.

## Command Invocation Pattern

```typescript
import { invoke } from '@tauri-apps/api/core';

// Example: Invoke counter increment
const result = await invoke<number>('counter_increment');
```

## Auth Commands

### `start_oauth`

Initiates the OAuth login flow by opening the browser to the OAuth authorization URL.

**Input**: None (uses app state)
**Output**: `Result<(), string>`

```typescript
await invoke('start_oauth');
```

### `handle_oauth_callback`

Handles the OAuth callback URL after the user authenticates.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `url` | string | Full callback URL with code and state params |

**Output**: `Result<AuthSession, string>`

```typescript
const session = await invoke<AuthSession>('handle_oauth_callback', { url: callbackUrl });
```

### `get_session`

Retrieves the current authentication session.

**Input**: None
**Output**: `Result<AuthSession | null, string>`

```typescript
const session = await invoke<AuthSession | null>('get_session');
```

### `logout`

Logs out the current user and clears session state.

**Input**: None
**Output**: `Result<(), string>`

```typescript
await invoke('logout');
```

### `quit_app`

Exits the application.

**Input**: None
**Output**: `void`

```typescript
await invoke('quit_app');
```

---

## Counter Commands

### `counter_increment`

Increments the counter by 1.

**Input**: None
**Output**: `Result<number, string>` — New counter value

```typescript
const value = await invoke<number>('counter_increment');
```

### `counter_decrement`

Decrements the counter by 1.

**Input**: None
**Output**: `Result<number, string>` — New counter value

```typescript
const value = await invoke<number>('counter_decrement');
```

### `counter_reset`

Resets the counter to 0.

**Input**: None
**Output**: `Result<number, string>` — Always returns 0

```typescript
const value = await invoke<number>('counter_reset');
```

### `counter_get_value`

Gets the current counter value.

**Input**: None
**Output**: `Result<number, string>` — Current counter value

```typescript
const value = await invoke<number>('counter_get_value');
```

---

## Agent Commands

### `agent_create_conversation`

Creates a new conversation.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Conversation title |

**Output**: `Result<Conversation, string>`

```typescript
const conv = await invoke<Conversation>('agent_create_conversation', { title: "My Chat" });
```

### `agent_list_conversations`

Lists all conversations for the current user.

**Input**: None
**Output**: `Result<Conversation[], string>`

```typescript
const conversations = await invoke<Conversation[]>('agent_list_conversations');
```

### `agent_get_messages`

Gets all messages in a conversation.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Conversation ID |

**Output**: `Result<ChatMessage[], string>`

```typescript
const messages = await invoke<ChatMessage[]>('agent_get_messages', { id: "conv_01hxyz..." });
```

### `agent_chat`

Sends a chat message and streams the AI response via Tauri Channel (SSE-like).

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `conversation_id` | string | Target conversation |
| `content` | string | User message content |
| `api_key` | string | OpenAI API key |
| `base_url` | string | API base URL |
| `model` | string | Model name (e.g., `gpt-4`) |
| `channel` | Channel | Tauri event channel for streaming |

**Output**: `Result<(), string>` (streaming via channel)

```typescript
const channel = new Channel<string>();
channel.onmessage = (chunk) => {
  console.log('Streamed chunk:', chunk);
};

await invoke('agent_chat', {
  conversation_id: 'conv_01hxyz...',
  content: 'Hello!',
  api_key: 'sk-...',
  base_url: 'https://api.openai.com/v1',
  model: 'gpt-4',
  channel
});
```

---

## Settings Commands

### `settings_get`

Retrieves current settings (API key is redacted).

**Input**: None
**Output**: `Result<object, string>`

```typescript
const settings = await invoke<object>('settings_get');
```

### `settings_update`

Updates settings.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `api_key` | string | API key |
| `base_url` | string | API base URL |
| `model` | string | Model name |

**Output**: `Result<object, string>` — Updated settings (key redacted)

```typescript
const updated = await invoke<object>('settings_update', {
  api_key: 'sk-...',
  base_url: 'https://api.openai.com/v1',
  model: 'gpt-4'
});
```

---

## Admin Commands

### `admin_get_dashboard_stats`

Gets admin dashboard statistics.

**Input**: None
**Output**: `Result<object, string>`

```typescript
const stats = await invoke<object>('admin_get_dashboard_stats');
```

---

## Config Commands

### `get_config`

Gets the application configuration.

**Input**: None
**Output**: `AppConfig`

```typescript
interface AppConfig {
  google_client_id: string;
  google_client_secret: string;
  api_url: string;
}

const config = await invoke<AppConfig>('get_config');
```

---

## Sync Commands

### `sync_start`

Starts the background sync process.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `sync_state` | SyncState | Current sync state |
| `app_handle` | AppHandle | Tauri app handle |

**Output**: `Result<(), string>`

```typescript
await invoke('sync_start', { sync_state, app_handle });
```

### `sync_stop`

Stops the background sync process.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `sync_state` | SyncState | Current sync state |

**Output**: `Result<(), string>`

```typescript
await invoke('sync_stop', { sync_state });
```

### `sync_once`

Runs a single sync cycle.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `sync_state` | SyncState | Current sync state |

**Output**: `Result<(), string>`

```typescript
await invoke('sync_once', { sync_state });
```

### `sync_get_stats`

Gets sync statistics.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `sync_state` | SyncState | Current sync state |

**Output**: `Result<SyncStatsResponse, string>`

```typescript
const stats = await invoke<SyncStatsResponse>('sync_get_stats', { sync_state });
```

### `sync_configure`

Configures sync parameters.

**Input**:
| Field | Type | Description |
|-------|------|-------------|
| `sync_state` | SyncState | Current sync state |
| `config` | SyncConfigureRequest | Sync configuration |

**Output**: `Result<(), string>`

```typescript
await invoke('sync_configure', { sync_state, config: { interval_ms: 5000 } });
```

---

## Type Definitions

### AuthSession

```typescript
interface AuthSession {
  user_sub: string;
  tenant_id: string;
  role: string;
}
```

### Conversation

```typescript
interface Conversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}
```

### ChatMessage

```typescript
interface ChatMessage {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  tool_calls?: ToolCall[];
  created_at: string;
}
```

### ToolCall

```typescript
interface ToolCall {
  id: string;
  name: string;
  arguments: string;
  result?: string;
}
```

### SyncStatsResponse

```typescript
interface SyncStatsResponse {
  last_sync: string | null;
  pending_changes: number;
  total_synced: number;
}
```

### SyncConfigureRequest

```typescript
interface SyncConfigureRequest {
  interval_ms?: number;
  // Additional sync configuration fields
}
```
