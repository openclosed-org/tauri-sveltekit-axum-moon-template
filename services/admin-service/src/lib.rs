//! Admin domain service — dashboard stats aggregation.
//!
//! Admin is a composition layer with no independent domain entities.
//! It aggregates tenant and counter data to produce dashboard statistics.
//!
//! ## Architecture
//! ```text
//! domain/          → Admin domain errors and types
//! application/     → AdminDashboardService (orchestrates use cases)
//! ports/           → Abstract interfaces for external dependencies
//! infrastructure/  → Adapters that implement ports using concrete services
//! contracts/       → DTO re-exports from packages/contracts/
//! events/          → Domain events
//! policies/        → Timeout, retry, idempotency policies
//! ```

pub mod application;
pub mod contracts;
pub mod domain;
pub mod events;
pub mod infrastructure;
pub mod policies;
pub mod ports;
