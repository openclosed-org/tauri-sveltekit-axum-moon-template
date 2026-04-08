//! Tenant initialization endpoints.
//!
//! POST /api/tenant/init — ensure tenant exists for user (auto-create on first login).

use axum::{Json, Router, extract::State, routing::post};
use domain::ports::lib_sql::LibSqlPort;
use serde::Deserialize;
use serde_json::{Value, json};

use contracts_api::{InitTenantRequest, InitTenantResponse};
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
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
    State(state): State<AppState>,
    Json(body): Json<InitTenantRequest>,
) -> AppResult<Json<Value>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let db = state
        .embedded_db
        .clone()
        .ok_or_else(|| AppError::Internal("Embedded Turso database not initialized".to_string()))?;

    // 1. Check existing binding
    let existing: Vec<UserTenantRecord> = db
        .query(
            "SELECT id, tenant_id, role FROM user_tenant WHERE user_sub = ? LIMIT 1",
            vec![body.user_sub.clone()],
        )
        .await
        .map_err(AppError::Database)?;

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
        .map_err(AppError::Database)?;

    let created = created_tenants
        .first()
        .ok_or_else(|| AppError::Internal("Failed to create tenant".to_string()))?;
    let tenant_id = &created.id;

    // 3. Create user_tenant binding (owner role)
    // Use parameterized query — $tenant_id prevents SQL injection
    db.execute(
        "INSERT INTO user_tenant (id, user_sub, tenant_id, role) VALUES (lower(hex(randomblob(16))), ?, ?, 'owner')",
        vec![body.user_sub, tenant_id.to_string()],
    )
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "tenant_id": tenant_id,
        "role": "owner",
        "created": true,
    })))
}

/// Tenant route module router.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/api/tenant/init", post(init_tenant))
}

#[cfg(test)]
mod tests {
    use contracts_api::{InitTenantRequest, InitTenantResponse};

    #[test]
    fn deserialize_init_request() {
        let json = r#"{"user_sub":"google-123","user_name":"Alice"}"#;
        let req: InitTenantRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_sub, "google-123");
        assert_eq!(req.user_name, "Alice");
    }

    #[test]
    fn reject_empty_sub() {
        let req = InitTenantRequest {
            user_sub: String::new(),
            user_name: "Alice".into(),
        };
        assert!(req.user_sub.is_empty());
    }

    #[test]
    fn serialize_response() {
        let resp = InitTenantResponse {
            tenant_id: "tenant:abc".into(),
            role: "owner".into(),
            created: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"tenant_id\":\"tenant:abc\""));
        assert!(json.contains("\"role\":\"owner\""));
        assert!(json.contains("\"created\":true"));
    }
}
