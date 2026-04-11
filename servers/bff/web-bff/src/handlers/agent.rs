//! Agent REST API handlers with SSE streaming — migrated to web-bff.

use axum::{
    Router,
    extract::{Json, Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
};
use contracts_api::{
    ChatMessage, ChatRequest, ConversationDetail, ConversationSummary, CreateConversationRequest,
};
use feature_agent::AgentService;
use futures_util::{StreamExt, stream};
use serde_json::json;
use std::convert::Infallible;
use std::pin::Pin;
use utoipa::OpenApi;

use crate::state::BffState;

pub fn router() -> Router<BffState> {
    Router::new()
        .route(
            "/api/agent/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route("/api/agent/conversations/{id}/messages", get(get_messages))
        .route("/api/agent/chat", post(chat_handler))
}

fn get_db(state: &BffState) -> Result<storage_turso::EmbeddedTurso, Json<serde_json::Value>> {
    state
        .embedded_db
        .clone()
        .ok_or_else(|| Json(json!({ "error": "Embedded database not initialized" })))
}

/// List all agent conversations.
#[utoipa::path(
    get,
    path = "/api/agent/conversations",
    tag = "agent",
    responses(
        (status = 200, description = "List of conversation summaries", body = Vec<ConversationSummary>, content_type = "application/json"),
        (status = 500, description = "Internal server error", body = serde_json::Value, content_type = "application/json"),
    ),
)]
pub async fn list_conversations(State(state): State<BffState>) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::agent_service::LibSqlAgentService::new(db, state.http_client.clone());
    match service.get_conversations().await {
        Ok(convs) => Json(serde_json::json!(convs)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// Create a new agent conversation.
#[utoipa::path(
    post,
    path = "/api/agent/conversations",
    tag = "agent",
    request_body = CreateConversationRequest,
    responses(
        (status = 200, description = "Conversation created successfully", body = ConversationSummary, content_type = "application/json"),
        (status = 500, description = "Internal server error", body = serde_json::Value, content_type = "application/json"),
    ),
)]
pub async fn create_conversation(
    State(state): State<BffState>,
    Json(req): Json<CreateConversationRequest>,
) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::agent_service::LibSqlAgentService::new(db, state.http_client.clone());
    match service.create_conversation(&req.title).await {
        Ok(conv) => Json(serde_json::json!(conv)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// Get messages for a specific conversation.
#[utoipa::path(
    get,
    path = "/api/agent/conversations/{id}/messages",
    tag = "agent",
    params(
        ("id" = String, Path, description = "Conversation ID"),
    ),
    responses(
        (status = 200, description = "List of chat messages", body = Vec<ChatMessage>, content_type = "application/json"),
        (status = 500, description = "Internal server error", body = serde_json::Value, content_type = "application/json"),
    ),
)]
pub async fn get_messages(
    State(state): State<BffState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::agent_service::LibSqlAgentService::new(db, state.http_client.clone());
    match service.get_messages(&id).await {
        Ok(msgs) => Json(serde_json::json!(msgs)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// Chat with the agent via SSE streaming.
#[utoipa::path(
    post,
    path = "/api/agent/chat",
    tag = "agent",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "SSE stream of agent responses", content_type = "text/event-stream"),
        (status = 500, description = "Internal server error", body = serde_json::Value, content_type = "application/json"),
    ),
)]
pub async fn chat_handler(
    State(state): State<BffState>,
    Json(req): Json<ChatRequest>,
) -> Sse<Pin<Box<dyn futures_util::Stream<Item = Result<Event, Infallible>> + Send>>> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => {
            let err_msg = match e {
                Json(v) => v["error"].as_str().unwrap_or("DB error").to_string(),
            };
            return Sse::new(Box::pin(stream::once(async move {
                Ok(Event::default().data(err_msg))
            })));
        }
    };
    let service = usecases::agent_service::LibSqlAgentService::new(db, state.http_client.clone());
    match service
        .chat_stream(
            &req.conversation_id,
            &req.content,
            &req.api_key,
            &req.base_url,
            &req.model,
        )
        .await
    {
        Ok(s) => {
            let event_stream = s.map(|result| -> Result<Event, Infallible> {
                match result {
                    Ok(content) => {
                        if content.contains("[tool:") {
                            Ok(Event::default().event("tool").data(content))
                        } else {
                            Ok(Event::default().event("assistant").data(content))
                        }
                    }
                    Err(e) => Ok(Event::default().data(format!("Error: {}", e))),
                }
            });
            Sse::new(Box::pin(event_stream))
        }
        Err(e) => Sse::new(Box::pin(stream::once(async move {
            Ok(Event::default().data(format!("Error: {}", e)))
        }))),
    }
}
