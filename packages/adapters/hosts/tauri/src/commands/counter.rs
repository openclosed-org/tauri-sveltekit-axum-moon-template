//! Counter Tauri commands — bridge to CounterService.

use feature_counter::CounterService;
use storage_turso::EmbeddedTurso;
use tauri::Manager;

fn build_turso_counter_service(
    db: EmbeddedTurso,
) -> counter_service::application::RepositoryBackedCounterService<
    counter_service::infrastructure::LibSqlCounterRepository<EmbeddedTurso>,
> {
    let repo = counter_service::infrastructure::LibSqlCounterRepository::new(db);
    counter_service::application::RepositoryBackedCounterService::new(repo)
}

#[tauri::command]
pub async fn counter_increment(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_turso_counter_service(db);
    service.increment().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_decrement(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_turso_counter_service(db);
    service.decrement().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_reset(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_turso_counter_service(db);
    service.reset().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn counter_get_value(app: tauri::AppHandle) -> Result<i64, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let service = build_turso_counter_service(db);
    service.get_value().await.map_err(|e| e.to_string())
}
