use contracts_api::{AdminDashboardStats, InitTenantResponse};
use reqwest::Client;
use serde::Serialize;

/// View model for admin dashboard — aggregates multiple service stats
#[derive(Serialize, utoipa::ToSchema)]
pub struct DashboardView {
    pub tenant_count: usize,
    pub total_counter_value: i64,
    pub recent_tenants: Vec<TenantSummaryView>,
    pub system_health: SystemHealthView,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TenantSummaryView {
    pub id: String,
    pub name: String,
    pub counter_value: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SystemHealthView {
    pub api_status: String,
    pub worker_status: String,
    pub database_status: String,
}

/// Aggregates dashboard stats from the internal API server
pub async fn fetch_dashboard(internal_api_url: &str) -> Result<DashboardView, crate::error::AdminBffError> {
    let client = Client::new();

    // Fetch admin stats from internal API
    let stats_resp = client
        .get(format!("{}/api/admin/stats", internal_api_url))
        .send()
        .await
        .map_err(|e| crate::error::AdminBffError::ServiceUnavailable(e.to_string()))?;

    let stats: AdminDashboardStats = stats_resp
        .json()
        .await
        .map_err(|e| crate::error::AdminBffError::Internal(e.to_string()))?;

    // Fetch tenant list
    let tenants_resp = client
        .get(format!("{}/api/tenant/list", internal_api_url))
        .send()
        .await
        .map_err(|e| crate::error::AdminBffError::ServiceUnavailable(e.to_string()))?;

    let tenants: Vec<InitTenantResponse> = tenants_resp
        .json()
        .await
        .unwrap_or_default();

    let view = DashboardView {
        tenant_count: stats.tenant_count as usize,
        total_counter_value: stats.counter_value,
        recent_tenants: tenants
            .into_iter()
            .take(10)
            .map(|t| TenantSummaryView {
                id: t.tenant_id.clone(),
                name: t.tenant_id,
                counter_value: 0, // Would need per-tenant counter query
            })
            .collect(),
        system_health: SystemHealthView {
            api_status: "healthy".to_string(),
            worker_status: "healthy".to_string(),
            database_status: "healthy".to_string(),
        },
    };

    Ok(view)
}
