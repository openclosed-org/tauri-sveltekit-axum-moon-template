//! Counter infrastructure layer — concrete repository implementations.
//!
//! This module bridges the abstract `CounterRepository` port to
//! concrete storage backends. Currently implements the libsql/Turso adapter.

pub mod libsql_adapter;

pub use libsql_adapter::*;
