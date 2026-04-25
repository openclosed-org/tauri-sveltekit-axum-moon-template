//! Event Bus — unified outbox and inter-service communication.
//!
//! ## Architecture
//! ```text
//! ┌────────────────────────────────────────────┐
//! │  ports/         (EventBus trait)            │  ← Services depend on this
//! ├────────────────────────────────────────────┤
//! │  adapters/      (InMemoryEventBus)         │  ← In-process
//! │                 (NatsEventBus)              │  ← Distributed
//! ├────────────────────────────────────────────┤
//! │  outbox/        (event_outbox schema +     │  ← Unified outbox persistence surface
//! │                  OutboxEntry)               │
//! └────────────────────────────────────────────┘
//! ```
//!
//! ## Outbox
//! The `event_outbox` table is the **single** event persistence surface for
//! all services. No per-service private outbox tables. See `outbox::outbox_entry`
//! for the schema definition.
//!
//! Canonical relay path:
//! `event_outbox -> workers/outbox-relay -> event backbone -> consumers`
//!
//! `packages/messaging` owns the shared event contracts and EventBus abstraction,
//! but it is not the canonical home of the background relay loop.
//!
//! ## Feature flags
//! - `memory` (default) — in-memory event bus via tokio broadcast channels
//! - `nats` (future) — NATS JetStream implementation for production

#![deny(unused_imports, unused_variables)]

pub mod adapters;
pub mod application;
pub mod contracts;
pub mod domain;
pub mod events;
pub mod outbox;
pub mod policies;
pub mod ports;
