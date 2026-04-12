//! Settings domain errors.

/// Domain-level error for settings operations.
#[derive(Debug, thiserror::Error)]
pub enum SettingsDomainError {
    #[error("Settings not found for user: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
