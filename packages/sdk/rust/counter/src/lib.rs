//! SDK Counter — client-side interface for counter operations.
//!
//! This crate defines the **contract** that all counter backends must satisfy.
//! App shells (desktop, web, mobile) depend only on this crate, never on
//! `counter-service` directly.
//!
//! ## Backends
//! - `sdk-counter-embedded` — embedded Turso (desktop offline-first)
//! - (future) `sdk-counter-http` — HTTP client to BFF (web)

pub mod service;
pub mod types;

pub use service::CounterService;
pub use types::{CounterError, CounterId};
