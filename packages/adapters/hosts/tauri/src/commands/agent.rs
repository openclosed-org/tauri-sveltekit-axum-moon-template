//! Agent Tauri commands — bridge to AgentService with Channel streaming.

use contracts_api::ChatMessage;
use feature_agent::AgentService;
use feature_agent::Conversation;
use futures_util::StreamExt;
use storage_turso::EmbeddedTurso;
use tauri::{AppHandle, Manager, ipc::Channel};

fn agent_service(app: &AppHandle) -> agent_service::infrastructure::LibSqlAgentRepository<EmbeddedTurso> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let http_client = reqwest::Client::new();
    agent_service::infrastructure::LibSqlAgentRepository::new(db, http_client)
}

#[tauri::command]
pub async fn agent_create_conversation(
    app: AppHandle,
    title: String,
) -> Result<Conversation, String> {
    agent_service(&app)
        .create_conversation(&title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_list_conversations(app: AppHandle) -> Result<Vec<Conversation>, String> {
    agent_service(&app)
        .get_conversations()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_get_messages(app: AppHandle, id: String) -> Result<Vec<ChatMessage>, String> {
    agent_service(&app)
        .get_messages(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_chat(
    app: AppHandle,
    conversation_id: String,
    content: String,
    api_key: String,
    base_url: String,
    model: String,
    channel: Channel<String>,
) -> Result<(), String> {
    let service = agent_service(&app);

    let mut stream = service
        .chat_stream(&conversation_id, &content, &api_key, &base_url, &model)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                channel.send(text).map_err(|e| e.to_string())?;
            }
            Err(e) => {
                channel
                    .send(format!("Error: {}", e))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
