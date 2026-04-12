//! Settings REST API routes.
//!
//! GET    /api/settings              — get user settings
//! PUT    /api/settings              — update user settings

use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, put},
};
use domain::ports::TenantId;
use feature_settings::{AgentConnectionSettings, SettingsService};
use settings_service::infrastructure::LibSqlSettingsRepository;
use settings_service::application::ApplicationSettingsService;
use utoipa::OpenApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings", get(get_settings))
        .route("/api/settings", put(update_settings))
}

/// Get user settings.
#[utoipa::path(
    get,
    path = "/api/settings",
    tag = "settings",
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "User settings retrieved", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_settings(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    // tenant_id.0 contains the JWT `sub` claim (extracted by tenant_middleware)
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let repo = LibSqlSettingsRepository::new(db);
    let service = ApplicationSettingsService::new(repo);

    match service.get_settings(&tenant_id.0).await {
        Ok(settings) => (StatusCode::OK, Json(serde_json::json!({
            "api_key_masked": mask_api_key(&settings.api_key),
            "base_url": settings.base_url,
            "model": settings.model,
        }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// Update user settings.
#[utoipa::path(
    put,
    path = "/api/settings",
    tag = "settings",
    request_body = serde_json::Value,
    security(("tenant_auth" = [])),
    responses(
        (status = 200, description = "Settings updated", body = serde_json::Value, content_type = "application/json"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn update_settings(
    State(state): State<AppState>,
    tenant: Option<Extension<TenantId>>,
    Json(body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let db = match state.embedded_db.clone() {
        Some(db) => db,
        None => return db_not_ready(),
    };
    let tenant_id = match extract_tenant(tenant) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let api_key = body.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = body.get("base_url").and_then(|v| v.as_str()).unwrap_or("https://api.openai.com/v1").to_string();
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-4o-mini").to_string();

    let repo = LibSqlSettingsRepository::new(db);
    let service = ApplicationSettingsService::new(repo);

    let new_settings = AgentConnectionSettings { api_key, base_url, model };

    match service.update_agent_connection(&tenant_id.0, new_settings).await {
        Ok(settings) => (StatusCode::OK, Json(serde_json::json!({
            "api_key_masked": mask_api_key(&settings.api_key),
            "base_url": settings.base_url,
            "model": settings.model,
        }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}

fn db_not_ready() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Embedded database not initialized" })),
    )
}

fn extract_tenant(
    tenant: Option<Extension<TenantId>>,
) -> Result<TenantId, (StatusCode, Json<serde_json::Value>)> {
    tenant.map(|Extension(id)| id).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing tenant context" })),
        )
    })
}
