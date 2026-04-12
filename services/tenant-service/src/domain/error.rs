//! Tenant domain errors.

/// Error type for tenant domain operations.
#[derive(Debug, thiserror::Error)]
pub enum TenantDomainError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
