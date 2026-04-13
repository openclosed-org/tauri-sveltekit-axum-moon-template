//! contracts/api — Route-level shared DTOs.
//! All types derive TS for automatic TypeScript generation.

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::fmt;
use ts_rs::TS;
use utoipa::ToSchema;
use validator::Validate;

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct HealthResponse {
    /// Server status: "ok" or "degraded"
    pub status: String,
}

/// Request body for tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct InitTenantRequest {
    /// OAuth provider's subject identifier.
    #[validate(length(min = 1, message = "user_sub is required"))]
    pub user_sub: String,
    /// Display name for the user.
    #[validate(length(min = 1, max = 100))]
    pub user_name: String,
}

/// Response from tenant initialization.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct InitTenantResponse {
    /// The tenant ID in "table:key" format.
    pub tenant_id: String,
    /// User's role within the tenant.
    pub role: String,
    /// Whether a new tenant was created.
    pub created: bool,
}

/// Chat message (user or assistant).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    #[ts(type = "any")]
    pub arguments: serde_json::Value,
    pub result: Option<String>,
}

/// Agent configuration (user-provided API key + endpoint).
#[derive(Clone, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct AgentConfig {
    /// API key — omitted from serialized payloads to prevent accidental exposure.
    #[ts(skip)]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct AdminDashboardStats {
    #[ts(type = "number")]
    pub tenant_count: u64,
    #[ts(type = "number")]
    pub counter_value: i64,
    pub last_login: Option<String>,
    pub app_version: String,
}

/// Generic counter operation response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct CounterResponse {
    /// The current counter value after the operation.
    pub value: i64,
}

/// Generic error response returned on failure.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct ErrorResponse {
    /// Error message describing what went wrong.
    pub error: String,
}

/// Agent conversation summary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Agent conversation with messages.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct ConversationDetail {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

/// Request body for creating an agent conversation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct CreateConversationRequest {
    /// Conversation title.
    pub title: String,
}

/// Request body for agent chat.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_health_response() {
        HealthResponse::export().unwrap();
    }

    #[test]
    fn export_init_tenant_request() {
        InitTenantRequest::export().unwrap();
    }

    #[test]
    fn export_init_tenant_response() {
        InitTenantResponse::export().unwrap();
    }

    #[test]
    fn export_chat_message() {
        ChatMessage::export().unwrap();
    }

    #[test]
    fn export_tool_call() {
        ToolCall::export().unwrap();
    }

    #[test]
    fn export_agent_config() {
        AgentConfig::export().unwrap();
    }

    #[test]
    fn export_admin_dashboard_stats() {
        AdminDashboardStats::export().unwrap();
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
