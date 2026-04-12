//! Server-level adapters for service ports.
//!
//! These adapters implement the port traits defined in services by wrapping
//! concrete service implementations. This keeps services independent while
//! allowing the server to compose them together.

pub mod admin;
