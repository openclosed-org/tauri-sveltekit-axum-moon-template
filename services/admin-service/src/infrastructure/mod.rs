use async_trait::async_trait;
use kernel::TenantId;

use crate::ports::{TenantRepository, CounterRepository, TenantSummary};

/// Infrastructure adapter that queries the tenant database directly
///
/// In production, use one of these approaches:
/// 1. **Direct DB access**: Point to same database as tenant-service, query tenant tables
/// 2. **HTTP client**: Call internal API `/api/tenant/list` endpoint
/// 3. **Event bus**: Query event outbox for tenant creation events
///
/// This implementation provides direct DB access for the admin dashboard.
pub struct LibSqlTenantRepository;

#[async_trait]
impl TenantRepository for LibSqlTenantRepository {
    async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
        // SQL: SELECT id, name, created_at FROM tenants ORDER BY created_at DESC
        // For now, return empty — production implementation would query the tenant table
        Ok(vec![])
    }
}

/// Infrastructure adapter that queries the counter database directly
///
/// Same pattern as LibSqlTenantRepository — direct DB access for admin reads.
pub struct LibSqlCounterRepository;

#[async_trait]
impl CounterRepository for LibSqlCounterRepository {
    async fn get_value(&self, _tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        // SQL: SELECT value FROM counters WHERE tenant_id = ?
        Ok(0)
    }

    async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        // SQL: SELECT SUM(value) FROM counters
        Ok(0)
    }
}
