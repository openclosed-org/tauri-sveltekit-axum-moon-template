//! Event Bus — inter-service communication via events.
//!
//! ## Architecture
//! ```text
//! ┌──────────────────────────────────────┐
//! │  ports/       (EventBus trait)       │  ← What services depend on
//! ├──────────────────────────────────────┤
//! │  adapters/    (InMemoryEventBus)     │  ← Phase 1 implementation
//! │               (NatsEventBus)         │  ← Phase 2 (future)
//! ├──────────────────────────────────────┤
//! │  outbox/      (OutboxEntry +         │  ← Guaranteed delivery
//! │               OutboxPublisher)       │
//! └──────────────────────────────────────┘
//! ```
//!
//! ## Feature flags
//! - `memory` (default) — in-memory event bus via tokio broadcast channels
//! - `nats` (future) — NATS JetStream implementation for production
//!
//! ## Usage
//! ```ignore
//! use event_bus::ports::{EventBus, EventEnvelope};
//! use event_bus::adapters::memory_bus::InMemoryEventBus;
//!
//! let bus = InMemoryEventBus::new();
//! bus.publish(EventEnvelope::new(
//!     AppEvent::CounterChanged(CounterChanged { ... }),
//!     "counter-service",
//! )).await?;
//! ```

pub mod adapters;
pub mod application;
pub mod contracts;
pub mod domain;
pub mod events;
pub mod outbox;
pub mod policies;
pub mod ports;
