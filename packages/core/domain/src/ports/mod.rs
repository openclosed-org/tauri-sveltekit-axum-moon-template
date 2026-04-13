//! Port types and cross-crate utilities.
//!
//! `TenantId` is re-exported from `kernel` to ensure a single canonical type
//! across the workspace.  Code that previously used `domain::ports::TenantId`
//! will continue to compile unchanged.

pub mod lib_sql;
pub mod surreal_db;

// Re-export so that `domain::ports::TenantId` remains a valid import path
pub use kernel::TenantId;
