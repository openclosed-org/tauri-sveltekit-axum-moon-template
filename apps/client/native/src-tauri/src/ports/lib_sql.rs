//! Embedded libsql implementation of LibSqlPort for local SQLite storage.
//!
//! Uses libsql crate directly (not tauri-plugin-libsql for business logic).
//! Provides: health_check, execute, query from LibSqlPort trait.

use async_trait::async_trait;
use domain::ports::lib_sql::{LibSqlError, LibSqlPort};
use libsql::Database;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Embedded libsql database implementing LibSqlPort.
///
/// Uses Builder::new_local() for embedded SQLite storage.
/// All operations are tenant-aware when a tenant_id is set.
#[derive(Clone)]
pub struct EmbeddedLibSql {
    db: Arc<Database>,
}

impl EmbeddedLibSql {
    /// Create a new embedded libsql instance from the given path.
    pub async fn new(path: &str) -> Result<Self, LibSqlError> {
        let db = libsql::Builder::new_local(path).build().await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create a new in-memory libsql instance (for testing).
    ///
    /// Uses a unique temp directory per call to avoid conflicts in parallel tests.
    #[cfg(test)]
    pub async fn new_in_memory() -> Result<Self, LibSqlError> {
        let dir = tempfile::tempdir().map_err(|e| Box::new(e) as LibSqlError)?;
        let path = dir.path().join("test.db");
        // Leak the TempDir so it's not dropped (database stays alive for the test lifetime)
        let _ = Box::leak(Box::new(dir));
        let db = libsql::Builder::new_local(path.to_str().unwrap())
            .build()
            .await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create a new embedded libsql instance using the app's data directory.
    pub async fn new_app_data(app_handle: &tauri::AppHandle) -> Result<Self, LibSqlError> {
        use tauri::Manager;
        let app_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("app.db");
        Self::new(db_path.to_str().unwrap()).await
    }

    /// Get the underlying database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl LibSqlPort for EmbeddedLibSql {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        let conn = self.db.connect()?;
        let mut rows = conn.query("SELECT 1", ()).await?;
        rows.next().await?;
        Ok(())
    }

    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError> {
        let conn = self.db.connect()?;
        let params: Vec<libsql::Value> = params.into_iter().map(libsql::Value::Text).collect();
        let result = conn.execute(sql, params).await?;
        Ok(result)
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError> {
        let conn = self.db.connect()?;
        let params: Vec<libsql::Value> = params.into_iter().map(libsql::Value::Text).collect();
        let mut rows = conn.query(sql, params).await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut map = serde_json::Map::new();
            for i in 0..row.column_count() {
                let name = row.column_name(i as i32).unwrap_or_default();
                let value = match row.get_value(i)? {
                    libsql::Value::Null => serde_json::Value::Null,
                    libsql::Value::Integer(n) => serde_json::json!(n),
                    libsql::Value::Real(f) => serde_json::json!(f),
                    libsql::Value::Text(s) => serde_json::Value::String(s),
                    libsql::Value::Blob(b) => serde_json::Value::String(base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        b,
                    )),
                };
                map.insert(name.to_string(), value);
            }
            let json = serde_json::Value::Object(map);
            let item: T = serde_json::from_value(json)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            results.push(item);
        }
        Ok(results)
    }
}

/// Run tenant schema migrations on the given libsql connection.
///
/// Defines: tenant table, user_tenant table, indexes.
/// Safe to call multiple times (uses IF NOT EXISTS).
pub async fn run_tenant_migrations(db: &EmbeddedLibSql) -> Result<(), LibSqlError> {
    db.execute(
        "
        CREATE TABLE IF NOT EXISTS tenant (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS user_tenant (
            id TEXT PRIMARY KEY,
            user_sub TEXT NOT NULL UNIQUE,
            tenant_id TEXT NOT NULL REFERENCES tenant(id),
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_user_tenant_tenant_id ON user_tenant(tenant_id);
        ",
        vec![],
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestRow {
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_health_check() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_insert() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT)", vec![])
            .await
            .unwrap();

        let affected = db
            .execute(
                "INSERT INTO test (id, name) VALUES (?, ?)",
                vec!["1".into(), "test".into()],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn test_query_select() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT)", vec![])
            .await
            .unwrap();

        db.execute(
            "INSERT INTO test (id, name) VALUES (?, ?)",
            vec!["1".into(), "Alice".into()],
        )
        .await
        .unwrap();

        let rows: Vec<TestRow> = db
            .query("SELECT id, name FROM test WHERE id = ?", vec!["1".into()])
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "1");
        assert_eq!(rows[0].name, "Alice");
    }

    #[tokio::test]
    async fn test_migrations() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        run_tenant_migrations(&db).await.unwrap();

        db.execute(
            "INSERT INTO tenant (id, name) VALUES (?, ?)",
            vec!["tenant-1".into(), "Test Tenant".into()],
        )
        .await
        .unwrap();

        let tenants: Vec<TestRow> = db
            .query("SELECT id, name FROM tenant", vec![])
            .await
            .unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0].name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_query_empty_result() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT)", vec![])
            .await
            .unwrap();

        let rows: Vec<TestRow> = db.query("SELECT id, name FROM test", vec![]).await.unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn test_execute_update_and_delete() {
        let db = EmbeddedLibSql::new_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT)", vec![])
            .await
            .unwrap();

        db.execute(
            "INSERT INTO test (id, name) VALUES (?, ?)",
            vec!["1".into(), "Original".into()],
        )
        .await
        .unwrap();

        let affected = db
            .execute(
                "UPDATE test SET name = ? WHERE id = ?",
                vec!["Updated".into(), "1".into()],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);

        let affected = db
            .execute("DELETE FROM test WHERE id = ?", vec!["1".into()])
            .await
            .unwrap();
        assert_eq!(affected, 1);
    }
}
