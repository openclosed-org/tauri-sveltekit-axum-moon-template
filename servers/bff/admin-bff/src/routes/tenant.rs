//! Admin tenant management routes.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use tenant_service::infrastructure::libsql_adapter::LibSqlTenantRepository;
use tenant_service::ports::TenantRepository;

use crate::error::AdminBffResult;
use crate::state::AdminBffState;

#[derive(Serialize, utoipa::ToSchema)]
pub struct TenantListView {
    pub tenants: Vec<TenantItemView>,
    pub total: usize,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TenantItemView {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub member_count: usize,
}

/// GET /api/admin/tenants — List all tenants for admin review
#[utoipa::path(
    get,
    path = "/api/admin/tenants",
    responses(
        (status = 200, description = "Tenant list retrieved successfully", body = TenantListView),
        (status = 401, description = "Unauthorized")
    ),
    tag = "admin"
)]
pub async fn list_tenants(
    State(state): State<AdminBffState>,
) -> AdminBffResult<Json<TenantListView>> {
    let db = state.embedded_db.clone().ok_or_else(|| {
        crate::error::AdminBffError::Internal("Embedded database not initialized".to_string())
    })?;

    let repo = LibSqlTenantRepository::new(db);
    let tenants = repo.list_tenants().await.map_err(|e| {
        crate::error::AdminBffError::Internal(format!("Failed to list tenants: {}", e))
    })?;

    let view = TenantListView {
        total: tenants.len(),
        tenants: tenants
            .into_iter()
            .map(|t| TenantItemView {
                id: t.id,
                name: t.name,
                created_at: t.created_at,
                member_count: 0, // TODO: requires user_tenant count query
            })
            .collect(),
    };

    Ok(Json(view))
}

pub fn router() -> Router<AdminBffState> {
    Router::new().route("/api/admin/tenants", get(list_tenants))
}
