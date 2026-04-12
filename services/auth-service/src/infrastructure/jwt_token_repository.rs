//! JWT-based token repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};

use crate::domain::error::AuthError;
use crate::domain::token::{TokenClaims, TokenPair};
use crate::ports::TokenRepository;

/// JWT-based token repository.
pub struct JwtTokenRepository {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_ttl: Duration,
    refresh_token_ttl: Duration,
}

impl JwtTokenRepository {
    /// Create a new JWT token repository.
    /// 
    /// # Arguments
    /// * `secret` - Secret key for signing JWTs (should be at least 32 bytes)
    /// * `access_token_ttl` - Access token time-to-live
    /// * `refresh_token_ttl` - Refresh token time-to-live
    pub fn new(
        secret: &str,
        access_token_ttl: Duration,
        refresh_token_ttl: Duration,
    ) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        Self {
            encoding_key,
            decoding_key,
            access_token_ttl,
            refresh_token_ttl,
        }
    }

    /// Create with default TTLs (15min access, 7d refresh).
    pub fn with_default_ttl(secret: &str) -> Self {
        Self::new(
            secret,
            Duration::minutes(15),
            Duration::days(7),
        )
    }

    /// Hash a refresh token for storage/comparison.
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl TokenRepository for JwtTokenRepository {
    async fn generate_tokens(&self, claims: &TokenClaims) -> Result<TokenPair, AuthError> {
        let now = Utc::now();

        // Generate access token
        let access_exp = (now + self.access_token_ttl).timestamp() as usize;
        let access_claims = serde_json::json!({
            "sub": claims.sub,
            "user_id": claims.user_id,
            "tenant_id": claims.tenant_id,
            "exp": access_exp,
            "iat": now.timestamp() as usize,
            "roles": claims.roles,
            "type": "access",
        });

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &self.encoding_key,
        )
        .map_err(|e| AuthError::TokenGenerationFailed(format!("Failed to encode access token: {e}")))?;

        // Generate refresh token (includes hash for validation)
        let refresh_exp = (now + self.refresh_token_ttl).timestamp() as usize;
        let refresh_claims = serde_json::json!({
            "sub": claims.sub,
            "user_id": claims.user_id,
            "tenant_id": claims.tenant_id,
            "exp": refresh_exp,
            "iat": now.timestamp() as usize,
            "roles": claims.roles,
            "type": "refresh",
        });

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &self.encoding_key,
        )
        .map_err(|e| AuthError::TokenGenerationFailed(format!("Failed to encode refresh token: {e}")))?;

        let expires_at = now + self.access_token_ttl;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_ttl.num_seconds(),
            expires_at,
        })
    }

    async fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let token_data = decode::<serde_json::Value>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map_err(|e| AuthError::InvalidToken(format!("Invalid access token: {e}")))?;

        // Verify it's an access token
        let token_type = token_data
            .claims
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InvalidToken("Missing token type".to_string()))?;

        if token_type != "access" {
            return Err(AuthError::InvalidToken("Not an access token".to_string()));
        }

        let claims = token_data.claims;
        Ok(TokenClaims {
            sub: claims["sub"].as_str().unwrap_or_default().to_string(),
            user_id: claims["user_id"].as_str().unwrap_or_default().to_string(),
            tenant_id: claims["tenant_id"].as_str().map(String::from),
            exp: claims["exp"].as_u64().unwrap_or_default() as usize,
            iat: claims["iat"].as_u64().unwrap_or_default() as usize,
            roles: claims["roles"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn validate_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let token_data = decode::<serde_json::Value>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map_err(|e| AuthError::InvalidToken(format!("Invalid refresh token: {e}")))?;

        // Verify it's a refresh token
        let token_type = token_data
            .claims
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InvalidToken("Missing token type".to_string()))?;

        if token_type != "refresh" {
            return Err(AuthError::InvalidToken("Not a refresh token".to_string()));
        }

        let claims = token_data.claims;
        Ok(TokenClaims {
            sub: claims["sub"].as_str().unwrap_or_default().to_string(),
            user_id: claims["user_id"].as_str().unwrap_or_default().to_string(),
            tenant_id: claims["tenant_id"].as_str().map(String::from),
            exp: claims["exp"].as_u64().unwrap_or_default() as usize,
            iat: claims["iat"].as_u64().unwrap_or_default() as usize,
            roles: claims["roles"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn revoke_refresh_token(&self, _token: &str) -> Result<(), AuthError> {
        // JWTs are stateless, so we can't truly revoke them without a token blacklist.
        // In production, you'd store hashed tokens in a revocation table.
        // For now, we rely on short TTLs and session deletion.
        // This is a placeholder for future token blacklist implementation.
        let _hash = Self::hash_token(_token);
        // TODO: Insert hash into revocation table
        Ok(())
    }
}
