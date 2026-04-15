//! Integration tests for counter-service with real libsql port.
//!
//! These tests verify the full stack: application service → repository → libsql.

use chrono::{DateTime, Utc};
use counter_service::application::{RepositoryBackedCounterService, TenantScopedCounterService};
use counter_service::contracts::service::CounterService;
use counter_service::infrastructure::LibSqlCounterRepository;
use counter_service::ports::{CounterRepository, RepositoryError};
use data::ports::lib_sql::{LibSqlError, LibSqlPort};
use kernel::TenantId;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Minimal libsql port for testing — wraps rusqlite.
struct InMemoryLibSqlPort {
    conn: Arc<Mutex<Connection>>,
}

impl InMemoryLibSqlPort {
    fn new() -> Self {
        let conn = Connection::open_in_memory().expect("failed to open in-memory sqlite");
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

#[async_trait::async_trait]
impl LibSqlPort for InMemoryLibSqlPort {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        let conn = self.conn.lock().await;
        conn.execute("SELECT 1", [])
            .map_err(|e| Box::new(e) as LibSqlError)?;
        Ok(())
    }

    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError> {
        let conn = self.conn.lock().await;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let affected = conn
            .execute(sql, rusqlite::params_from_iter(param_refs.into_iter()))
            .map_err(|e| Box::new(e) as LibSqlError)?;
        Ok(affected as u64)
    }

    async fn query<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError> {
        let conn = self.conn.lock().await;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let mut stmt = conn.prepare(sql).map_err(|e| Box::new(e) as LibSqlError)?;

        let columns = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let rows: Vec<serde_json::Value> = stmt
            .query_map(rusqlite::params_from_iter(param_refs.into_iter()), |row| {
                let mut map = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    // Handle different SQLite types
                    let value: serde_json::Value = match row.get_ref(i)? {
                        rusqlite::types::ValueRef::Integer(v) => {
                            serde_json::Value::Number(serde_json::Number::from(v))
                        }
                        rusqlite::types::ValueRef::Text(v) => {
                            let s = String::from_utf8_lossy(v).to_string();
                            serde_json::Value::String(s)
                        }
                        rusqlite::types::ValueRef::Real(v) => serde_json::Number::from_f64(v)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null),
                        rusqlite::types::ValueRef::Blob(v) => serde_json::Value::Array(
                            v.iter().map(|b| serde_json::json!(*b)).collect(),
                        ),
                        rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    };
                    map.insert(col.clone(), value);
                }
                Ok(serde_json::Value::Object(map))
            })
            .map_err(|e| Box::new(e) as LibSqlError)?
            .filter_map(|r| r.ok())
            .collect();

        let json = serde_json::to_value(&rows).map_err(|e| Box::new(e) as LibSqlError)?;
        let items: Vec<T> = serde_json::from_value(json).map_err(|e| Box::new(e) as LibSqlError)?;
        Ok(items)
    }
}

#[tokio::test]
async fn full_stack_increment_reset_increment() {
    let port = InMemoryLibSqlPort::new();
    let repo = LibSqlCounterRepository::new(port);
    repo.migrate().await.unwrap();

    let service = TenantScopedCounterService::new(repo);
    let tenant = TenantId("test-tenant".into());

    let v1 = service.increment(&tenant, None).await.unwrap();
    assert_eq!(v1, 1);

    let v2 = service.increment(&tenant, None).await.unwrap();
    assert_eq!(v2, 2);

    let r = service.reset(&tenant, None).await.unwrap();
    assert_eq!(r, 0);

    let v3 = service.increment(&tenant, None).await.unwrap();
    assert_eq!(v3, 1);
}

#[tokio::test]
async fn full_stack_tenant_isolation() {
    let port = InMemoryLibSqlPort::new();
    let repo = LibSqlCounterRepository::new(port);
    repo.migrate().await.unwrap();

    let service = TenantScopedCounterService::new(repo);
    let a = TenantId("tenant-a".into());
    let b = TenantId("tenant-b".into());

    service.increment(&a, None).await.unwrap();
    service.increment(&a, None).await.unwrap();
    service.increment(&b, None).await.unwrap();

    assert_eq!(service.get_value(&a).await.unwrap(), 2);
    assert_eq!(service.get_value(&b).await.unwrap(), 1);
}
