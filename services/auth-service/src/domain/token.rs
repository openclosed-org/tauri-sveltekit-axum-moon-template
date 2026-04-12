//! Token entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JWT token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub roles: Vec<String>,
}

/// Token pair — access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub expires_at: DateTime<Utc>,
}
