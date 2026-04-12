//! Admin Tauri commands — bridge to AdminService.

use feature_admin::AdminService;
use storage_turso::EmbeddedTurso;
use tauri::Manager;

#[tauri::command]
pub async fn admin_get_dashboard_stats(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();

    let tenant_repo = tenant_service::infrastructure::LibSqlTenantRepository::new(db.clone());
    let tenant_svc = tenant_service::application::TenantService::new(tenant_repo);

    let counter_repo = counter_service::infrastructure::LibSqlCounterRepository::new(db.clone());
    let counter_svc = counter_service::application::RepositoryBackedCounterService::new(counter_repo);

    let admin_svc = admin_service::application::AdminDashboardService::new(tenant_svc, counter_svc);

    match admin_svc.get_dashboard_stats().await {
        Ok(stats) => Ok(serde_json::json!(stats)),
        Err(e) => Err(e.to_string()),
    }
}
