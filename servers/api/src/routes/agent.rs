//! Agent REST API routes with SSE streaming.

use crate::state::AppState;
use axum::{
    Router,
    extract::{Json, Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
};
use feature_agent::AgentService;
use futures_util::{StreamExt, stream};
use serde::Deserialize;
use std::convert::Infallible;
use std::pin::Pin;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/agent/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route("/agent/conversations/:id/messages", get(get_messages))
        .route("/agent/chat", post(chat_handler))
}

fn get_db(state: &AppState) -> Result<storage_libsql::EmbeddedLibSql, Json<serde_json::Value>> {
    state
        .embedded_db
        .clone()
        .ok_or_else(|| Json(serde_json::json!({ "error": "Embedded database not initialized" })))
}

#[derive(Deserialize)]
struct CreateConversationReq {
    title: String,
}

async fn create_conversation(
    State(state): State<AppState>,
    Json(req): Json<CreateConversationReq>,
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

async fn list_conversations(State(state): State<AppState>) -> Json<serde_json::Value> {
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

async fn get_messages(
    State(state): State<AppState>,
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

#[derive(Deserialize)]
struct ChatReq {
    conversation_id: String,
    content: String,
    api_key: String,
    base_url: String,
    model: String,
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatReq>,
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
