//! contracts/auth — Authentication and authorization DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// JWT token pair returned on successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// OAuth callback payload from provider.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// User profile information from OAuth provider.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    pub email: String,
    pub name: String,
    pub picture: String,
    pub sub: String,
}

/// Authenticated user session info.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSession {
    pub user_sub: String,
    pub tenant_id: String,
    pub role: String,
}
