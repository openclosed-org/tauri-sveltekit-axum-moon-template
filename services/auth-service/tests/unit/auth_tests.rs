//! Auth service unit tests.

#[cfg(test)]
mod auth_tests {
    use chrono::{Duration, Utc};

    use crate::application::{AuthService, AuthServiceTrait, AuthInput};
    use crate::domain::error::AuthError;
    use crate::domain::session::Session;
    use crate::domain::token::{TokenClaims, TokenPair};
    use crate::ports::{OAuthProvider, SessionRepository, TokenRepository};

    /// Mock session repository for testing.
    struct MockSessionRepository {
        sessions: std::sync::Mutex<Vec<Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn create_session(&self, session: &Session) -> Result<(), AuthError> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.push(session.clone());
            Ok(())
        }

        async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AuthError> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.iter().find(|s| s.id == session_id).cloned())
        }

        async fn delete_session(&self, session_id: &str) -> Result<(), AuthError> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.retain(|s| s.id != session_id);
            Ok(())
        }

        async fn touch_session(&self, session_id: &str) -> Result<(), AuthError> {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
                session.touch();
            }
            Ok(())
        }
    }

    /// Mock token repository for testing.
    struct MockTokenRepository;

    #[async_trait::async_trait]
    impl TokenRepository for MockTokenRepository {
        async fn generate_tokens(&self, claims: &TokenClaims) -> Result<TokenPair, AuthError> {
            let now = chrono::Utc::now();
            let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap_or(now);

            Ok(TokenPair {
                access_token: format!("access-{}", claims.sub),
                refresh_token: format!("refresh-{}", claims.sub),
                token_type: "Bearer".to_string(),
                expires_in: (expires_at - now).num_seconds(),
                expires_at,
            })
        }

        async fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
            // Simple mock validation
            if token.starts_with("access-") {
                let sub = token.trim_start_matches("access-").to_string();
                Ok(TokenClaims {
                    sub: sub.clone(),
                    user_id: sub,
                    tenant_id: None,
                    exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
                    iat: Utc::now().timestamp() as usize,
                    roles: vec!["user".to_string()],
                })
            } else {
                Err(AuthError::InvalidToken("invalid access token".to_string()))
            }
        }

        async fn validate_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
            // Simple mock validation
            if token.starts_with("refresh-") {
                let sub = token.trim_start_matches("refresh-").to_string();
                Ok(TokenClaims {
                    sub: sub.clone(),
                    user_id: sub,
                    tenant_id: None,
                    exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
                    iat: Utc::now().timestamp() as usize,
                    roles: vec!["user".to_string()],
                })
            } else {
                Err(AuthError::InvalidToken("invalid refresh token".to_string()))
            }
        }

        async fn revoke_refresh_token(&self, _token: &str) -> Result<(), AuthError> {
            Ok(())
        }
    }

    /// Mock OAuth provider for testing.
    struct MockOAuthProvider;

    #[async_trait::async_trait]
    impl OAuthProvider for MockOAuthProvider {
        async fn get_auth_url(&self, state: &str) -> Result<String, AuthError> {
            Ok(format!("https://oauth.example.com/authorize?state={}", state))
        }

        async fn exchange_code(&self, _code: &str) -> Result<TokenClaims, AuthError> {
            Ok(TokenClaims {
                sub: "oauth-user-123".to_string(),
                user_id: "user-123".to_string(),
                tenant_id: None,
                exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                roles: vec!["user".to_string()],
            })
        }

        async fn refresh_tokens(&self, _refresh_token: &str) -> Result<TokenClaims, AuthError> {
            Ok(TokenClaims {
                sub: "oauth-user-123".to_string(),
                user_id: "user-123".to_string(),
                tenant_id: None,
                exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                roles: vec!["user".to_string()],
            })
        }
    }

    #[tokio::test]
    async fn test_authenticate_user() {
        let session_repo = MockSessionRepository::new();
        let token_repo = MockTokenRepository;
        let oauth_provider = MockOAuthProvider;

        let service = AuthService::new(session_repo, token_repo, oauth_provider);

        let input = AuthInput {
            user_id: "user-123".to_string(),
            user_sub: "sub-123".to_string(),
            tenant_id: Some("tenant-123".to_string()),
            roles: vec!["user".to_string()],
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test".to_string()),
        };

        let result = service.authenticate(input).await.unwrap();
        assert!(!result.session_id.is_empty());
        assert!(!result.tokens.access_token.is_empty());
        assert!(!result.tokens.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_get_oauth_url() {
        let session_repo = MockSessionRepository::new();
        let token_repo = MockTokenRepository;
        let oauth_provider = MockOAuthProvider;

        let service = AuthService::new(session_repo, token_repo, oauth_provider);

        let url = service.get_oauth_url().await.unwrap();
        assert!(url.contains("https://oauth.example.com/authorize"));
        assert!(url.contains("state="));
    }

    #[tokio::test]
    async fn test_complete_oauth() {
        let session_repo = MockSessionRepository::new();
        let token_repo = MockTokenRepository;
        let oauth_provider = MockOAuthProvider;

        let service = AuthService::new(session_repo, token_repo, oauth_provider);

        let result = service.complete_oauth("auth-code-123").await.unwrap();
        assert!(!result.session_id.is_empty());
        assert!(!result.tokens.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_logout() {
        let session_repo = MockSessionRepository::new();
        let token_repo = MockTokenRepository;
        let oauth_provider = MockOAuthProvider;

        let service = AuthService::new(session_repo, token_repo, oauth_provider);

        // Authenticate first
        let input = AuthInput {
            user_id: "user-456".to_string(),
            user_sub: "sub-456".to_string(),
            tenant_id: None,
            roles: vec!["user".to_string()],
            ip_address: None,
            user_agent: None,
        };

        let auth_result = service.authenticate(input).await.unwrap();
        let session_id = auth_result.session_id.clone();

        // Then logout
        service.logout(&session_id).await.unwrap();

        // Verify session is deleted
        let session = session_repo.get_session(&session_id).await.unwrap();
        assert!(session.is_none());
    }
}
