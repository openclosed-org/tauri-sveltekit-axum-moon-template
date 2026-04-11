//! Event bus abstraction — EventBus trait, Outbox pattern, inter-service communication.
//!
//! ## Status
//! - [ ] Phase 0: Stub — no implementation
//! - [ ] Phase 1: Implement in-memory event bus (dev/test)
//! - [ ] Phase 2: NATS JetStream implementation (production)
//!
//! ## Architecture
//! - `ports/` — EventBus trait definition
//! - `adapters/` — Concrete implementations (memory for dev, NATS for prod)
//! - `outbox/` — Outbox pattern implementation for reliable delivery

pub mod adapters;
pub mod outbox;
pub mod ports;
