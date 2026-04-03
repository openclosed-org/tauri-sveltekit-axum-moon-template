//! Agent Tauri commands — bridge to AgentService with Channel streaming.

use feature_agent::AgentService;
use futures_util::StreamExt;
use storage_libsql::EmbeddedLibSql;
use tauri::{AppHandle, Manager, ipc::Channel};

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
    let db = app.state::<EmbeddedLibSql>().inner().clone();
    let http_client = reqwest::Client::new();
    let service = usecases::agent_service::LibSqlAgentService::new(db, http_client);

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
