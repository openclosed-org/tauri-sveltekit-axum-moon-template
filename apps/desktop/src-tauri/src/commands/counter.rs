//! Counter Tauri commands — bridge to SDK CounterService.
//!
//! Uses `sdk_counter_embedded::EmbeddedCounterClient` so the desktop app
//! never imports `counter-service` directly.

use sdk_counter::{CounterId, CounterService};
use sdk_counter_embedded::EmbeddedCounterClient;
use storage_turso::EmbeddedTurso;
use tauri::Manager;

/// Default counter ID for desktop — in production this would come from auth context.
fn desktop_counter_id() -> CounterId {
    CounterId::new("desktop-default")
}

#[tauri::command]
pub async fn counter_increment(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let client = EmbeddedCounterClient::new(db).await?;
    let counter_id = desktop_counter_id();
    client
        .increment(&counter_id, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_decrement(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let client = EmbeddedCounterClient::new(db).await?;
    let counter_id = desktop_counter_id();
    client
        .decrement(&counter_id, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_reset(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let client = EmbeddedCounterClient::new(db).await?;
    let counter_id = desktop_counter_id();
    client
        .reset(&counter_id, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_get_value(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let client = EmbeddedCounterClient::new(db).await?;
    let counter_id = desktop_counter_id();
    client
        .get_value(&counter_id)
        .await
        .map_err(|e| e.to_string())
}
