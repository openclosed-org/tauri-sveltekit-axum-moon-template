//! Tenant initialization endpoints.
//!
//! POST /api/tenant/init — ensure tenant exists for user (auto-create on first login).

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::ports::surreal_db::TenantAwareSurrealDb;
use crate::state::AppState;
use domain::ports::surreal_db::SurrealDbPort;

/// Helper: create a serde_json::Value::String from &str.
fn json_str(s: &str) -> Value {
    Value::String(s.to_string())
}

/// Format a RecordId as "table:key" string.
fn record_id_to_string(rid: &RecordId) -> String {
    let key_str = match &rid.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => format!("{:?}", rid.key),
    };
    format!("{}:{}", rid.table, key_str)
}

/// Request body for tenant init.
#[derive(Debug, Deserialize)]
pub struct InitTenantRequest {
    pub user_sub: String,
    pub user_name: String,
}

/// Response from tenant init.
#[derive(Debug, Serialize)]
pub struct InitTenantResponse {
    pub tenant_id: String,
    pub role: String,
    pub created: bool,
}

/// Result type from user_tenant SELECT query.
#[derive(Debug, Deserialize)]
struct UserTenantRecord {
    #[allow(dead_code)]
    id: RecordId,
    tenant_id: RecordId,
    role: String,
}

/// Result type from tenant CREATE query.
#[derive(Debug, Deserialize)]
struct TenantRecord {
    #[allow(dead_code)]
    id: RecordId,
}

/// POST /api/tenant/init
///
/// Ensures a tenant exists for the given user_sub.
/// - First login: creates tenant + user_tenant (role: 'owner')
/// - Subsequent login: returns existing tenant_id
pub async fn init_tenant(
    State(state): State<AppState>,
    Json(body): Json<InitTenantRequest>,
) -> Result<Json<Value>, StatusCode> {
    if body.user_sub.is_empty() || body.user_name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Use admin-mode DB for tenant operations (cross-tenant query)
    let admin_db = TenantAwareSurrealDb::new_admin(state.db.clone());

    // 1. Check existing binding
    let existing: Vec<UserTenantRecord> = admin_db
        .query(
            "SELECT id, tenant_id, role FROM user_tenant WHERE user_sub = $sub",
            BTreeMap::from([("sub".into(), json_str(&body.user_sub))]),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(ut) = existing.first() {
        // Already bound — return existing
        return Ok(Json(json!({
            "tenant_id": record_id_to_string(&ut.tenant_id),
            "role": ut.role,
            "created": false,
        })));
    }

    // 2. Create tenant
    let tenant_name = if body.user_name.is_empty() {
        body.user_sub.clone()
    } else {
        body.user_name.clone()
    };

    let created_tenants: Vec<TenantRecord> = admin_db
        .query(
            "CREATE tenant SET name = $name",
            BTreeMap::from([("name".into(), json_str(&tenant_name))]),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created = created_tenants
        .first()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let tenant_id = record_id_to_string(&created.id);

    // 3. Create user_tenant binding (owner role)
    // Use the tenant_id string directly — SurrealDB will parse "tenant:xxx" as a record
    let _: Vec<serde_json::Value> = admin_db
        .query(
            &format!(
                "CREATE user_tenant SET user_sub = $sub, tenant_id = {}, role = 'owner'",
                tenant_id
            ),
            BTreeMap::from([("sub".into(), json_str(&body.user_sub))]),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    use super::*;

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
