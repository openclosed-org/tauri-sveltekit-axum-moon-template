//! Runtime — distributed system runtime abstraction layer.
//!
//! This crate provides port-based abstractions for distributed system primitives
//! like messaging, state management, workflows, locking, and service invocation.
//!
//! ## Architecture
//! ```text
//! ┌──────────────────────────────────────────┐
//! │  ports/       (trait definitions)        │  ← Services depend on these
//! ├──────────────────────────────────────────┤
//! │  adapters/                               │
//! │    ├── memory/   (in-memory for tests)   │  ← Test implementations
//! │    ├── direct/   (in-process direct)     │  ← Single-node deployments
//! │    └── dapr/     (Dapr sidecar)          │  ← Distributed deployments
//! └──────────────────────────────────────────┘
//! ```
//!
//! ## Ports
//! - `invocation` — Synchronous request/response service calls
//! - `pubsub` — Publish/subscribe messaging
//! - `state` — Key-value state persistence
//! - `workflow` — Long-running orchestration
//! - `lock` — Distributed locking
//! - `binding` — Service wiring and dependency injection
//! - `secret` — Secure credential management
//! - `queue` — Persistent message queues
//!
//! ## Design principles
//! - Services depend on ports, NOT on concrete implementations
//! - Adapters implement ports for specific backends
//! - Memory adapters enable testing without external dependencies
//!
//! ## Usage
//! ```ignore
//! use runtime::ports::{PubSub, State, Invocation};
//! use runtime::adapters::memory::{MemoryPubSub, MemoryState, MemoryInvocation};
//!
//! // For testing
//! let pubsub: Box<dyn PubSub> = Box::new(MemoryPubSub::new());
//! let state: Box<dyn State> = Box::new(MemoryState::new());
//! let invocation: Box<dyn Invocation> = Box::new(MemoryInvocation::new());
//! ```

pub mod ports;
pub mod adapters;
