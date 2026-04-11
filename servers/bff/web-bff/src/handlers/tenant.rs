//! Tenant initialization handlers — migrated to web-bff.
//!
//! POST /api/tenant/init — ensure tenant exists for user (auto-create on first login).

use axum::{Json, Router, extract::State, routing::post};
use domain::ports::lib_sql::LibSqlPort;
use serde::Deserialize;
use serde_json::{Value, json};

use contracts_api::{InitTenantRequest, InitTenantResponse};
use validator::Validate;

use crate::state::BffState;
use crate::error::{BffError, BffResult};

#[derive(Debug, Deserialize)]
struct UserTenantRecord {
    id: String,
    tenant_id: String,
    role: String,
}

/// Result type from tenant CREATE query.
#[derive(Debug, Deserialize)]
struct TenantRecord {
    id: String,
}

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
        (status = 200, description = "Tenant initialized successfully", body = InitTenantResponse, content_type = "application/json"),
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

    let db = state
        .embedded_db
        .clone()
        .ok_or_else(|| BffError::Internal("Embedded Turso database not initialized".to_string()))?;

    // 1. Check existing binding
    let existing: Vec<UserTenantRecord> = db
        .query(
            "SELECT id, tenant_id, role FROM user_tenant WHERE user_sub = ? LIMIT 1",
            vec![body.user_sub.clone()],
        )
        .await
        .map_err(|e| BffError::Internal(format!("Database query failed: {}", e)))?;

    if let Some(ut) = existing.first() {
        // Already bound — return existing
        return Ok(Json(json!({
            "tenant_id": ut.tenant_id,
            "role": ut.role,
            "created": false,
        })));
    }

    // 2. Create tenant
    let tenant_name = body.user_name.clone();

    let created_tenants: Vec<TenantRecord> = db
        .query(
            "INSERT INTO tenant (id, name) VALUES (lower(hex(randomblob(16))), ?) RETURNING id",
            vec![tenant_name],
        )
        .await
        .map_err(|e| BffError::Internal(format!("Failed to create tenant: {}", e)))?;

    let created = created_tenants
        .first()
        .ok_or_else(|| BffError::Internal("Failed to create tenant".to_string()))?;
    let tenant_id = &created.id;

    // 3. Create user_tenant binding (owner role)
    db.execute(
        "INSERT INTO user_tenant (id, user_sub, tenant_id, role) VALUES (lower(hex(randomblob(16))), ?, ?, 'owner')",
        vec![body.user_sub, tenant_id.to_string()],
    )
        .await
        .map_err(|e| BffError::Internal(format!("Failed to create user-tenant binding: {}", e)))?;

    Ok(Json(json!({
        "tenant_id": tenant_id,
        "role": "owner",
        "created": true,
    })))
}
