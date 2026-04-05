//! adapter-google — Google OAuth PKCE adapter.
//!
//! Migrated from runtime_tauri/commands/auth.rs (422 lines).
//! Provides GoogleAuthAdapter struct wrapping all OAuth operations.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URI: &str = "http://127.0.0.1:1420/oauth/callback";
const SCOPES: &str = "openid email profile";

// ── Error type ──────────────────────────────────────────────────

/// Authentication error variants.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Invalid callback: {0}")]
    InvalidCallback(String),
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    #[error("Token expired: {0}")]
    TokenExpired(String),
}

impl From<AuthError> for String {
    fn from(e: AuthError) -> String {
        e.to_string()
    }
}

// ── Public types (Tauri store types — per D-03) ────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub email: String,
    pub name: String,
    pub picture: String,
    pub sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub expires_at: u64,
    pub user: UserProfile,
}

// ── Internal types ─────────────────────────────────────────────

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

// ── Config helpers ─────────────────────────────────────────────

fn client_id() -> String {
    oauth_env("GOOGLE_CLIENT_ID", "YOUR_GOOGLE_CLIENT_ID").unwrap_or_else(|error| {
        tracing::warn!(%error, "invalid GOOGLE_CLIENT_ID configuration");
        "YOUR_GOOGLE_CLIENT_ID".to_string()
    })
}

fn client_secret() -> String {
    oauth_env("GOOGLE_CLIENT_SECRET", "YOUR_GOOGLE_CLIENT_SECRET").unwrap_or_else(|error| {
        tracing::warn!(%error, "invalid GOOGLE_CLIENT_SECRET configuration");
        "YOUR_GOOGLE_CLIENT_SECRET".to_string()
    })
}

fn oauth_env(key: &'static str, placeholder: &'static str) -> Result<String, String> {
    let raw = std::env::var(key).unwrap_or_default();
    let mut normalized = raw.trim().to_string();

    if normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_string();
        tracing::warn!(%key, "trimmed trailing semicolon from OAuth env var");
    }

    let has_double_quotes = normalized.starts_with('"') && normalized.ends_with('"');
    let has_single_quotes = normalized.starts_with('\'') && normalized.ends_with('\'');

    if has_double_quotes || has_single_quotes {
        normalized = normalized[1..normalized.len().saturating_sub(1)].to_string();
        tracing::warn!(%key, "trimmed wrapping quotes from OAuth env var");
    }

    if normalized.is_empty() || normalized == placeholder {
        return Err(format!("{key} is missing or still using placeholder value"));
    }

    Ok(normalized)
}

// ── GoogleAuthAdapter ──────────────────────────────────────────

/// Google OAuth PKCE adapter.
///
/// Wraps all OAuth operations into a cohesive adapter struct,
/// extracted from the monolithic Tauri command handlers.
pub struct GoogleAuthAdapter;

impl GoogleAuthAdapter {
    /// Create a new GoogleAuthAdapter instance.
    pub fn new() -> Self {
        Self
    }

    /// Start the Google OAuth PKCE login flow.
    ///
    /// 1. Generates PKCE code_verifier + code_challenge
    /// 2. Generates CSRF state parameter
    /// 3. Stores verifier/state in Tauri store
    /// 4. Starts TCP listener for redirect callback
    /// 5. Opens system browser with authorization URL
    pub async fn start_login(&self, app: &AppHandle) -> Result<(), AuthError> {
        start_oauth(app.clone()).await.map_err(AuthError::Network)
    }

    /// Handle the OAuth callback URL.
    ///
    /// Validates state, exchanges code for tokens, decodes user profile.
    pub async fn handle_callback(
        &self,
        app: &AppHandle,
        url: &str,
    ) -> Result<AuthSession, AuthError> {
        handle_oauth_callback(app.clone(), url.to_string())
            .await
            .map_err(AuthError::TokenExchange)
    }

    /// Get the current auth session from the Tauri store.
    pub fn get_session(&self, app: &AppHandle) -> Result<Option<AuthSession>, String> {
        get_session(app.clone())
    }

    /// Refresh the access token using the stored refresh token.
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

// ── Core OAuth logic (migrated from runtime_tauri/commands/auth.rs) ─

/// Start the Google OAuth PKCE login flow.
pub async fn start_oauth(app: AppHandle) -> Result<(), String> {
    // 1. Generate PKCE code_verifier (64 random alphanumeric chars)
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    // 2. Compute code_challenge = base64url(SHA256(code_verifier))
    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    // 3. Generate state parameter for CSRF protection
    let state: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // 4. Store code_verifier and state in app store for callback validation
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("pkce_verifier", serde_json::json!(code_verifier));
    store.set("oauth_state", serde_json::json!(state));

    // 5. Start one-shot HTTP listener on 127.0.0.1:1420 to catch Google's redirect
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

    // 6. Build authorization URL
    let url = format!(
        "{GOOGLE_AUTH_URL}?client_id={}&redirect_uri={REDIRECT_URI}&response_type=code&scope={SCOPES}&code_challenge={code_challenge}&code_challenge_method=S256&state={state}&access_type=offline&prompt=consent",
        client_id()
    );

    // 7. Open system browser
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

    // 4. Exchange authorization code for tokens
    tracing::debug!("exchanging authorization code for tokens");
    let client = reqwest::Client::new();
    let cid = client_id();
    let csec = client_secret();
    let params = [
        ("code", code.as_str()),
        ("client_id", cid.as_str()),
        ("client_secret", csec.as_str()),
        ("redirect_uri", REDIRECT_URI),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier.as_str()),
    ];

    let resp = client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        tracing::error!(%body, "token exchange failed");
        return Err(format!("Token exchange failed: {body}"));
    }

    let token_resp: TokenResponse = resp.json().await.map_err(|e| e.to_string())?;
    tracing::debug!("token exchange successful, decoding id_token");

    // 5. Decode id_token to extract user profile (JWT payload only, signature verification deferred to v2)
    let id_token_parts: Vec<&str> = token_resp.id_token.split('.').collect();
    if id_token_parts.len() < 2 {
        return Err("Invalid id_token format".into());
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(id_token_parts[1])
        .map_err(|e| format!("Failed to decode id_token payload: {e}"))?;
    let user: UserProfile = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Failed to parse user profile: {e}"))?;

    // 6. Calculate expiry timestamp
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + token_resp.expires_in;

    // 7. Build session and store it
    let session = AuthSession {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token.unwrap_or_default(),
        id_token: token_resp.id_token,
        expires_at,
        user,
    };

    store.set("access_token", serde_json::json!(session.access_token));
    store.set("refresh_token", serde_json::json!(session.refresh_token));
    store.set("id_token", serde_json::json!(session.id_token));
    store.set("expires_at", serde_json::json!(session.expires_at));
    store.set("user", serde_json::json!(session.user));

    // 8. Clean up temporary PKCE and state
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

/// Exchange refresh_token for new access_token.
pub async fn refresh_access_token(
    app: &AppHandle,
    refresh_token: &str,
) -> Result<(String, u64), String> {
    let client = reqwest::Client::new();
    let client_id = client_id();
    let client_secret = client_secret();
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Refresh token rejected: {body}"));
    }

    let refresh: RefreshResponse = resp.json().await.map_err(|e| e.to_string())?;
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + refresh.expires_in;

    // Update store with new access_token and expiry
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(refresh.access_token));
    store.set("expires_at", serde_json::json!(expires_at));

    // If Google rotated the refresh_token, update it too
    if let Some(new_refresh) = refresh.refresh_token {
        store.set("refresh_token", serde_json::json!(new_refresh));
    }

    Ok((refresh.access_token, expires_at))
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
