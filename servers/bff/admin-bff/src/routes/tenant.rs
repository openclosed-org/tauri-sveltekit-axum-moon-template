//! Admin tenant management routes.

use axum::{
    routing::get,
    Json,
    Router,
    extract::State,
};
use serde::Serialize;
use utoipa::OpenApi;
use tenant_service::ports::TenantRepository;
use tenant_service::infrastructure::libsql_adapter::LibSqlTenantRepository;

use crate::state::AdminBffState;
use crate::error::AdminBffResult;

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

#[derive(OpenApi)]
#[openapi(
    paths(list_tenants),
    components(schemas(TenantListView, TenantItemView))
)]
pub struct TenantOpenApi;

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
    let db = state.embedded_db.clone()
        .ok_or_else(|| crate::error::AdminBffError::Internal("Embedded database not initialized".to_string()))?;

    let repo = LibSqlTenantRepository::new(db);
    let tenants = repo.list_tenants().await
        .map_err(|e| crate::error::AdminBffError::Internal(format!("Failed to list tenants: {}", e)))?;

    let view = TenantListView {
        total: tenants.len(),
        tenants: tenants.into_iter().map(|t| TenantItemView {
            id: t.id,
            name: t.name,
            created_at: t.created_at,
            member_count: 0, // TODO: requires user_tenant count query
        }).collect(),
    };

    Ok(Json(view))
}

pub fn router() -> Router<AdminBffState> {
    Router::new()
        .route("/api/admin/tenants", get(list_tenants))
}
