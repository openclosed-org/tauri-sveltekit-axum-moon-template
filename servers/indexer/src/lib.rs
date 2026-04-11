//! Protocol event indexer.
//!
//! Pulls events from various sources, normalizes them to business DTOs,
//! and writes to Turso for read-optimized queries.

pub mod sinks;
pub mod sources;
pub mod transformers;
