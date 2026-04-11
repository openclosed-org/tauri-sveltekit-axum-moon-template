//! BFF layer for {{target}} — aggregation and view model assembly.
//!
//! This crate is a **composition root only**:
//! - Register middleware (CORS, tracing, tenant validation)
//! - Wire routes to usecase implementations
//! - Initialize connections
//! - **No business logic** — all logic lives in service crates
