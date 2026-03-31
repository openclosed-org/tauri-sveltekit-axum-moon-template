//! Turso cloud database port implementation.
//!
//! Uses libsql::Builder::new_remote() for Turso cloud connections.
//! Implements the LibSqlPort trait for cloud-based SQLite.

use async_trait::async_trait;
use domain::ports::lib_sql::{LibSqlError, LibSqlPort};
use libsql::Database;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Turso cloud database implementing LibSqlPort.
///
/// Uses Builder::new_remote() for Turso cloud connections.
/// Suitable for production deployments where local storage is not needed.
#[derive(Clone)]
pub struct TursoDb {
    db: Arc<Database>,
}

impl TursoDb {
    /// Create a new Turso database connection.
    ///
    /// # Arguments
    /// * `url` - Turso database URL (e.g., "libsql://your-db.turso.io")
    /// * `auth_token` - Turso authentication token
    pub async fn new(url: &str, auth_token: &str) -> Result<Self, LibSqlError> {
        let db = libsql::Builder::new_remote(url.to_string(), auth_token.to_string())
            .build()
            .await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Get the underlying database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl LibSqlPort for TursoDb {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        let conn = self.db.connect()?;
        conn.execute("SELECT 1", ()).await?;
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

/// Run tenant schema migrations on the given Turso connection.
///
/// Defines: tenant table, user_tenant table, indexes.
/// Safe to call multiple times (uses IF NOT EXISTS).
pub async fn run_tenant_migrations(db: &TursoDb) -> Result<(), LibSqlError> {
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

    #[test]
    fn test_turso_db_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TursoDb>();
    }
}
