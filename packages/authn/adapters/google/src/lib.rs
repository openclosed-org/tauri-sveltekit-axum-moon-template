//! adapter-google — legacy Tauri-specific Google OAuth client adapter.
//!
//! This optional lane starts a browser login flow for a desktop shell. It is not
//! the backend resource-server verifier used by web-bff; generic OIDC bearer
//! token verification lives in `authn-oidc-verifier`.
//!
//! Wraps adapter-google-backend (pure OAuth logic) with Tauri-specific capabilities:
//! - Browser opener (via tauri-plugin-opener)
//! - Tauri store integration (via tauri-plugin-store)
//! - Tauri event emission
//! - TCP listener for OAuth callback

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpListener;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

const REDIRECT_URI: &str = "http://127.0.0.1:1420/oauth/callback";

// ── Re-export backend types for compatibility ────────────────

pub use adapter_google_backend::{AuthError, AuthSession, UserProfile};

// ── Tauri-specific wrapper functions ─────────────────────────

// ── GoogleAuthAdapter (Tauri-specific wrapper) ──────────────

/// Google OAuth PKCE adapter for Tauri.
///
/// Delegates pure OAuth logic to adapter-google-backend,
/// handles Tauri-specific concerns: browser opener, store, events.
pub struct GoogleAuthAdapter;

impl GoogleAuthAdapter {
    /// Create a new GoogleAuthAdapter instance.
    pub fn new() -> Self {
        Self
    }

    /// Start the Google OAuth PKCE login flow.
    ///
    /// 1. Generates PKCE code_verifier + code_challenge (via backend)
    /// 2. Generates CSRF state parameter (via backend)
    /// 3. Stores verifier/state in Tauri store
    /// 4. Starts TCP listener for redirect callback
    /// 5. Opens system browser with authorization URL (via backend + Tauri opener)
    pub async fn start_login(&self, app: &AppHandle) -> Result<(), String> {
        start_oauth(app.clone()).await
    }

    /// Handle the OAuth callback URL.
    ///
    /// Validates state, exchanges code for tokens (via backend), decodes user profile.
    pub async fn handle_callback(&self, app: &AppHandle, url: &str) -> Result<AuthSession, String> {
        handle_oauth_callback(app.clone(), url.to_string()).await
    }

    /// Get the current auth session from the Tauri store.
    pub fn get_session(&self, app: &AppHandle) -> Result<Option<AuthSession>, String> {
        get_session(app.clone())
    }

    /// Refresh the access token using the stored refresh token (via backend).
    pub async fn refresh_token(
        &self,
        app: &AppHandle,
        refresh_token: &str,
    ) -> Result<(String, u64), String> {
        refresh_access_token(app, refresh_token).await
    }

    /// Clear all auth tokens from store and notify frontend.
    pub fn clear_session(&self, app: &AppHandle) {
        clear_session_and_notify(app);
    }

    /// Start the background token refresh timer.
    pub fn start_timer(&self, app: AppHandle) {
        start_refresh_timer(app);
    }
}

impl Default for GoogleAuthAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Core OAuth logic (delegates to backend) ─────────────────

/// Start the Google OAuth PKCE login flow.
pub async fn start_oauth(app: AppHandle) -> Result<(), String> {
    // 1. Generate PKCE parameters (via backend)
    let code_verifier = adapter_google_backend::generate_code_verifier();
    let code_challenge = adapter_google_backend::compute_code_challenge(&code_verifier);
    let state = adapter_google_backend::generate_state();

    // 2. Store code_verifier and state in app store for callback validation
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("pkce_verifier", serde_json::json!(code_verifier));
    store.set("oauth_state", serde_json::json!(state));

    // 3. Start one-shot HTTP listener on 127.0.0.1:1420 to catch Google's redirect
    let listener = TcpListener::bind("127.0.0.1:1420")
        .map_err(|e| format!("Cannot bind 127.0.0.1:1420 — is another instance running? {e}"))?;
    let app_for_callback = app.clone();
    tokio::task::spawn_blocking(move || {
        tracing::info!("TCP listener waiting for callback on 127.0.0.1:1420");
        if let Ok((mut stream, _)) = listener.accept() {
            tracing::debug!("received connection on callback listener");
            let mut buf = Vec::with_capacity(4096);
            let mut tmp = [0u8; 1024];
            // Read all data until connection closes or we have the full headers
            loop {
                match stream.read(&mut tmp) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    Err(_) => break,
                }
                // Stop reading once we have the full HTTP headers (double CRLF)
                if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&buf);
            tracing::debug!(first_line = %request.lines().next().unwrap_or(""), "received HTTP request");

            // Extract first line: "GET /oauth/callback?code=...&state=... HTTP/1.1"
            if let Some(first_line) = request.lines().next()
                && let Some(path) = first_line.split_whitespace().nth(1)
            {
                let callback_url = format!("http://127.0.0.1:1420{path}");
                tracing::info!(%callback_url, "emitting oauth-callback event");
                // Use custom event name to avoid conflicts with tauri-plugin-deep-link
                if let Err(e) = app_for_callback.emit("oauth-callback", &callback_url) {
                    tracing::error!(%e, "failed to emit oauth-callback event");
                }
            }
            // Return HTML so the browser tab shows confirmation
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<html><body style='font-family:system-ui;text-align:center;padding:40px'><h2>Login successful!</h2><p>You can close this tab.</p></body></html>"
            );
            tracing::debug!("sent response to browser");
        }
        tracing::debug!("TCP listener exiting");
    });

    // 4. Build authorization URL (via backend)
    let url =
        adapter_google_backend::build_authorization_url(&code_challenge, &state, REDIRECT_URI);

    // 5. Open system browser (Tauri-specific)
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Handle the OAuth callback URL from Google.
pub async fn handle_oauth_callback(app: AppHandle, url: String) -> Result<AuthSession, String> {
    tracing::info!("processing OAuth callback");

    // 1. Parse callback URL: com.example.app://oauth/callback?code=...&state=...
    let parsed = url::Url::parse(&url).map_err(|e| format!("Invalid callback URL: {e}"))?;
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

    let code = params
        .get("code")
        .ok_or("No authorization code in callback")?
        .to_string();
    let returned_state = params
        .get("state")
        .ok_or("No state in callback")?
        .to_string();

    // 2. Validate state against stored state
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    let stored_state = store
        .get("oauth_state")
        .and_then(|v| v.as_str().map(String::from))
        .ok_or("No stored OAuth state")?;

    if returned_state != stored_state {
        tracing::error!("OAuth state mismatch — possible CSRF attack");
        return Err("OAuth state mismatch — possible CSRF attack".into());
    }

    // 3. Retrieve stored PKCE verifier
    let code_verifier = store
        .get("pkce_verifier")
        .and_then(|v| v.as_str().map(String::from))
        .ok_or("No stored PKCE verifier")?;

    // 4. Exchange authorization code for tokens (via backend)
    tracing::debug!("exchanging authorization code for tokens");
    let session =
        adapter_google_backend::exchange_code_for_tokens(&code, &code_verifier, REDIRECT_URI)
            .await
            .map_err(|e| e.to_string())?;

    // 5. Store session in Tauri store
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(session.access_token));
    store.set("refresh_token", serde_json::json!(session.refresh_token));
    store.set("id_token", serde_json::json!(session.id_token));
    store.set("expires_at", serde_json::json!(session.expires_at));
    store.set("user", serde_json::json!(session.user));

    // 6. Clean up temporary PKCE and state
    store.delete("pkce_verifier");
    store.delete("oauth_state");

    tracing::info!(email = %session.user.email, "OAuth login successful");
    Ok(session)
}

/// Get the current auth session from the Tauri store.
pub fn get_session(app: AppHandle) -> Result<Option<AuthSession>, String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;

    let Some(access_token) = store
        .get("access_token")
        .and_then(|v| v.as_str().map(String::from))
    else {
        return Ok(None);
    };

    let refresh_token = store
        .get("refresh_token")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let id_token = store
        .get("id_token")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let expires_at = store
        .get("expires_at")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let user = store
        .get("user")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    match user {
        Some(user) => Ok(Some(AuthSession {
            access_token,
            refresh_token,
            id_token,
            expires_at,
            user,
        })),
        None => Ok(None),
    }
}

/// Exchange refresh_token for new access_token (via backend).
pub async fn refresh_access_token(
    app: &AppHandle,
    refresh_token: &str,
) -> Result<(String, u64), String> {
    let (new_access_token, new_expires_at, new_refresh_token) =
        adapter_google_backend::refresh_access_token(refresh_token)
            .await
            .map_err(|e| e.to_string())?;

    // Update store with new tokens
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(new_access_token));
    store.set("expires_at", serde_json::json!(new_expires_at));

    // If Google rotated the refresh_token, update it too
    if let Some(new_refresh) = new_refresh_token {
        store.set("refresh_token", serde_json::json!(new_refresh));
    }

    Ok((new_access_token, new_expires_at))
}

/// Clear all auth tokens from store and emit expiry event to frontend.
pub fn clear_session_and_notify(app: &AppHandle) {
    let store = app.store("auth.json").ok();
    if let Some(ref store) = store {
        store.delete("access_token");
        store.delete("refresh_token");
        store.delete("id_token");
        store.delete("expires_at");
        store.delete("user");
    }
    app.emit("auth:expired", ()).ok();
}

/// Start background token refresh timer. Call from Tauri setup().
pub fn start_refresh_timer(app: AppHandle) {
    let store = match app.store("auth.json") {
        Ok(s) => s,
        Err(_) => return,
    };

    let Some(refresh_token) = store
        .get("refresh_token")
        .and_then(|v| v.as_str().map(String::from))
    else {
        return;
    };

    let Some(expires_at) = store.get("expires_at").and_then(|v| v.as_u64()) else {
        return;
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Calculate delay: refresh 5 minutes (300s) before expiry
    let delay_secs = if expires_at > now + 300 {
        expires_at - now - 300
    } else {
        0 // Refresh immediately or token already expired
    };

    if expires_at <= now {
        // Token already expired — clear immediately
        clear_session_and_notify(&app);
        return;
    }

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;

        match refresh_access_token(&app, &refresh_token).await {
            Ok((_new_token, new_expiry)) => {
                // Schedule next refresh for the new token's expiry
                let next_delay = new_expiry
                    - std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                    - 300;

                let app2 = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(next_delay)).await;
                    // Re-read refresh_token in case it was rotated
                    if let Ok(store) = app2.store("auth.json")
                        && let Some(rt) = store
                            .get("refresh_token")
                            .and_then(|v| v.as_str().map(String::from))
                        && refresh_access_token(&app2, &rt).await.is_err()
                    {
                        clear_session_and_notify(&app2);
                    }
                });
            }
            Err(_) => {
                clear_session_and_notify(&app);
            }
        }
    });
}

/// Quit the application.
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
