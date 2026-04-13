//! Counter application layer — use case orchestration.
//!
//! This module implements `feature_counter::CounterService` by composing
//! the `CounterRepository` port. It contains NO direct database calls —
//! all storage goes through the repository abstraction.

pub mod service;

pub use service::*;

