//! Memory adapters — in-memory implementations of all runtime ports.
//!
//! These adapters provide fully functional in-memory implementations for:
//! - Unit testing services without external dependencies
//! - Local development without infrastructure
//! - Integration testing with minimal setup
//!
//! ## Usage
//! ```ignore
//! use runtime::adapters::memory::*;
//!
//! let pubsub = MemoryPubSub::new();
//! let state = MemoryState::new();
//! let invocation = MemoryInvocation::new();
//! let workflow = MemoryWorkflow::new();
//! let lock = MemoryLock::new();
//! let binding = MemoryBinding::new(RuntimeMode::Memory);
//! let secret = MemorySecret::new();
//! let queue = MemoryQueue::new();
//! ```

pub mod invocation;
pub mod pubsub;
pub mod state;
pub mod workflow;
pub mod lock;
pub mod binding;
pub mod secret;
pub mod queue;

// Re-export all memory adapters for convenient importing.
pub use binding::MemoryBinding;
pub use invocation::MemoryInvocation;
pub use lock::MemoryLock;
pub use pubsub::MemoryPubSub;
pub use queue::MemoryQueue;
pub use secret::MemorySecret;
pub use state::MemoryState;
pub use workflow::MemoryWorkflow;
