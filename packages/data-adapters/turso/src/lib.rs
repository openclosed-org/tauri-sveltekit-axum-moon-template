//! Turso storage adapter — implementations of LibSqlPort for embedded and remote backends.
//!
//! Uses the `turso` crate (Turso's official Rust SDK) instead of `libsql`.
//! This adapter is Windows-friendly: no `libsql-ffi` C compilation required.

pub mod backend;
pub mod embedded;
pub mod remote;

pub use backend::TursoBackend;
pub use embedded::EmbeddedTurso;
pub use remote::TursoCloud;
