//! Infrastructure adapters — concrete repository implementations.

pub mod libsql_adapter;
pub mod surrealdb_adapter;

pub use libsql_adapter::LibSqlTenantRepository;
pub use surrealdb_adapter::SurrealDbTenantRepository;
