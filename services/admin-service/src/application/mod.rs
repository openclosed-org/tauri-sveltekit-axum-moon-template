//! Admin service application layer — dashboard stats aggregation.
//!
//! This module orchestrates use cases by coordinating the port interfaces.
//! It contains NO direct dependencies on other services — all external
//! access goes through abstract ports.

use async_trait::async_trait;
use feature_admin::{AdminDashboardStats, AdminError, AdminService};
use kernel::TenantId;

use crate::ports::{CounterRepository, TenantRepository, TenantSummary};

/// AdminDashboardService implements the AdminService trait by coordinating
/// the TenantRepository and CounterRepository ports.
pub struct AdminDashboardService<T: TenantRepository, C: CounterRepository> {
    tenant_repo: T,
    counter_repo: C,
}

impl<T: TenantRepository, C: CounterRepository> AdminDashboardService<T, C> {
    pub fn new(tenant_repo: T, counter_repo: C) -> Self {
        Self {
            tenant_repo,
            counter_repo,
        }
    }
}

#[async_trait]
impl<T: TenantRepository, C: CounterRepository> AdminService for AdminDashboardService<T, C> {
    async fn get_dashboard_stats(&self) -> Result<AdminDashboardStats, AdminError> {
        let tenants = self
            .tenant_repo
            .list_tenants()
            .await
            .map_err(|e| AdminError::Database(e))?;

        // Get counter value — try tenant-scoped first, fall back to global
        let counter_value = if tenants.is_empty() {
            // No tenants, get global counter
            self.counter_repo
                .get_global_value()
                .await
                .map_err(|e| AdminError::Counter(e.to_string()))?
        } else {
            // Get counter for first tenant as a representative metric
            let tenant_id = TenantId(tenants[0].id.clone());
            self.counter_repo
                .get_value(&tenant_id)
                .await
                .map_err(|e| AdminError::Counter(e.to_string()))?
        };

        Ok(AdminDashboardStats {
            tenant_count: tenants.len() as u64,
            counter_value,
            last_login: tenants.first().map(|t| t.created_at.clone()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock tenant repository for testing.
    struct MockTenantRepo;

    #[async_trait]
    impl TenantRepository for MockTenantRepo {
        async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![
                TenantSummary {
                    id: "tenant-1".to_string(),
                    name: "Test Tenant".to_string(),
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                },
                TenantSummary {
                    id: "tenant-2".to_string(),
                    name: "Another Tenant".to_string(),
                    created_at: "2024-01-02T00:00:00Z".to_string(),
                },
            ])
        }
    }

    /// Mock counter repository for testing.
    struct MockCounterRepo;

    #[async_trait]
    impl CounterRepository for MockCounterRepo {
        async fn get_value(&self, _tenant_id: &TenantId) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(42)
        }

        async fn get_global_value(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(99)
        }
    }

    #[tokio::test]
    async fn test_dashboard_stats() {
        let service = AdminDashboardService::new(MockTenantRepo, MockCounterRepo);
        let stats = service.get_dashboard_stats().await.unwrap();

        assert_eq!(stats.tenant_count, 2);
        assert_eq!(stats.counter_value, 42);
        assert_eq!(stats.last_login, Some("2024-01-01T00:00:00Z".to_string()));
        assert!(!stats.app_version.is_empty());
    }

    #[tokio::test]
    async fn test_dashboard_stats_no_tenants() {
        struct EmptyTenantRepo;

        #[async_trait]
        impl TenantRepository for EmptyTenantRepo {
            async fn list_tenants(&self) -> Result<Vec<TenantSummary>, Box<dyn std::error::Error + Send + Sync>> {
                Ok(vec![])
            }
        }

        let service = AdminDashboardService::new(EmptyTenantRepo, MockCounterRepo);
        let stats = service.get_dashboard_stats().await.unwrap();

        assert_eq!(stats.tenant_count, 0);
        assert_eq!(stats.counter_value, 99); // Global counter
        assert_eq!(stats.last_login, None);
    }
}
