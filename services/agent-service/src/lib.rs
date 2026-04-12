//! Agent domain service — AI agent orchestration, tool calling, conversation management.
//!
//! ## Architecture
//! ```text
//! domain/          → Domain error types
//! ports/           → LlmProvider, ToolExecutor traits
//! application/     → Migration constants
//! infrastructure/  → LibSqlAgentRepository (LibSQL + HTTP LLM streaming)
//! contracts/       → DTO re-exports from packages/contracts/
//! interfaces/      → Factory functions (for BFF composition)
//! sync/            → OfflineFirst sync strategies
//! ```

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
