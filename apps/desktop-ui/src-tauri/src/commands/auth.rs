use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const CLIENT_ID: &str = option_env!("GOOGLE_CLIENT_ID").unwrap_or("YOUR_GOOGLE_CLIENT_ID");
const CLIENT_SECRET: &str =
    option_env!("GOOGLE_CLIENT_SECRET").unwrap_or("YOUR_GOOGLE_CLIENT_SECRET");
const REDIRECT_URI: &str = "com.example.app://oauth/callback";
const SCOPES: &str = "openid email profile";

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

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: String,
    expires_in: u64,
}

#[tauri::command]
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

    // 5. Build authorization URL
    let url = format!(
        "{GOOGLE_AUTH_URL}?client_id={CLIENT_ID}&redirect_uri={REDIRECT_URI}&response_type=code&scope={SCOPES}&code_challenge={code_challenge}&code_challenge_method=S256&state={state}&access_type=offline&prompt=consent"
    );

    // 6. Open system browser
    app.shell().open(&url, None).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn handle_oauth_callback(app: AppHandle, url: String) -> Result<AuthSession, String> {
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
        return Err("OAuth state mismatch — possible CSRF attack".into());
    }

    // 3. Retrieve stored PKCE verifier
    let code_verifier = store
        .get("pkce_verifier")
        .and_then(|v| v.as_str().map(String::from))
        .ok_or("No stored PKCE verifier")?;

    // 4. Exchange authorization code for tokens
    let client = reqwest::Client::new();
    let params = [
        ("code", code.as_str()),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
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
        return Err(format!("Token exchange failed: {body}"));
    }

    let token_resp: TokenResponse = resp.json().await.map_err(|e| e.to_string())?;

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

    Ok(session)
}

#[tauri::command]
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
