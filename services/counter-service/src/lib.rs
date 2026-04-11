//! Counter domain service — counting, statistics, analytics.
//!
//! ## Status
//! - [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
//! - [ ] Phase 1: Implement domain/application/ports
//! - [ ] Phase 2: Independent deployment
//!
//! ## Architecture
//! - `domain/` — Counter entity, value objects, invariants
//! - `application/` — Use cases (increment, decrement, reset)
//! - `ports/` — External dependency abstractions (CounterRepository)
//! - `contracts/` — Stable contract definitions
//! - `sync/` — OfflineFirst sync strategies
//! - `infrastructure/` — Database implementations
//! - `interfaces/` — API route handlers

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
