//! Tenant domain service — isolation strategy, member management, tenant lifecycle.
//!
//! ## Status
//! - [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
//! - [ ] Phase 1: Implement domain/application/ports
//! - [ ] Phase 2: Independent deployment
//!
//! ## Architecture
//! - `domain/` — Tenant entity, membership rules, isolation policies
//! - `application/` — Use cases (create_tenant, add_member, remove_member)
//! - `ports/` — External dependency abstractions (TenantRepository)
//! - `contracts/` — Stable contract definitions
//! - `sync/` — OfflineFirst sync strategies

pub mod application;
pub mod contracts;
pub mod domain;
pub mod ports;
pub mod sync;
