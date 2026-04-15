//! Tenant initialization handlers — migrated to web-bff.
//!
//! POST /api/tenant/init — ensure tenant exists for user (auto-create on first login).

use axum::{Json, Router, extract::State, routing::post};
use data::ports::lib_sql::LibSqlPort;
use serde_json::{Value, json};

use contracts_api::InitTenantRequest;
use tenant_service::application::{TenantService, TenantServiceTrait};
use tenant_service::infrastructure::libsql_adapter::LibSqlTenantRepository;
use validator::Validate;

use crate::error::{BffError, BffResult};
use crate::state::{BffState, DatabaseBackend};

pub fn router() -> Router<BffState> {
    Router::<BffState>::new().route("/api/tenant/init", post(init_tenant))
}

/// POST /api/tenant/init
///
/// Ensures a tenant exists for the given user_sub.
/// - First login: creates tenant + user_tenant (role: 'owner')
/// - Subsequent login: returns existing tenant_id
#[utoipa::path(
    post,
    path = "/api/tenant/init",
    tag = "tenant",
    request_body = InitTenantRequest,
    responses(
        (status = 200, description = "Tenant initialized successfully", body = serde_json::Value, content_type = "application/json"),
        (status = 400, description = "Bad request — empty user_sub or user_name"),
        (status = 401, description = "Unauthorized — missing or invalid JWT"),
        (status = 422, description = "Unprocessable Entity — invalid request body"),
        (status = 500, description = "Internal server error — database failure"),
    ),
)]
pub async fn init_tenant(
    State(state): State<BffState>,
    Json(body): Json<InitTenantRequest>,
) -> BffResult<Json<Value>> {
    body.validate()
        .map_err(|e| BffError::Validation(e.to_string()))?;

    match state.db.clone() {
        Some(DatabaseBackend::Embedded(db)) => {
            let repo = LibSqlTenantRepository::new(db);
            repo.migrate().await.map_err(|e| {
                BffError::Internal(format!("Failed to run tenant migrations: {}", e))
            })?;
            let service = TenantService::new(repo);
            let result = service
                .init_tenant_for_user(&body.user_sub, &body.user_name)
                .await
                .map_err(|e| BffError::Internal(format!("Failed to initialize tenant: {}", e)))?;
            Ok(Json(json!({
                "tenant_id": result.tenant_id,
                "role": result.role,
                "created": result.created,
            })))
        }
        Some(DatabaseBackend::Remote(db)) => {
            let repo = LibSqlTenantRepository::new(db);
            repo.migrate().await.map_err(|e| {
                BffError::Internal(format!("Failed to run tenant migrations: {}", e))
            })?;
            let service = TenantService::new(repo);
            let result = service
                .init_tenant_for_user(&body.user_sub, &body.user_name)
                .await
                .map_err(|e| BffError::Internal(format!("Failed to initialize tenant: {}", e)))?;
            Ok(Json(json!({
                "tenant_id": result.tenant_id,
                "role": result.role,
                "created": result.created,
            })))
        }
        None => Err(BffError::Internal("Database not initialized".to_string())),
    }
}
