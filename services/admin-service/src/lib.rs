//! Admin domain service — dashboard stats aggregation.
//!
//! Admin is a composition layer with no independent domain entities.
//! It aggregates tenant and counter data to produce dashboard statistics.
//!
//! ## Architecture
//! ```text
//! domain/          → Admin domain errors
//! application/     → AdminDashboardService (composes tenant + counter services)
//! infrastructure/  → Adapters that bridge concrete service impls to admin traits
//! ```

pub mod application;
pub mod domain;
