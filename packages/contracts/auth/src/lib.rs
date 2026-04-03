//! contracts/auth — Authentication and authorization DTOs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// JWT token pair returned on successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "auth/")]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    #[ts(type = "number")]
    pub expires_in: i64,
}

/// OAuth callback payload from provider.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "auth/")]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// Authenticated user session info.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "auth/")]
pub struct UserSession {
    pub user_sub: String,
    pub tenant_id: String,
    pub role: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_token_pair() {
        TokenPair::export().unwrap();
    }

    #[test]
    fn export_oauth_callback() {
        OAuthCallback::export().unwrap();
    }

    #[test]
    fn export_user_session() {
        UserSession::export().unwrap();
    }
}
