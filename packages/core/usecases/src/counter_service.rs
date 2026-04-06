//! Counter service — LibSQL-backed implementation.

use async_trait::async_trait;
use domain::ports::lib_sql::LibSqlPort;
use feature_counter::{CounterError, CounterService};

/// Counter table migration SQL.
pub const COUNTER_MIGRATION: &str = "CREATE TABLE IF NOT EXISTS counter (id INTEGER PRIMARY KEY, value INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL DEFAULT (datetime('now')))";

/// CounterService backed by LibSqlPort.
pub struct LibSqlCounterService<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> LibSqlCounterService<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

#[async_trait]
impl<P: LibSqlPort> CounterService for LibSqlCounterService<P> {
    async fn get_value(&self) -> Result<i64, CounterError> {
        let rows: Vec<(i64,)> = self
            .port
            .query("SELECT value FROM counter WHERE id = 1", vec![])
            .await
            .map_err(CounterError::Database)?;
        Ok(rows.first().map(|r| r.0).unwrap_or(0))
    }

    async fn increment(&self) -> Result<i64, CounterError> {
        self.port
            .execute(
                "INSERT INTO counter (id, value, updated_at) VALUES (1, 1, datetime('now')) ON CONFLICT(id) DO UPDATE SET value = value + 1, updated_at = datetime('now')",
                vec![],
            )
            .await
            .map_err(CounterError::Database)?;
        self.get_value().await
    }

    async fn decrement(&self) -> Result<i64, CounterError> {
        self.port
            .execute(
                "UPDATE counter SET value = value - 1, updated_at = datetime('now') WHERE id = 1",
                vec![],
            )
            .await
            .map_err(CounterError::Database)?;
        self.get_value().await
    }

    async fn reset(&self) -> Result<i64, CounterError> {
        self.port
            .execute(
                "UPDATE counter SET value = 0, updated_at = datetime('now') WHERE id = 1",
                vec![],
            )
            .await
            .map_err(CounterError::Database)?;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::ports::TenantId;
    use serde::de::DeserializeOwned;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    struct MockLibSqlPort {
        values: Arc<Mutex<HashMap<String, i64>>>,
    }

    impl MockLibSqlPort {
        fn new() -> Self {
            Self {
                values: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl LibSqlPort for MockLibSqlPort {
        async fn health_check(&self) -> Result<(), domain::ports::lib_sql::LibSqlError> {
            Ok(())
        }

        async fn execute(
            &self,
            sql: &str,
            params: Vec<String>,
        ) -> Result<u64, domain::ports::lib_sql::LibSqlError> {
            if params.is_empty() {
                return Ok(0);
            }

            let tenant_id = params[0].clone();
            let mut values = self.values.lock().await;

            if sql.contains("value = value + 1") {
                let entry = values.entry(tenant_id).or_insert(0);
                *entry += 1;
                return Ok(1);
            }

            if sql.contains("value = value - 1") {
                if let Some(entry) = values.get_mut(&tenant_id) {
                    *entry -= 1;
                }
                return Ok(1);
            }

            if sql.contains("value = 0") {
                if let Some(entry) = values.get_mut(&tenant_id) {
                    *entry = 0;
                }
                return Ok(1);
            }

            Ok(0)
        }

        async fn query<T: DeserializeOwned + Send + Sync>(
            &self,
            _sql: &str,
            params: Vec<String>,
        ) -> Result<Vec<T>, domain::ports::lib_sql::LibSqlError> {
            #[derive(serde::Serialize)]
            struct Row {
                value: i64,
            }

            let tenant_id = params
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            let values = self.values.lock().await;
            let value = values.get(&tenant_id).copied().unwrap_or(0);
            let rows = vec![Row { value }];
            let json = serde_json::to_value(rows).unwrap();
            let items: Vec<T> = serde_json::from_value(json)
                .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError)?;
            Ok(items)
        }
    }

    #[tokio::test]
    async fn tenant_a_increment_only_changes_tenant_a() {
        let service = LibSqlCounterService::new(MockLibSqlPort::new());
        let tenant_a = TenantId("tenant-a".into());
        let tenant_b = TenantId("tenant-b".into());

        let after_a = service.increment_for_tenant(&tenant_a).await.unwrap();
        let read_b = service.get_value_for_tenant(&tenant_b).await.unwrap();

        assert_eq!(after_a, 1, "tenant-a increment should produce 1");
        assert_eq!(read_b, 0, "tenant-b should remain unchanged");
    }

    #[tokio::test]
    async fn tenant_a_reset_only_resets_tenant_a() {
        let service = LibSqlCounterService::new(MockLibSqlPort::new());
        let tenant_a = TenantId("tenant-a".into());
        let tenant_b = TenantId("tenant-b".into());

        service.increment_for_tenant(&tenant_a).await.unwrap();
        service.increment_for_tenant(&tenant_b).await.unwrap();
        service.increment_for_tenant(&tenant_b).await.unwrap();

        let reset_a = service.reset_for_tenant(&tenant_a).await.unwrap();
        let read_a = service.get_value_for_tenant(&tenant_a).await.unwrap();
        let read_b = service.get_value_for_tenant(&tenant_b).await.unwrap();

        assert_eq!(reset_a, 0, "tenant-a reset must return baseline");
        assert_eq!(read_a, 0, "tenant-a should be reset to 0");
        assert_eq!(read_b, 2, "tenant-b value must remain unchanged");
    }

    #[tokio::test]
    async fn repeated_run_same_tenants_is_deterministic_after_reset() {
        let service = LibSqlCounterService::new(MockLibSqlPort::new());
        let tenant_a = TenantId("tenant-a".into());
        let tenant_b = TenantId("tenant-b".into());

        service.reset_for_tenant(&tenant_a).await.unwrap();
        service.reset_for_tenant(&tenant_b).await.unwrap();
        service.increment_for_tenant(&tenant_a).await.unwrap();

        let run1_a = service.get_value_for_tenant(&tenant_a).await.unwrap();
        let run1_b = service.get_value_for_tenant(&tenant_b).await.unwrap();

        service.reset_for_tenant(&tenant_a).await.unwrap();
        service.reset_for_tenant(&tenant_b).await.unwrap();
        service.increment_for_tenant(&tenant_a).await.unwrap();

        let run2_a = service.get_value_for_tenant(&tenant_a).await.unwrap();
        let run2_b = service.get_value_for_tenant(&tenant_b).await.unwrap();

        assert_eq!(run1_a, 1, "run-1 tenant-a baseline mismatch");
        assert_eq!(run1_b, 0, "run-1 tenant-b baseline mismatch");
        assert_eq!(run2_a, 1, "run-2 tenant-a baseline mismatch");
        assert_eq!(run2_b, 0, "run-2 tenant-b baseline mismatch");
    }
}
