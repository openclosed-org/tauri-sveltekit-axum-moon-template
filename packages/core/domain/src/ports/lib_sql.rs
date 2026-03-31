//! libsql local database Port trait.
//!
//! Used by native-tauri (Tauri app) for local embedded storage.
//! Uses standard SQLite-compatible SQL.

use async_trait::async_trait;
use serde::de::DeserializeOwned;

/// Error type for libsql operations.
pub type LibSqlError = Box<dyn std::error::Error + Send + Sync>;

/// libsql port — abstracts local embedded database access.
///
/// Implementations live in native-tauri crate.
/// Uses standard SQLite SQL (unlike SurrealDB's SurrealQL).
#[async_trait]
pub trait LibSqlPort: Send + Sync {
    /// Verify the database connection is alive.
    async fn health_check(&self) -> Result<(), LibSqlError>;

    /// Execute a SQL statement (INSERT, UPDATE, DELETE, DDL).
    /// Returns the number of affected rows.
    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError>;

    /// Execute a SQL query (SELECT) returning deserialized rows.
    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError>;
}
