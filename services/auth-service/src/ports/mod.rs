//! Auth service ports — external dependency abstractions.

use async_trait::async_trait;

use crate::domain::error::AuthError;
use crate::domain::session::Session;
use crate::domain::token::{TokenClaims, TokenPair};

/// Repository port for session management.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session.
    async fn create_session(&self, session: &Session) -> Result<(), AuthError>;

    /// Get a session by ID.
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AuthError>;

    /// Delete a session.
    async fn delete_session(&self, session_id: &str) -> Result<(), AuthError>;

    /// Update session's last accessed time.
    async fn touch_session(&self, session_id: &str) -> Result<(), AuthError>;
}

/// Repository port for token management.
#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Generate a token pair (access + refresh).
    async fn generate_tokens(&self, claims: &TokenClaims) -> Result<TokenPair, AuthError>;

    /// Validate and decode an access token.
    async fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthError>;

    /// Validate and decode a refresh token.
    async fn validate_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthError>;

    /// Revoke a refresh token.
    async fn revoke_refresh_token(&self, token: &str) -> Result<(), AuthError>;
}

/// Port for OAuth provider interactions.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Get authorization URL for OAuth flow.
    async fn get_auth_url(&self, state: &str) -> Result<String, AuthError>;

    /// Exchange authorization code for tokens.
    async fn exchange_code(&self, code: &str) -> Result<TokenClaims, AuthError>;

    /// Refresh tokens using refresh token.
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenClaims, AuthError>;
}
