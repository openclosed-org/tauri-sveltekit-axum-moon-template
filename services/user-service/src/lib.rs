//! User domain service — authentication, profiles, permissions, sessions.
//!
//! ## Status
//! - [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
//! - [ ] Phase 1: Implement domain/application/ports
//! - [ ] Phase 2: Independent deployment
//!
//! ## Architecture
//! - `domain/` — Pure domain logic (User entity, value objects, invariants)
//! - `application/` — Use cases (register, login, update_profile, etc.)
//! - `ports/` — External dependency abstractions ( UserRepository, SessionStore)
//! - `contracts/` — Stable contract definitions (DTOs, events)
//! - `sync/` — OfflineFirst sync strategies
//! - `infrastructure/` — External service integrations (email, SMS, OAuth providers)
//! - `interfaces/` — API route handlers and request/response adapters

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
