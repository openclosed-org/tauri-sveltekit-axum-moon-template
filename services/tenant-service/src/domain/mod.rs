//! Tenant domain module.

pub mod entity;
pub mod error;

pub use entity::{CreateTenantInput, Tenant};
pub use error::TenantDomainError;
