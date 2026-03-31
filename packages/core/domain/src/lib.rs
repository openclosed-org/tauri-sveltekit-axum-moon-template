//! domain crate — Port trait definitions for database backends.
//!
//! Per D-05/D-06: trait-per-DB pattern. Each database backend gets its own Port trait.
//! Implementations live in runtime_server (SurrealDB) and native-tauri (libsql) crates.

pub mod ports;
