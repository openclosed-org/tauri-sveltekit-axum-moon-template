//! Turso cloud database port implementation.
//!
//! Uses turso::sync::Builder for Turso cloud connections with sync support.
//! Implements the LibSqlPort trait for cloud-based SQLite.

use async_trait::async_trait;
use data_traits::ports::lib_sql::{LibSqlError, LibSqlPort};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use turso::Value;
use turso::sync::Builder as SyncBuilder;
use turso::sync::Database as SyncDatabase;

/// Turso cloud database implementing LibSqlPort.
///
/// Uses sync::Builder::new_remote() for Turso cloud connections.
/// Suitable for production deployments where local storage is not needed.
#[derive(Clone)]
pub struct TursoCloud {
    db: Arc<SyncDatabase>,
}

impl TursoCloud {
    /// Create a new Turso cloud database connection.
    ///
    /// # Arguments
    /// * `url` - Turso database URL (e.g., "libsql://your-db.turso.io")
    /// * `auth_token` - Turso authentication token
    pub async fn new(url: &str, auth_token: &str) -> Result<Self, LibSqlError> {
        let db = SyncBuilder::new_remote(":memory:")
            .with_remote_url(url)
            .with_auth_token(auth_token)
            .build()
            .await?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Get the underlying database reference.
    pub fn db(&self) -> &SyncDatabase {
        &self.db
    }
}

#[async_trait]
impl LibSqlPort for TursoCloud {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        let conn = self.db.connect().await?;
        conn.execute("SELECT 1", ()).await?;
        Ok(())
    }

    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError> {
        let conn = self.db.connect().await?;
        let values: Vec<Value> = params.into_iter().map(Value::Text).collect();
        let result = conn.execute(sql, values).await?;
        Ok(result)
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError> {
        let conn = self.db.connect().await?;
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

/// Run tenant schema migrations on the given Turso cloud connection.
///
/// Defines: tenant table, user_tenant table, indexes.
/// Safe to call multiple times (uses IF NOT EXISTS).
pub async fn run_tenant_migrations(db: &TursoCloud) -> Result<(), LibSqlError> {
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

    #[test]
    fn test_turso_cloud_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TursoCloud>();
    }
}
