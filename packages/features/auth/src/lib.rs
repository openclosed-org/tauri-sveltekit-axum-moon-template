//! feature-auth — Authentication feature crate.
//!
//! Defines the AuthService trait and supporting types.
//! Hexagonal boundary: depends on domain + usecases + contracts_auth,
//! NOT on any adapter (adapter-google, etc.).

use async_trait::async_trait;
use contracts_auth::TokenPair;

// ── Error type ──────────────────────────────────────────────────

/// Authentication error variants for the feature layer.
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
    #[error("Session expired: {0}")]
    SessionExpired(String),
    #[error("Database error: {0}")]
    Database(String),
}

// ── Value types ─────────────────────────────────────────────────

/// User profile from the OAuth provider.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub email: String,
    pub name: String,
    pub picture: String,
    pub sub: String,
}

/// Result of a successful authentication.
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user: UserProfile,
    pub tokens: TokenPair,
}

/// Session information for the authenticated user.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user: UserProfile,
    pub expires_at: u64,
    pub is_valid: bool,
}

// ── AuthService trait ───────────────────────────────────────────

/// Core authentication service trait.
///
/// Defines the contract that any auth adapter must implement.
/// This trait is adapter-agnostic — implementations may use Google,
/// GitHub, email/password, or any other provider.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Initiate the login flow (e.g., open browser for OAuth).
    async fn start_login(&self) -> Result<(), AuthError>;

    /// Handle the callback from the auth provider.
    async fn handle_callback(&self, url: &str) -> Result<AuthResult, AuthError>;

    /// Get the current session, if any.
    async fn get_session(&self) -> Result<Option<SessionInfo>, AuthError>;

    /// Log out and clear all session data.
    async fn logout(&self) -> Result<(), AuthError>;
}

// ── Stub: CounterServiceProxy ───────────────────────────────────

/// Stub for admin/agent counter service proxy.
/// Will be implemented when counter feature is built.
pub struct CounterServiceProxy;
