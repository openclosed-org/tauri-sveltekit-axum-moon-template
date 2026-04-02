# Phase 06: Google OAuth Authentication — Research

**Researched:** 2026-03-29
**Confidence:** HIGH

---

## 1. Google OAuth 2.0 PKCE Flow for Desktop Apps

### Standard Flow (RFC 8252)

```
1. App generates PKCE code_verifier (random 43-128 chars) + code_challenge (SHA256 hash)
2. App opens system browser → accounts.google.com/o/oauth2/v2/auth
   ?client_id=...&redirect_uri=com.example.app://callback&response_type=code&code_challenge=...&code_challenge_method=S256
3. User authenticates → Google redirects to com.example.app://callback?code=...&state=...
4. App captures code via deep link → POST https://oauth2.googleapis.com/token
   { code, client_id, redirect_uri, code_verifier, grant_type=authorization_code }
5. Google returns { access_token, refresh_token, id_token, expires_in }
6. App stores tokens, extracts user info from id_token (JWT)
```

### Key Parameters

| Parameter | Value | Source |
|-----------|-------|--------|
| `client_id` | Google Cloud OAuth 2.0 Desktop Client ID | User creates in Google Cloud Console |
| `redirect_uri` | `com.example.app://oauth/callback` | Custom scheme from tauri.conf.json identifier |
| `scope` | `openid email profile` | Standard OIDC scopes |
| `code_challenge_method` | `S256` | Required for PKCE |

### PKCE Implementation

```rust
use sha2::{Sha256, Digest};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;

fn generate_pkce() -> (String, String) {
    // code_verifier: 43-128 character random string
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    // code_challenge: SHA256(code_verifier) → base64url
    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    (code_verifier, code_challenge)
}
```

**Dependencies needed:** `sha2` (already transitively available), `base64` (common), `rand` (common). Can use existing workspace deps.

---

## 2. Tauri Deep Link Configuration (Tauri 2)

### Plugin Setup

`tauri-plugin-deep-link` is already declared in workspace Cargo.toml AND src-tauri/Cargo.toml. But NOT registered in `lib.rs`.

```rust
// lib.rs — add before .run()
.plugin(tauri_plugin_deep_link::init())
```

### Capabilities File

**Critical discovery:** `capabilities/default.json` does NOT exist yet. Must be created.

Tauri 2 requires capability declarations for all plugins. Deep link needs:

```json
{
  "identifier": "default",
  "description": "Default capabilities for the app",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "dialog:default",
    "store:default",
    "store:allow-get",
    "store:allow-set",
    "store:allow-delete",
    "deep-link:default"
  ]
}
```

### Platform Registration

**Windows:** Register protocol in `tauri.conf.json`:
```json
{
  "app": {
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "identifier": "com.example.app"
  }
}
```

The `tauri-plugin-deep-link` handles platform registration automatically based on the bundle identifier.

### Deep Link Callback Handler

```rust
use tauri_plugin_deep_link::DeepLink;

#[tauri::command]
async fn handle_deep_link(app: tauri::AppHandle, url: String) -> Result<(), String> {
    // Parse: com.example.app://oauth/callback?code=...&state=...
    let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
    let code = parsed.query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or("No authorization code in callback")?;

    // Exchange code for tokens...
    Ok(())
}
```

---

## 3. Token Storage with tauri-plugin-store

### Store API (Rust side)

```rust
use tauri_plugin_store::StoreExt;

// Write
app.store("auth.json")?.set("access_token", serde_json::json!(token));
app.store("auth.json")?.set("refresh_token", serde_json::json!(refresh));
app.store("auth.json")?.set("expires_at", serde_json::json!(expires_at));
app.store("auth.json")?.set("user", serde_json::json!(user_info));

// Read
let token = app.store("auth.json")?.get("access_token");
```

### Store API (TypeScript side)

```typescript
import { Store } from '@tauri-apps/plugin-store';

const store = new Store('auth.json');

await store.set('access_token', token);
await store.set('user', userInfo);
const savedToken = await store.get<string>('access_token');
```

### Data Schema

```typescript
interface StoredAuth {
  access_token: string;
  refresh_token: string;
  id_token: string;
  expires_at: number; // Unix timestamp (seconds)
  user: {
    email: string;
    name: string;
    picture: string;
    sub: string; // Google user ID
  };
}
```

### Why tauri-plugin-store (not libsql)

Per D-05 in CONTEXT.md: tokens should NOT be mixed with business data in libsql. Store gives:
- Simple key-value access
- Platform-native storage location
- Not synced to cloud (Turso)
- Already registered in Tauri builder

---

## 4. Background Token Refresh

### Strategy

```rust
// On app startup:
// 1. Read tokens from store
// 2. Calculate time until expiry
// 3. If already expired → clear tokens, return expired state
// 4. If < 5 min until expiry → refresh immediately
// 5. Otherwise → schedule tokio::spawn for (expires_at - 300 seconds)

use tokio::time::{interval, Duration};

async fn start_refresh_timer(app: tauri::AppHandle) {
    let store = app.store("auth.json").expect("store init failed");

    let Some(expires_at) = store.get("expires_at").and_then(|v| v.as_u64()) else {
        return; // No stored session
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if expires_at <= now {
        // Already expired
        clear_tokens(&app);
        return;
    }

    let refresh_at = if expires_at - now > 300 {
        Duration::from_secs(expires_at - now - 300)
    } else {
        Duration::from_secs(0) // Refresh immediately
    };

    tokio::spawn(async move {
        tokio::time::sleep(refresh_at).await;
        // POST to Google token endpoint with refresh_token
        // Update store with new access_token + expires_at
        // If fails: clear tokens, emit "auth:expired" event to frontend
    });
}
```

### Refresh Token Exchange

```rust
use reqwest::Client;

async fn refresh_access_token(client: &Client, refresh_token: &str) -> Result<TokenResponse> {
    let params = [
        ("client_id", GOOGLE_CLIENT_ID),
        ("client_secret", GOOGLE_CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    resp.json::<TokenResponse>().await
}
```

### Client Secret Handling

Google OAuth Desktop apps should use `client_secret` from the OAuth client. For production, this should be:
- Embedded at build time (env var during `cargo build`)
- NOT in source code

```rust
const GOOGLE_CLIENT_ID: &str = option_env!("GOOGLE_CLIENT_ID").unwrap_or("");
const GOOGLE_CLIENT_SECRET: &str = option_env!("GOOGLE_CLIENT_SECRET").unwrap_or("");
```

---

## 5. Login Page UX with Lottie

### Lottie Activation

`@lottiefiles/lottie-player` 0.9.8 is in package.json. Need to import in login page:

```svelte
<script lang="ts">
  import '@lottiefiles/lottie-player';
</script>

<lottie-player
  src="/animations/loading.json"
  background="transparent"
  speed="1"
  style="width: 40px; height: 40px;"
  loop
  autoplay
></lottie-player>
```

### Login Flow States

```
┌─────────────────────┐
│   IDLE STATE        │
│  [Google Button]    │
└─────────┬───────────┘
          │ click
          ▼
┌─────────────────────┐
│  LOADING STATE      │
│  [Lottie Spinner]   │
│  Opening browser... │
└─────────┬───────────┘
          │ callback received
          ▼
┌─────────────────────┐
│  SUCCESS STATE      │
│  Redirect → /counter│
└─────────────────────┘

OR (error path):
┌─────────────────────┐
│  ERROR STATE        │
│  [Google Button]    │
│  ⚠ Error message    │
└─────────────────────┘
```

### Svelte 5 Auth State

```typescript
// $lib/stores/auth.ts
import { invoke } from '@tauri-apps/api/core';

export let isAuthenticated = $state(false);
export let currentUser = $state<User | null>(null);
export let authLoading = $state(false);
export let authError = $state<string | null>(null);

export async function checkStoredSession() {
  const session = await invoke<AuthSession | null>('get_session');
  if (session && session.expires_at > Date.now() / 1000) {
    isAuthenticated = true;
    currentUser = session.user;
    return true;
  }
  return false;
}

export async function signInWithGoogle() {
  authLoading = true;
  authError = null;
  try {
    await invoke('start_oauth');
  } catch (e) {
    authError = String(e);
    authLoading = false;
  }
}
```

### Auth Guard for App Layout

```svelte
<!-- +layout.svelte in (app) group -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { isAuthenticated } from '$lib/stores/auth';

  $effect(() => {
    if (!isAuthenticated) {
      goto('/login');
    }
  });
</script>
```

---

## 6. Security Considerations

### Tauri 2 CSP

Current config has `csp: null` (no CSP). For production:
- Set CSP to allow Google domains
- Allow data: for Lottie

```json
"csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self' https://oauth2.googleapis.com https://www.googleapis.com; img-src 'self' https://lh3.googleusercontent.com data:"
```

### OAuth State Parameter

Generate random `state` parameter to prevent CSRF:
```rust
let state: String = rand::thread_rng()
    .sample_iter(&rand::distributions::Alphanumeric)
    .take(32)
    .map(char::from)
    .collect();

// Store state in app state, verify on callback
```

### Token Validation

Verify `id_token` signature using Google's public keys (JWKS):
```rust
// Use jsonwebtoken crate with Google's JWKS endpoint
// https://www.googleapis.com/oauth2/v3/certs
```

---

## 7. Dependencies Summary

### Rust (Workspace Cargo.toml — already declared)

| Crate | Purpose | Status |
|-------|---------|--------|
| `tauri-plugin-deep-link` | Deep link capture | ✅ Declared, NOT registered |
| `jsonwebtoken` | JWT parsing/verification | ✅ Declared |
| `reqwest` | Token endpoint calls | ✅ In AppState |

### Rust (New dependencies needed)

| Crate | Purpose |
|-------|---------|
| `sha2` | PKCE code_challenge |
| `base64` | Base64url encoding |
| `rand` | Random state/code_verifier |
| `url` | Callback URL parsing |

Note: `sha2`, `base64`, `rand` are commonly available in Tauri projects. Need to check if transitively available via existing deps.

### TypeScript (package.json — already declared)

| Package | Purpose | Status |
|---------|---------|--------|
| `@lottiefiles/lottie-player` | Loading animation | ✅ Declared (0.9.8) |

---

## 8. Validation Architecture

### Phase 6 Testing Strategy

| Test | Type | Target |
|------|------|--------|
| PKCE generation | Unit (Rust) | `generate_pkce()` produces valid verifier + challenge |
| Token exchange | Integration (Rust) | Mock Google endpoint, verify token parsing |
| Store persistence | Unit (Rust) | Store write → read round-trip |
| Login page states | Component (Vitest) | Idle → loading → success/error transitions |
| Auth guard | Component (Vitest) | Unauthenticated → redirect to login |
| Deep link handling | Integration (Rust) | Parse callback URL, extract code |
| Token refresh | Unit (Rust) | Schedule + execute refresh before expiry |

### Mock Strategy

For unit tests without real Google OAuth:
- Mock `reqwest::Client` responses
- Mock store with in-memory HashMap
- Use test Google OAuth client (Google provides test mode)

---

## 9. Key Implementation Sequence

1. **Backend:** Tauri commands + PKCE + token exchange + store
2. **Frontend:** Login page states + auth store + auth guard
3. **Integration:** Deep link callback → backend command → store → frontend state
4. **Polish:** Lottie animations + loading states + error handling

---

## Sources

- [Google OAuth 2.0 for Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app) — RFC 8252 compliance
- [tauri-plugin-deep-link v2](https://v2.tauri.app/plugin/deep-linking/) — Plugin setup, capabilities
- [tauri-plugin-store v2](https://v2.tauri.app/plugin/store/) — Persistent key-value storage
- [Tauri 2 Capabilities](https://v2.tauri.app/security/capabilities/) — Permission system
- CONTEXT.md — Phase decisions D-01 through D-12
- ARCHITECTURE.md §Pattern 5, §Integration Patterns — Plugin architecture, IPC

---

*Research complete for Phase 06: Google OAuth Authentication*
*Ready for planning*
