//! Agent service — LibSQL-backed chat + OpenAI streaming integration.

use async_trait::async_trait;
use contracts_api::ChatMessage;
use domain::ports::lib_sql::LibSqlPort;
use feature_agent::{AgentError, AgentService, Conversation};
use futures_util::{Stream, StreamExt, future};
use std::pin::Pin;

/// SQL migrations for agent tables.
pub const AGENT_MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS conversations (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
    "CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES conversations(id), role TEXT NOT NULL, content TEXT NOT NULL, tool_calls TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
];

/// AgentService backed by LibSqlPort + reqwest HTTP client.
pub struct LibSqlAgentService<P: LibSqlPort> {
    port: P,
    http_client: reqwest::Client,
}

impl<P: LibSqlPort> LibSqlAgentService<P> {
    pub fn new(port: P, http_client: reqwest::Client) -> Self {
        Self { port, http_client }
    }
}

fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Row type for conversations query.
#[derive(Debug, serde::Deserialize)]
struct ConversationRow {
    id: String,
    title: String,
    created_at: String,
}

/// Row type for messages query.
#[derive(Debug, serde::Deserialize)]
struct MessageRow {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    tool_calls: Option<String>,
    created_at: String,
}

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

#[async_trait]
impl<P: LibSqlPort> AgentService for LibSqlAgentService<P> {
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
        // Build messages history from existing conversation
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
        // Add the new user message
        api_messages.push(serde_json::json!({
            "role": "user",
            "content": content,
        }));

        // Build tool definitions
        let tools: Vec<serde_json::Value> = feature_agent::AVAILABLE_TOOLS
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

        // Save the user message to database
        self.send_message(conversation_id, content).await?;

        // Return SSE stream — parse "data: {json}" lines
        let stream = resp
            .bytes_stream()
            .map(
                |chunk: Result<bytes::Bytes, reqwest::Error>| -> Result<String, AgentError> {
                    let chunk = chunk.map_err(|e| AgentError::Api(e.to_string()))?;
                    let text = String::from_utf8_lossy(&chunk);
                    // Parse SSE format: "data: {json}\n\n"
                    let mut result = String::new();
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                continue;
                            }
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(content) =
                                    parsed["choices"][0]["delta"]["content"].as_str()
                                {
                                    result.push_str(content);
                                }
                            }
                        }
                    }
                    if result.is_empty() && text.trim().is_empty() {
                        return Err(AgentError::Api("Empty chunk".to_string()));
                    }
                    Ok(result)
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
