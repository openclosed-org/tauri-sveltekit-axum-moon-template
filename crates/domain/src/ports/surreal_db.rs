//! SurrealDB server-side Port trait.
//!
//! Used by runtime_server (Axum) to communicate with SurrealDB instance.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

/// Error type for SurrealDB operations.
pub type SurrealError = Box<dyn std::error::Error + Send + Sync>;

/// SurrealDB port — abstracts server-side SurrealDB access.
///
/// Implementations live in runtime_server crate.
/// SurrealDB uses SurrealQL (NOT standard SQL), hence a separate trait from LibSqlPort.
#[async_trait]
pub trait SurrealDbPort: Send + Sync {
    /// Verify the database connection is alive.
    async fn health_check(&self) -> Result<(), SurrealError>;

    /// Execute a SurrealQL query returning deserialized records.
    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        vars: BTreeMap<String, surrealdb::types::Value>,
    ) -> Result<Vec<T>, SurrealError>;
}
