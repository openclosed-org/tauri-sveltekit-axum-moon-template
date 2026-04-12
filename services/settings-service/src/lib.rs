//! Settings domain service — user-level preferences (api_key, base_url, model).
//!
//! ## Architecture
//! ```text
//! domain/          → UserSettings, AgentConnectionSettings (user-scoped, NOT tenant-scoped)
//! ports/           → SettingsRepository trait
//! application/     → ApplicationSettingsService (implements feature-settings::SettingsService)
//! infrastructure/  → LibSqlSettingsRepository
//! contracts/       → DTO re-exports from packages/contracts/
//! interfaces/      → Factory functions
//! sync/            → OfflineFirst sync strategies
//! ```
//!
//! ## Key Design Difference
//! - **counter-service**: tenant-scoped (keyed by tenant_id)
//! - **settings-service**: user-scoped (keyed by user_sub)
//!
//! Settings represent personal user preferences that persist across tenant switching.

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
