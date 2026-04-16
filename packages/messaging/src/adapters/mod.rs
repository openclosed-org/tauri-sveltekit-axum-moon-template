//! In-memory EventBus implementation using tokio broadcast channels.
//!
//! ## Characteristics
//! - **Zero external dependencies** — uses tokio's built-in broadcast channels
//! - **Bounded buffer** — events are dropped if no subscriber can keep up
//! - **Synchronous dispatch** — handlers run on the publish task, not spawned
//!
//! ## When to use
//! - Phase 1: Development and testing
//! - Single-process deployments
//!
//! ## When NOT to use
//! - Multi-process or distributed deployments
//! - When event durability is required (use NATS + Outbox instead)

pub mod memory_bus;
pub mod nats_bus;

pub use memory_bus::*;
pub use nats_bus::*;
