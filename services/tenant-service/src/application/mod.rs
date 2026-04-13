//! Application layer — use case orchestration.

pub mod service;

pub use service::{InitTenantResult, TenantService, TenantServiceError, TenantServiceTrait};
