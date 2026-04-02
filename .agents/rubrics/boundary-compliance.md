# Boundary Compliance Rubric

**Purpose:** Agent code review checklist for hexagonal architecture boundary compliance.
**Enforced by:** CI (repo:boundary-check) + agent review (this rubric).

## Layer Rules

### domain (packages/core/domain/)
- MUST NOT import from: adapters/*, hosts/*, contracts_*, servers/*
- MUST NOT reference: tauri, axum, surrealdb, libsql, reqwest
- CAN import: async-trait, serde, serde_json, std
- Contains: Port traits (SurrealDbPort, LibSqlPort), value objects (TenantId), error types

### usecases (packages/core/usecases/)
- MUST NOT import from: adapters/*, hosts/*, contracts_*, servers/*
- MUST NOT reference: tauri, axum
- CAN import: domain, async-trait, serde, serde_json, chrono, thiserror
- Contains: Service traits, service implementations, internal DTOs
- Rule: usecases defines its OWN input/output types (e.g., CreateTenantInput), NOT contracts_api types

### contracts_* (packages/contracts/api, auth, events)
- MUST NOT import from: domain, usecases, adapters/*, hosts/*, servers/*
- CAN import: serde, ts-rs, utoipa, validator
- Contains: Cross-boundary DTOs only, no business logic

### adapters/storage/* (packages/adapters/storage/surrealdb, libsql)
- CAN import: domain (for port traits)
- MUST NOT import from: usecases, contracts_*, servers/*
- MUST NOT contain: business rules, query logic beyond port trait implementation
- Contains: Port trait implementations, migration scripts

### adapters/hosts/* (packages/adapters/hosts/tauri)
- CAN import: domain, usecases
- MUST NOT import from: contracts_* (unless feature crates bridge)
- Contains: Tauri command handlers that bridge to usecases
- Rule: command handlers delegate to usecases, no inline business logic

### servers/* (servers/api)
- CAN import: domain, usecases, contracts_api, adapters/*
- Contains: HTTP route handlers, Axum state, middleware
- Rule: route handler maps contracts_api DTO ↔ usecases types

## Review Checklist

When reviewing code changes, verify:
- [ ] No new cross-layer imports violating rules above
- [ ] New adapter implementations reference domain port traits, not concrete types
- [ ] usecases code does NOT import from contracts_api
- [ ] Command handlers in runtime_tauri delegate to usecases
- [ ] No business logic in servers/api route handlers (only mapping + delegation)
