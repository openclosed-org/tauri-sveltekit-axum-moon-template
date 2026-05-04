//! contracts/api — Route-level shared DTOs.
//! These Rust DTOs are the schema source for generated OpenAPI artifacts.

#![deny(unused_imports, unused_variables)]

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::fmt;
use utoipa::ToSchema;
use validator::Validate;

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Server status: "ok" or "degraded"
    pub status: String,
}

impl HealthResponse {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}

/// Request body for tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct InitTenantRequest {
    /// OAuth provider's subject identifier.
    #[validate(length(min = 1, message = "user_sub is required"))]
    pub user_sub: String,
    /// Display name for the user.
    #[validate(length(min = 1, max = 100))]
    pub user_name: String,
}

/// Response from tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitTenantResponse {
    /// The tenant ID in "table:key" format.
    pub tenant_id: String,
    /// User's role within the tenant.
    pub role: String,
    /// Whether a new tenant was created.
    pub created: bool,
}

impl InitTenantResponse {
    pub fn new(tenant_id: impl Into<String>, role: impl Into<String>, created: bool) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            role: role.into(),
            created,
        }
    }
}

/// Chat message (user or assistant).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    /// "user" | "assistant" | "system" | "tool"
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub created_at: String,
}

/// Tool call in a chat message.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<String>,
}

/// Agent configuration (user-provided API key + endpoint).
#[derive(Clone, Deserialize, ToSchema)]
pub struct AgentConfig {
    /// API key — omitted from serialized payloads to prevent accidental exposure.
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl Serialize for AgentConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AgentConfig", 2)?;
        state.serialize_field("base_url", &self.base_url)?;
        state.serialize_field("model", &self.model)?;
        state.end()
    }
}

impl fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentConfig")
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .finish()
    }
}

/// Admin dashboard statistics.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminDashboardStats {
    pub tenant_count: u64,
    pub counter_value: i64,
    pub last_login: Option<String>,
    pub app_version: String,
}

/// Generic counter operation response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CounterResponse {
    /// The current counter value after the operation.
    pub value: i64,
}

/// Legacy generic error response kept for public Rust API compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong.
    pub error: String,
}

/// Agent conversation summary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Agent conversation with messages.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConversationDetail {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

/// Request body for creating an agent conversation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateConversationRequest {
    /// Conversation title.
    pub title: String,
}

/// Request body for agent chat.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatRequest {
    /// The conversation ID to continue or create.
    pub conversation_id: String,
    /// User message content.
    pub content: String,
    /// LLM API key (provided by client, not stored).
    pub api_key: String,
    /// LLM API base URL.
    pub base_url: String,
    /// Model name to use.
    pub model: String,
}

/// Current authenticated user profile.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: String,
    pub user_sub: String,
    pub display_name: String,
    pub email: Option<String>,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// Tenant binding visible to the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserTenantResponse {
    pub tenant_id: String,
    pub tenant_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_tenant_response_serializes_nullable_name() {
        let response = UserTenantResponse {
            tenant_id: "tenant-1".to_string(),
            tenant_name: None,
            role: "owner".to_string(),
            joined_at: "2026-05-02T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("tenant_name"));
        assert!(json.contains("null"));
    }

    #[test]
    fn agent_config_serialize_redacts_api_key() {
        let config = AgentConfig {
            api_key: "sk-secret-key-12345".to_string(),
            base_url: "https://api.example.com".to_string(),
            model: "gpt-4".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();

        assert!(
            !json.contains("sk-secret-key-12345"),
            "api_key must not appear in serialized JSON"
        );
        assert!(
            !json.contains("api_key"),
            "api_key field must be omitted from serialized JSON"
        );
        assert!(json.contains("base_url"));
        assert!(json.contains("model"));
    }

    #[test]
    fn agent_config_deserialize_populates_api_key() {
        let json =
            r#"{"api_key":"sk-from-json","base_url":"https://api.example.com","model":"gpt-4"}"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.api_key, "sk-from-json");
        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn agent_config_debug_redacts_api_key() {
        let config = AgentConfig {
            api_key: "sk-secret-key-12345".to_string(),
            base_url: "https://api.example.com".to_string(),
            model: "gpt-4".to_string(),
        };

        let debug_str = format!("{:?}", config);

        assert!(
            !debug_str.contains("sk-secret-key-12345"),
            "api_key must not appear in Debug output"
        );
        assert!(
            debug_str.contains("[REDACTED]"),
            "Debug output must contain [REDACTED]"
        );
        assert!(debug_str.contains("base_url"));
        assert!(debug_str.contains("model"));
    }
}
