//! Auth service — orchestrates authentication use cases via ports.

use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::domain::error::AuthError;
use crate::domain::session::Session;
use crate::domain::token::TokenPair;
use crate::ports::{OAuthProvider, SessionRepository, TokenRepository};

/// Input for authentication.
#[derive(Debug, Clone)]
pub struct AuthInput {
    pub user_id: String,
    pub user_sub: String,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Result of authentication.
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub session_id: String,
    pub tokens: TokenPair,
    pub expires_at: chrono::DateTime<Utc>,
}

/// Auth service trait — the application-layer contract.
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    /// Authenticate user and create session.
    async fn authenticate(&self, input: AuthInput) -> Result<AuthResult, AuthError>;

    /// Validate access token and return session.
    async fn validate_session(&self, access_token: &str) -> Result<Session, AuthError>;

    /// Refresh tokens using refresh token.
    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResult, AuthError>;

    /// Logout and invalidate session.
    async fn logout(&self, session_id: &str) -> Result<(), AuthError>;

    /// Get OAuth authorization URL.
    async fn get_oauth_url(&self) -> Result<String, AuthError>;

    /// Complete OAuth flow with authorization code.
    async fn complete_oauth(&self, code: &str) -> Result<AuthResult, AuthError>;
}

/// Concrete auth service implementation.
pub struct AuthService<S, T, O>
where
    S: SessionRepository,
    T: TokenRepository,
    O: OAuthProvider,
{
    session_repo: S,
    token_repo: T,
    oauth_provider: O,
    session_ttl: Duration,
}

impl<S, T, O> AuthService<S, T, O>
where
    S: SessionRepository,
    T: TokenRepository,
    O: OAuthProvider,
{
    pub fn new(session_repo: S, token_repo: T, oauth_provider: O) -> Self {
        Self {
            session_repo,
            token_repo,
            oauth_provider,
            session_ttl: Duration::hours(24), // Default 24-hour session
        }
    }

    /// Set custom session TTL.
    pub fn with_session_ttl(mut self, ttl: Duration) -> Self {
        self.session_ttl = ttl;
        self
    }
}

#[async_trait]
impl<S, T, O> AuthServiceTrait for AuthService<S, T, O>
where
    S: SessionRepository,
    T: TokenRepository,
    O: OAuthProvider,
{
    async fn authenticate(&self, input: AuthInput) -> Result<AuthResult, AuthError> {
        let now = Utc::now();
        let expires_at = now + self.session_ttl;

        // Create session
        let session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: input.user_id.clone(),
            user_sub: input.user_sub.clone(),
            tenant_id: input.tenant_id.clone(),
            expires_at,
            created_at: now,
            last_accessed_at: now,
            ip_address: input.ip_address.clone(),
            user_agent: input.user_agent.clone(),
        };

        self.session_repo.create_session(&session).await?;

        // Generate tokens
        let claims = crate::domain::token::TokenClaims {
            sub: input.user_sub,
            user_id: input.user_id,
            tenant_id: input.tenant_id,
            exp: (expires_at.timestamp() as usize),
            iat: (now.timestamp() as usize),
            roles: input.roles,
        };

        let tokens = self.token_repo.generate_tokens(&claims).await?;

        Ok(AuthResult {
            session_id: session.id,
            tokens,
            expires_at,
        })
    }

    async fn validate_session(&self, access_token: &str) -> Result<Session, AuthError> {
        // Validate token
        let claims = self.token_repo.validate_access_token(access_token).await?;

        // Check if token is expired
        let exp = claims.exp as i64;
        let now = Utc::now().timestamp();
        if now > exp {
            return Err(AuthError::TokenExpired);
        }

        // Get session (we store session_id in token sub field)
        let session = self.session_repo.get_session(&claims.sub).await?;

        match session {
            Some(s) if !s.is_expired() => Ok(s),
            Some(_) => Err(AuthError::SessionExpired),
            None => Err(AuthError::SessionNotFound(claims.sub)),
        }
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResult, AuthError> {
        // Validate refresh token
        let claims = self.token_repo.validate_refresh_token(refresh_token).await?;

        // Generate new tokens
        let now = Utc::now();
        let expires_at = now + self.session_ttl;

        let new_claims = crate::domain::token::TokenClaims {
            sub: claims.sub.clone(),
            user_id: claims.user_id.clone(),
            tenant_id: claims.tenant_id.clone(),
            exp: (expires_at.timestamp() as usize),
            iat: (now.timestamp() as usize),
            roles: claims.roles,
        };

        let tokens = self.token_repo.generate_tokens(&new_claims).await?;

        // Update session
        self.session_repo.touch_session(&claims.sub).await?;

        Ok(AuthResult {
            session_id: claims.sub,
            tokens,
            expires_at,
        })
    }

    async fn logout(&self, session_id: &str) -> Result<(), AuthError> {
        self.session_repo.delete_session(session_id).await
    }

    async fn get_oauth_url(&self) -> Result<String, AuthError> {
        let state = uuid::Uuid::new_v4().to_string();
        self.oauth_provider.get_auth_url(&state).await
    }

    async fn complete_oauth(&self, code: &str) -> Result<AuthResult, AuthError> {
        // Exchange code for tokens
        let claims = self.oauth_provider.exchange_code(code).await?;

        let now = Utc::now();
        let expires_at = now + self.session_ttl;

        // Create session
        let session = Session {
            id: claims.sub.clone(),
            user_id: claims.user_id.clone(),
            user_sub: claims.sub.clone(),
            tenant_id: claims.tenant_id.clone(),
            expires_at,
            created_at: now,
            last_accessed_at: now,
            ip_address: None,
            user_agent: None,
        };

        self.session_repo.create_session(&session).await?;

        // Generate token pair
        let tokens = self.token_repo.generate_tokens(&claims).await?;

        Ok(AuthResult {
            session_id: session.id,
            tokens,
            expires_at,
        })
    }
}
