//! Chat domain service — conversations, messages, real-time streaming.
//!
//! ## Status
//! - [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
//! - [ ] Phase 1: Implement domain/application/ports
//! - [ ] Phase 2: Independent deployment
//!
//! ## Architecture
//! - `domain/` — ChatMessage, ChatSession entities
//! - `application/` — Use cases (send_message, get_history, etc.)
//! - `ports/` — External dependency abstractions (MessageStore)
//! - `contracts/` — Stable contract definitions
//! - `sync/` — OfflineFirst sync strategies
//! - `infrastructure/` — WebSocket/SSE real-time implementations
//! - `interfaces/` — API route handlers

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod ports;
pub mod sync;
