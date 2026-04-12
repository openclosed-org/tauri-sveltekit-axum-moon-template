//! Mock OAuth provider implementation for development/testing.

use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::domain::error::AuthError;
use crate::domain::token::TokenClaims;
use crate::ports::OAuthProvider;

/// Configuration for OAuth provider.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: "dev-client-id".to_string(),
            client_secret: "dev-client-secret".to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }
    }
}

/// Mock OAuth provider for development.
/// 
/// In production, replace with actual OAuth provider implementations
/// (Google, GitHub, Microsoft, etc.).
pub struct MockOAuthProvider {
    config: OAuthConfig,
}

impl MockOAuthProvider {
    pub fn new(config: OAuthConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: OAuthConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct OAuthState {
    state: String,
    timestamp: i64,
}

#[async_trait]
impl OAuthProvider for MockOAuthProvider {
    async fn get_auth_url(&self, state: &str) -> Result<String, AuthError> {
        let oauth_state = OAuthState {
            state: state.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let state_encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_string(&oauth_state).map_err(|e| {
                AuthError::OAuthError(format!("Failed to encode OAuth state: {e}"))
            })?);

        let mut url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&state={}",
            self.config.auth_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            state_encoded,
        );

        if !self.config.scopes.is_empty() {
            url.push_str(&format!(
                "&scope={}",
                urlencoding::encode(&self.config.scopes.join(" "))
            ));
        }

        Ok(url)
    }

    async fn exchange_code(&self, _code: &str) -> Result<TokenClaims, AuthError> {
        // Mock implementation - in production, this would:
        // 1. POST to token_url with code, client_id, client_secret, redirect_uri
        // 2. Parse response to get id_token/access_token
        // 3. Validate and decode token
        // 4. Extract user info and return as TokenClaims

        // For development, return a mock user
        Ok(TokenClaims {
            sub: "mock-user-sub-123".to_string(),
            user_id: "mock-user-id-456".to_string(),
            tenant_id: None,
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            roles: vec!["user".to_string()],
        })
    }

    async fn refresh_tokens(&self, _refresh_token: &str) -> Result<TokenClaims, AuthError> {
        // Mock implementation - in production, this would:
        // 1. POST to token_url with refresh_token, client_id, client_secret
        // 2. Parse response to get new tokens
        // 3. Validate and decode new access token
        // 4. Return as TokenClaims

        Ok(TokenClaims {
            sub: "mock-user-sub-123".to_string(),
            user_id: "mock-user-id-456".to_string(),
            tenant_id: None,
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            roles: vec!["user".to_string()],
        })
    }
}
