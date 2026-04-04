use adapter_google::GoogleAuthAdapter;
use async_trait::async_trait;
use feature_auth::AuthService;
use feature_auth::{AuthError as FeatureAuthError, AuthResult, SessionInfo, UserProfile};
use tauri::AppHandle;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthSession {
    pub tokens: contracts_auth::TokenPair,
    pub id_token: String,
    pub user: UserProfile,
}

struct TauriAuthService {
    adapter: GoogleAuthAdapter,
    app: AppHandle,
}

impl TauriAuthService {
    fn new(app: AppHandle) -> Self {
        Self {
            adapter: GoogleAuthAdapter::new(),
            app,
        }
    }

    fn convert_session(session: adapter_google::AuthSession) -> AuthSession {
        AuthSession {
            tokens: contracts_auth::TokenPair {
                access_token: session.access_token,
                refresh_token: session.refresh_token,
                expires_in: i64::try_from(session.expires_at).unwrap_or(0),
            },
            id_token: session.id_token,
            user: UserProfile {
                email: session.user.email,
                name: session.user.name,
                picture: session.user.picture,
                sub: session.user.sub,
            },
        }
    }

    async fn get_runtime_session(&self) -> Result<Option<AuthSession>, FeatureAuthError> {
        let session = self
            .adapter
            .get_session(&self.app)
            .map_err(FeatureAuthError::Database)?;

        Ok(session.map(Self::convert_session))
    }
}

#[async_trait]
impl feature_auth::AuthService for TauriAuthService {
    async fn start_login(&self) -> Result<(), FeatureAuthError> {
        self.adapter
            .start_login(&self.app)
            .await
            .map_err(|e| FeatureAuthError::Network(e.to_string()))
    }

    async fn handle_callback(&self, url: &str) -> Result<AuthResult, FeatureAuthError> {
        let session = self
            .adapter
            .handle_callback(&self.app, url)
            .await
            .map_err(|e| FeatureAuthError::TokenExchange(e.to_string()))?;

        Ok(AuthResult {
            user: UserProfile {
                email: session.user.email,
                name: session.user.name,
                picture: session.user.picture,
                sub: session.user.sub,
            },
            tokens: contracts_auth::TokenPair {
                access_token: session.access_token,
                refresh_token: session.refresh_token,
                expires_in: i64::try_from(session.expires_at).map_err(|_| {
                    FeatureAuthError::Config("Invalid expires_at value".to_string())
                })?,
            },
        })
    }

    async fn get_session(&self) -> Result<Option<SessionInfo>, FeatureAuthError> {
        let session = self
            .adapter
            .get_session(&self.app)
            .map_err(FeatureAuthError::Database)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| FeatureAuthError::Config(e.to_string()))?
            .as_secs();

        Ok(session.map(|s| SessionInfo {
            user: UserProfile {
                email: s.user.email,
                name: s.user.name,
                picture: s.user.picture,
                sub: s.user.sub,
            },
            expires_at: s.expires_at,
            is_valid: s.expires_at > now,
        }))
    }

    async fn logout(&self) -> Result<(), FeatureAuthError> {
        self.adapter.clear_session(&self.app);
        Ok(())
    }
}

#[tauri::command]
pub async fn start_oauth(app: AppHandle) -> Result<(), String> {
    let service = TauriAuthService::new(app);
    service.start_login().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn handle_oauth_callback(app: AppHandle, url: String) -> Result<AuthSession, String> {
    let service = TauriAuthService::new(app);
    service
        .handle_callback(&url)
        .await
        .map_err(|e| e.to_string())?;

    service
        .get_runtime_session()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())
}

#[tauri::command]
pub async fn get_session(app: AppHandle) -> Result<Option<AuthSession>, String> {
    let service = TauriAuthService::new(app);
    service
        .get_runtime_session()
        .await
        .map_err(|e| e.to_string())
}

pub fn start_refresh_timer(app: AppHandle) {
    let adapter = GoogleAuthAdapter::new();
    adapter.start_timer(app);
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    adapter_google::quit_app(app);
}
