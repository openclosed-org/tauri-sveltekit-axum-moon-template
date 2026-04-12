//! Tenant domain service — multi-tenant isolation, member management, tenant lifecycle.
//!
//! ## Architecture
//! ```text
//! domain/          → Tenant entity, CreateTenantInput, errors (zero deps)
//! ports/           → TenantRepository trait (storage abstraction)
//! application/     → TenantService (orchestrates via ports)
//! infrastructure/  → LibSqlTenantRepository, SurrealDbTenantRepository
//! contracts/       → DTO re-exports from packages/contracts/
//! sync/            → OfflineFirst sync strategies
//! ```

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod ports;
pub mod sync;
