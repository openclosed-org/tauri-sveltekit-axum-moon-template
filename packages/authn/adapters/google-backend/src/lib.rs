//! adapter-google-backend — legacy Google OAuth PKCE client adapter (no Tauri dependencies).
//!
//! This crate is an optional authorization-code login client lane. It is not the
//! backend resource-server token verifier used by web-bff. Generic OIDC bearer
//! token verification lives in `authn-oidc-verifier`.
//!
//! Provides Google-specific OAuth operations:
//! - PKCE code verifier/challenge generation
//! - Authorization URL construction
//! - Token exchange
//! - Session management
//! - Token refresh
//!
//! This crate contains NO Tauri dependencies, but it should not be used as a
//! server-side trust boundary without adding id_token signature validation.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

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

// ── Public types ────────────────────────────────────────────────

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

// ── PKCE helpers ────────────────────────────────────────────────

/// Generate PKCE code verifier (64 random alphanumeric chars).
pub fn generate_code_verifier() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// Compute code_challenge = base64url(SHA256(code_verifier)).
pub fn compute_code_challenge(code_verifier: &str) -> String {
    let hash = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate random state parameter for CSRF protection.
pub fn generate_state() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
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

// ── Authorization URL construction ─────────────────────────────

/// Build Google OAuth authorization URL.
pub fn build_authorization_url(code_challenge: &str, state: &str, redirect_uri: &str) -> String {
    format!(
        "{GOOGLE_AUTH_URL}?client_id={}&redirect_uri={redirect_uri}&response_type=code&scope=openid%20email%20profile&code_challenge={code_challenge}&code_challenge_method=S256&state={state}&access_type=offline&prompt=consent",
        client_id()
    )
}

// ── Token exchange ─────────────────────────────────────────────

/// Exchange authorization code for tokens.
pub async fn exchange_code_for_tokens(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<AuthSession, AuthError> {
    let client = reqwest::Client::new();
    let cid = client_id();
    let csec = client_secret();
    let params = [
        ("code", code),
        ("client_id", cid.as_str()),
        ("client_secret", csec.as_str()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AuthError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        tracing::error!(%body, "token exchange failed");
        return Err(AuthError::TokenExchange(format!(
            "Token exchange failed: {body}"
        )));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

    tracing::debug!("token exchange successful, decoding id_token");

    // Legacy client-lane decode only: extracts display profile from JWT payload.
    // This is not server-side identity verification; use authn-oidc-verifier for that boundary.
    let id_token_parts: Vec<&str> = token_resp.id_token.split('.').collect();
    if id_token_parts.len() < 2 {
        return Err(AuthError::TokenExchange("Invalid id_token format".into()));
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(id_token_parts[1])
        .map_err(|e| AuthError::TokenExchange(format!("Failed to decode id_token payload: {e}")))?;
    let user: UserProfile = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AuthError::TokenExchange(format!("Failed to parse user profile: {e}")))?;

    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + token_resp.expires_in;

    Ok(AuthSession {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token.unwrap_or_default(),
        id_token: token_resp.id_token,
        expires_at,
        user,
    })
}

/// Exchange refresh token for new access token.
pub async fn refresh_access_token(
    refresh_token: &str,
) -> Result<(String, u64, Option<String>), AuthError> {
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
        .map_err(|e| AuthError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AuthError::TokenExchange(format!(
            "Refresh token rejected: {body}"
        )));
    }

    let refresh: RefreshResponse = resp
        .json()
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + refresh.expires_in;

    Ok((refresh.access_token, expires_at, refresh.refresh_token))
}

// ── Session validation ─────────────────────────────────────────

/// Check if session is still valid.
pub fn is_session_valid(session: &AuthSession) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    session.expires_at > now
}

/// Calculate delay before token refresh (5 minutes before expiry).
pub fn calculate_refresh_delay(session: &AuthSession) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if session.expires_at > now + 300 {
        session.expires_at - now - 300
    } else {
        0
    }
}
