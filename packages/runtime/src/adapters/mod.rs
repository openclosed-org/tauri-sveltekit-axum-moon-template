//! Runtime adapters — concrete implementations of runtime ports.
//!
//! Adapters are grouped by implementation type:
//! - `memory/` — In-memory implementations for testing
//! - `direct/` — Direct in-process calls (single-node deployments)
//! - `dapr/` — Dapr sidecar adapter (distributed deployments)

pub mod memory;
pub mod nats;
