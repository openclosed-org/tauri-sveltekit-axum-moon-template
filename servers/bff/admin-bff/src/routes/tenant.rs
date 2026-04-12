use axum::{
    routing::get,
    Json,
    Router,
    extract::State,
};
use serde::Serialize;
use utoipa::OpenApi;
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
    State(_state): State<AdminBffState>,
) -> AdminBffResult<Json<TenantListView>> {
    // Placeholder — in production, call internal API /api/tenant/list
    let view = TenantListView {
        tenants: vec![],
        total: 0,
    };
    Ok(Json(view))
}

pub fn router() -> Router<AdminBffState> {
    Router::new()
        .route("/api/admin/tenants", get(list_tenants))
}
