//! Admin domain service — dashboard stats aggregation.
//!
//! Admin has no independent domain entities; it composes tenant and counter
//! services to produce `AdminDashboardStats`.

/// Admin-specific domain errors.
#[derive(Debug, thiserror::Error)]
pub enum AdminDomainError {
    #[error("Aggregation error: {0}")]
    AggregationError(String),
}
