//! Outbox pattern for reliable event delivery.
//!
//! ## Problem
//! When a service writes to its database AND publishes an event, these are
//! two separate operations. If the event publish fails after the DB commit,
//! other services never learn about the change.
//!
//! ## Solution
//! 1. Write business data AND event record in a single DB transaction
//! 2. The canonical `workers/outbox-relay` background worker reads unprocessed outbox rows and publishes them
//! 3. Once published, the outbox row is marked as "processed"
//!
//! This guarantees at-least-once delivery (events may be published twice
//! in crash scenarios — consumers must be idempotent).

mod outbox_entry;
pub use outbox_entry::*;
