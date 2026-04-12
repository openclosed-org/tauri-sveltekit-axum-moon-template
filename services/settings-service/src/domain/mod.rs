//! Settings domain entities.
//!
//! Key design difference from counter:
//! - Counter is **tenant-scoped** (keyed by tenant_id)
//! - Settings is **user-scoped** (keyed by user_sub)
//!
//! This means settings survive tenant switching and represent
//! personal user preferences, not organizational configuration.

pub mod entity;
pub mod error;

pub use entity::{AgentConnectionSettings, UserSettings};
pub use error::SettingsDomainError;
