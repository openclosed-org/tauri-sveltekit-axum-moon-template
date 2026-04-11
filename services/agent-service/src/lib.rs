//! Agent domain service — AI agent orchestration, tool calling, conversation management.
//!
//! ## Status
//! - [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
//! - [ ] Phase 1: Implement domain/application/ports
//! - [ ] Phase 2: Independent deployment
//!
//! ## Architecture
//! - `domain/` — Agent entity, tool definitions, conversation state
//! - `application/` — Use cases (chat_stream, tool_call, configure)
//! - `ports/` — External dependency abstractions (LLMProvider, ToolRegistry)
//! - `contracts/` — Stable contract definitions
//! - `sync/` — OfflineFirst sync strategies
//! - `infrastructure/` — LLM provider implementations
//! - `interfaces/` — API route handlers

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
