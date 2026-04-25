//! Repository-level release anchor crate.
//!
//! This crate exists only so release-plz can manage the repository as a single
//! template product without traversing unpublished workspace-internal crates.

/// Current repository release line anchor.
pub const REPOSITORY_RELEASE_ANCHOR: &str = "axum-harness";
