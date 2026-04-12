//! LibSQL + HTTP implementation of agent service.
//!
//! Handles conversation CRUD, message persistence, and OpenAI-compatible
//! streaming with tool execution.

use async_trait::async_trait;
use contracts_api::ChatMessage;
use domain::ports::lib_sql::LibSqlPort;
use feature_agent::{AgentError, AgentService, Conversation, AVAILABLE_TOOLS};
use futures_util::{Stream, StreamExt, future};
use serde::Deserialize;
use std::pin::Pin;

// ── Row types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ConversationRow {
    id: String,
    title: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct MessageRow {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    tool_calls: Option<String>,
    created_at: String,
}

/// Raw tenant row for list_tenants tool (local shape to avoid cross-service dep).
#[derive(Debug, Deserialize, serde::Serialize)]
struct TenantRow {
    id: String,
    name: String,
    created_at: String,
}

// ── Type conversions ─────────────────────────────────────────────────────────

impl From<ConversationRow> for Conversation {
    fn from(r: ConversationRow) -> Self {
        Conversation {
            id: r.id,
            title: r.title,
            created_at: r.created_at,
        }
    }
}

impl From<MessageRow> for ChatMessage {
    fn from(r: MessageRow) -> Self {
        let tool_calls = r.tool_calls.and_then(|s| serde_json::from_str(&s).ok());
        ChatMessage {
            id: r.id,
            conversation_id: r.conversation_id,
            role: r.role,
            content: r.content,
            tool_calls,
            created_at: r.created_at,
        }
    }
}

// ── Service ──────────────────────────────────────────────────────────────────

/// AgentService backed by LibSqlPort + reqwest HTTP client.
pub struct LibSqlAgentRepository<P: LibSqlPort> {
    port: P,
    http_client: reqwest::Client,
}

impl<P: LibSqlPort> LibSqlAgentRepository<P> {
    pub fn new(port: P, http_client: reqwest::Client) -> Self {
        Self { port, http_client }
    }

    /// Run agent table migrations (idempotent).
    pub async fn migrate(&self) -> Result<(), AgentError> {
        for migration in super::super::application::migrations::AGENT_MIGRATIONS {
            self.port
                .execute(migration, vec![])
                .await
                .map_err(AgentError::Database)?;
        }
        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

async fn persist_tool_result<P: LibSqlPort>(
    port: &P,
    conversation_id: &str,
    tool_name: &str,
    arguments: &serde_json::Value,
    result: &str,
) -> Result<(), AgentError> {
    let id = generate_id();
    let now = chrono::Utc::now().to_rfc3339();
    let tool_calls = serde_json::json!([
        {
            "id": id,
            "name": tool_name,
            "arguments": arguments,
            "result": result,
        }
    ]);

    port.execute(
        "INSERT INTO messages (id, conversation_id, role, content, tool_calls, created_at) VALUES (?, ?, 'tool', ?, ?, ?)",
        vec![
            generate_id(),
            conversation_id.to_string(),
            result.to_string(),
            tool_calls.to_string(),
            now,
        ],
    )
    .await
    .map_err(AgentError::Database)?;

    Ok(())
}

async fn execute_tool_by_name<P: LibSqlPort>(
    port: &P,
    conversation_id: &str,
    name: &str,
    arguments: serde_json::Value,
) -> Result<String, AgentError> {
    let result = match name {
        "get_counter_value" => {
            let rows: Vec<(i64,)> = port
                .query("SELECT value FROM counter WHERE id = 1", vec![])
                .await
                .map_err(AgentError::Database)?;
            let value = rows.first().map(|r| r.0).unwrap_or(0);
            serde_json::json!({ "counter_value": value }).to_string()
        }
        "list_tenants" => {
            let tenants: Vec<TenantRow> = port
                .query(
                    "SELECT id, name, created_at FROM tenant ORDER BY created_at DESC",
                    vec![],
                )
                .await
                .map_err(AgentError::Database)?;
            serde_json::json!({ "tenants": tenants }).to_string()
        }
        "get_system_status" => {
            port.health_check().await.map_err(AgentError::Database)?;
            serde_json::json!({ "status": "ok" }).to_string()
        }
        other => {
            return Err(AgentError::Api(format!(
                "Unknown tool: {other}. Allowed: get_counter_value, list_tenants, get_system_status"
            )));
        }
    };

    persist_tool_result(port, conversation_id, name, &arguments, &result).await?;
    Ok(result)
}

// ── Trait implementation ─────────────────────────────────────────────────────

#[async_trait]
impl<P: LibSqlPort + Clone + Send + Sync + 'static> AgentService for LibSqlAgentRepository<P> {
    async fn create_conversation(&self, title: &str) -> Result<Conversation, AgentError> {
        let id = generate_id();
        let now = chrono::Utc::now().to_rfc3339();
        self.port
            .execute(
                "INSERT INTO conversations (id, title, created_at) VALUES (?, ?, ?)",
                vec![id.clone(), title.to_string(), now.clone()],
            )
            .await
            .map_err(AgentError::Database)?;
        Ok(Conversation {
            id,
            title: title.to_string(),
            created_at: now,
        })
    }

    async fn get_conversations(&self) -> Result<Vec<Conversation>, AgentError> {
        let rows: Vec<ConversationRow> = self
            .port
            .query(
                "SELECT id, title, created_at FROM conversations ORDER BY created_at DESC",
                vec![],
            )
            .await
            .map_err(AgentError::Database)?;
        Ok(rows.into_iter().map(Conversation::from).collect())
    }

    async fn get_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>, AgentError> {
        let rows: Vec<MessageRow> = self
            .port
            .query(
                "SELECT id, conversation_id, role, content, tool_calls, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
                vec![conversation_id.to_string()],
            )
            .await
            .map_err(AgentError::Database)?;
        Ok(rows.into_iter().map(ChatMessage::from).collect())
    }

    async fn send_message(
        &self,
        conversation_id: &str,
        content: &str,
    ) -> Result<ChatMessage, AgentError> {
        let id = generate_id();
        let now = chrono::Utc::now().to_rfc3339();
        self.port
            .execute(
                "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?, ?, 'user', ?, ?)",
                vec![
                    id.clone(),
                    conversation_id.to_string(),
                    content.to_string(),
                    now.clone(),
                ],
            )
            .await
            .map_err(AgentError::Database)?;
        Ok(ChatMessage {
            id,
            conversation_id: conversation_id.to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            tool_calls: None,
            created_at: now,
        })
    }

    async fn chat_stream(
        &self,
        conversation_id: &str,
        content: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, AgentError>> + Send>>, AgentError> {
        let messages = self.get_messages(conversation_id).await?;
        let mut api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();
        api_messages.push(serde_json::json!({
            "role": "user",
            "content": content,
        }));

        let tools: Vec<serde_json::Value> = AVAILABLE_TOOLS
            .iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": name,
                        "description": desc,
                        "parameters": { "type": "object", "properties": {} }
                    }
                })
            })
            .collect();

        let request_body = serde_json::json!({
            "model": model,
            "messages": api_messages,
            "tools": tools,
            "stream": true,
        });

        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AgentError::Api(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AgentError::Api(format!("OpenAI API error: {}", body)));
        }

        self.send_message(conversation_id, content).await?;

        let conversation_id = conversation_id.to_string();
        let port = self.port.clone();
        let raw = Box::pin(resp.bytes_stream())
            as Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>;

        let stream = futures_util::stream::unfold(
            (raw, conversation_id, port),
            |(mut inner, conversation_id, port)| async move {
                let next = inner.next().await;
                match next {
                    Some(Ok(chunk)) => {
                        let text = String::from_utf8_lossy(&chunk);
                        let mut result = String::new();

                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    continue;
                                }
                                if let Ok(parsed) =
                                    serde_json::from_str::<serde_json::Value>(data)
                                {
                                    if let Some(content) =
                                        parsed["choices"][0]["delta"]["content"].as_str()
                                    {
                                        result.push_str(content);
                                    }
                                    if let Some(tool_calls) =
                                        parsed["choices"][0]["delta"]["tool_calls"].as_array()
                                    {
                                        for call in tool_calls {
                                            let name =
                                                call["function"]["name"].as_str().unwrap_or("");
                                            if name.is_empty() {
                                                continue;
                                            }
                                            let arguments = call["function"]["arguments"]
                                                .as_str()
                                                .and_then(|raw| serde_json::from_str(raw).ok())
                                                .unwrap_or_else(|| serde_json::json!({}));

                                            match execute_tool_by_name(
                                                &port,
                                                &conversation_id,
                                                name,
                                                arguments,
                                            )
                                            .await
                                            {
                                                Ok(tool_result) => {
                                                    result.push_str(&format!(
                                                        "\n[tool:{name}] {tool_result}\n"
                                                    ));
                                                }
                                                Err(err) => {
                                                    result.push_str(&format!(
                                                        "\n[tool:{name}] error: {err}\n"
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if result.is_empty() && text.trim().is_empty() {
                            Some((
                                Err(AgentError::Api("Empty chunk".to_string())),
                                (inner, conversation_id, port),
                            ))
                        } else {
                            Some((Ok(result), (inner, conversation_id, port)))
                        }
                    }
                    Some(Err(e)) => Some((
                        Err(AgentError::Api(e.to_string())),
                        (inner, conversation_id, port),
                    )),
                    None => None,
                }
            },
        )
        .filter(|r: &Result<String, AgentError>| {
            future::ready(match r {
                Ok(s) => !s.is_empty(),
                Err(_) => true,
            })
        });

        Ok(Box::pin(stream))
    }
}
