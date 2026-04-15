//! EventBus port — publish and subscribe to domain events.
//!
//! This trait is the **only** way services emit or consume events.
//! Implementations include in-memory channels (Phase 1) and NATS JetStream (Phase 2).

mod event_bus;

pub use event_bus::*;
