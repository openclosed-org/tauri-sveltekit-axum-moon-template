//! Application layer — use case orchestration.

pub mod service;

pub use service::{TenantService, TenantServiceError, TenantServiceTrait};
