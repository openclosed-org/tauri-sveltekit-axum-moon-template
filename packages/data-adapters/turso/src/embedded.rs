//! Embedded Turso database implementation of LibSqlPort for local SQLite storage.
//!
//! Uses turso crate directly for embedded SQLite storage.
//! Provides: health_check, execute, query from LibSqlPort trait.

use async_trait::async_trait;
use data_traits::ports::lib_sql::{LibSqlError, LibSqlPort};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use turso::{Builder, Database, Value};

/// Embedded Turso database implementing LibSqlPort.
///
/// Uses Builder::new_local() for embedded SQLite storage.
/// All operations are tenant-aware when a tenant_id is set.
#[derive(Clone)]
pub struct EmbeddedTurso {
    db: Arc<Database>,
}

impl EmbeddedTurso {
    fn normalize_local_path(path: &str) -> &str {
        path.strip_prefix("libsql://file:")
            .or_else(|| path.strip_prefix("file:"))
            .unwrap_or(path)
    }

    /// Create a new embedded Turso instance from the given path.
    pub async fn new(path: &str) -> Result<Self, LibSqlError> {
        let db = Builder::new_local(Self::normalize_local_path(path))
            .build()
            .await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create a new in-memory Turso instance (for testing).
    ///
    /// Uses a unique temp directory per call to avoid conflicts in parallel tests.
    #[cfg(test)]
    pub async fn new_in_memory() -> Result<Self, LibSqlError> {
        let dir = tempfile::tempdir().map_err(|e| Box::new(e) as LibSqlError)?;
        let path = dir.path().join("test.db");
        // Leak the TempDir so it's not dropped (database stays alive for the test lifetime)
        let _ = Box::leak(Box::new(dir));
        let db = Builder::new_local(path.to_str().unwrap()).build().await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Get the underlying database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl LibSqlPort for EmbeddedTurso {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        let conn = self.db.connect()?;
        let mut rows = conn.query("SELECT 1", ()).await?;
        rows.next().await?;
        Ok(())
    }

    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError> {
        let conn = self.db.connect()?;
        let values: Vec<Value> = params.into_iter().map(Value::Text).collect();
        let result = conn.execute(sql, values).await?;
        Ok(result)
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError> {
        let conn = self.db.connect()?;
        let values: Vec<Value> = params.into_iter().map(Value::Text).collect();
        let mut rows = conn.query(sql, values).await?;

        let column_names: Vec<String> = (0..rows.column_count())
            .map(|i| rows.column_name(i).unwrap_or_default())
            .collect();

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut map = serde_json::Map::new();
            for (i, name) in column_names.iter().enumerate() {
                let value = match row.get_value(i)? {
                    Value::Null => serde_json::Value::Null,
                    Value::Integer(n) => serde_json::json!(n),
                    Value::Real(f) => serde_json::json!(f),
                    Value::Text(s) => serde_json::Value::String(s),
                    Value::Blob(b) => serde_json::Value::String(base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        b,
                    )),
                };
                map.insert(name.clone(), value);
            }
            let json = serde_json::Value::Object(map);
            let item: T = serde_json::from_value(json)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            results.push(item);
        }
        Ok(results)
    }
}

/// Run tenant schema migrations on the given Turso connection.
///
/// Defines: tenant table, user_tenant table, indexes.
/// Safe to call multiple times (uses IF NOT EXISTS).
pub async fn run_tenant_migrations(db: &EmbeddedTurso) -> Result<(), LibSqlError> {
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
        CREATE INDEX IF NOT EXISTS idx_user_tenant_user_sub ON user_tenant(user_sub);
        ",
        vec![],
    )
    .await?;

    db.execute("ALTER TABLE user_tenant ADD COLUMN joined_at TEXT", vec![])
        .await
        .ok();

    db.execute(
        "UPDATE user_tenant SET joined_at = datetime('now') WHERE joined_at IS NULL OR joined_at = ''",
        vec![],
    )
    .await
    .ok();

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
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_insert() {
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
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
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
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
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
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

    #[derive(Debug, Deserialize)]
    struct UserTenantRow {
        joined_at: String,
    }

    #[tokio::test]
    async fn test_migrations_backfill_existing_user_tenant_joined_at() {
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
        db.execute(
            "CREATE TABLE tenant (id TEXT PRIMARY KEY, name TEXT NOT NULL)",
            vec![],
        )
        .await
        .unwrap();
        db.execute(
            "CREATE TABLE user_tenant (id TEXT PRIMARY KEY, user_sub TEXT NOT NULL UNIQUE, tenant_id TEXT NOT NULL REFERENCES tenant(id), role TEXT NOT NULL DEFAULT 'member')",
            vec![],
        )
        .await
        .unwrap();
        db.execute(
            "INSERT INTO tenant (id, name) VALUES (?, ?)",
            vec!["tenant-1".into(), "Test Tenant".into()],
        )
        .await
        .unwrap();
        db.execute(
            "INSERT INTO user_tenant (id, user_sub, tenant_id, role) VALUES (?, ?, ?, ?)",
            vec![
                "binding-1".into(),
                "user-1".into(),
                "tenant-1".into(),
                "owner".into(),
            ],
        )
        .await
        .unwrap();

        run_tenant_migrations(&db).await.unwrap();

        let rows: Vec<UserTenantRow> = db
            .query(
                "SELECT joined_at FROM user_tenant WHERE id = ?",
                vec!["binding-1".into()],
            )
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].joined_at.is_empty());
    }

    #[tokio::test]
    async fn test_query_empty_result() {
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
        db.execute("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT)", vec![])
            .await
            .unwrap();

        let rows: Vec<TestRow> = db.query("SELECT id, name FROM test", vec![]).await.unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn test_execute_update_and_delete() {
        let db = EmbeddedTurso::new_in_memory().await.unwrap();
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

    #[tokio::test]
    async fn normalizes_file_url_paths() {
        assert_eq!(
            EmbeddedTurso::normalize_local_path("file:/tmp/test.db"),
            "/tmp/test.db"
        );
        assert_eq!(
            EmbeddedTurso::normalize_local_path("libsql://file:/tmp/test.db"),
            "/tmp/test.db"
        );
        assert_eq!(
            EmbeddedTurso::normalize_local_path("/tmp/test.db"),
            "/tmp/test.db"
        );
    }
}
