//! Auth domain errors.

/// Authentication domain errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
}
