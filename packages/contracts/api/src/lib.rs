//! contracts/api — Route-level shared DTOs.
//! All types derive TS for automatic TypeScript generation.

use serde::{Deserialize, Serialize};
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
    pub arguments: serde_json::Value,
    pub result: Option<String>,
}

/// Agent configuration (user-provided API key + endpoint).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "api/")]
pub struct AgentConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
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
}
