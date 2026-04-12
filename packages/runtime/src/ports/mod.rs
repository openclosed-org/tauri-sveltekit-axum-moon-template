//! Runtime ports — distributed system runtime abstraction traits.
//!
//! These traits define the interfaces that services depend on.
//! Concrete implementations are provided by adapters.

pub mod invocation;
pub mod pubsub;
pub mod state;
pub mod workflow;
pub mod lock;
pub mod binding;
pub mod secret;
pub mod queue;

// Re-export all port traits for convenient importing.
pub use binding::{Binding, BindingError, CapabilityBinding, RuntimeMode};
pub use invocation::{Invocation, InvocationContext, InvocationError, InvocationRequest, InvocationResponse};
pub use lock::{Lock, LockError, LockGuard, LockManager};
pub use pubsub::{MessageEnvelope, MessageHandler, PubSub, PubSubError, SubscriptionId};
pub use queue::{Queue, QueueError, QueueMessage};
pub use secret::{Secret, SecretEntry, SecretError};
pub use state::{State, StateEntry, StateError};
pub use workflow::{Workflow, WorkflowError, WorkflowInstance, WorkflowStatus};
