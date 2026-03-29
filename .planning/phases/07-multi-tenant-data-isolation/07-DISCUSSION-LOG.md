# Phase 7: Multi-Tenant Data Isolation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 07-multi-tenant-data-isolation
**Areas discussed:** Database scope, Tenant ID propagation, User-tenant binding, Schema strategy, Cross-tenant isolation behavior

---

## Database Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Both databases | Tenant isolation on both SurrealDB and libsql | |
| SurrealDB only | Server-side isolation only, libsql stays per-device | ✓ |
| libsql only | Local-first, client-side isolation | |

**User's choice:** SurrealDB only (Recommended)
**Notes:** libsql is embedded-per-device (single user), no cross-tenant leak risk.

---

## Tenant ID Propagation

| Option | Description | Selected |
|--------|-------------|----------|
| JWT decode middleware | Reuse jsonwebtoken crate to decode id_token from Authorization header | ✓ |
| openidconnect + tower-sessions | Full OIDC client + server-side sessions | |
| Rauthy sidecar | Full-featured OIDC provider as sidecar service | |

**User's choice:** Initially asked about Rauthy vs openidconnect+webauthn-rs stack. After analysis of boilerplate fit, chose JWT decode middleware (Recommended).
**Notes:** Zero new deps, reuses existing jsonwebtoken crate. Rauthy deferred to v2 for projects that need multi-provider auth.

---

## User-Tenant Binding

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-create on first login | Simplest, 1 user = 1 tenant | |
| Admin invite flow | Requires admin roles, invite management | |
| Hybrid (auto-create + invite-ready) | Auto-create by default, schema supports invite later | ✓ |

**User's choice:** Hybrid (Recommended)
**Clarification:** Schema-ready with stub invite (tables exist, auto-create works, invite endpoints stubbed but not wired).

---

## Schema Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Query-level scoping | WHERE tenant_id = $tenant_id in application layer | |
| Namespace-per-tenant | SurrealDB namespace isolation per tenant | |
| Port trait wrapper | Wrapper trait that auto-injects tenant_id | ✓ |

**User's choice:** Port trait wrapper
**Clarification:** Optional tenant field on trait impl — `tenant_id: Option<String>`. Some = scoped, None = unscoped (admin/migrations).

---

## Cross-Tenant Isolation Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Silent filtering | Auto-inject WHERE clause, mismatched returns empty | ✓ |
| Explicit validation errors | Check before every op, return Err on mismatch | |

**User's choice:** Silent filtering (Recommended)
**Notes:** Matches success criteria "empty results, not errors".

---

## Agent's Discretion

- JWT 解析失败处理方式
- Tenant name 默认值策略
- SurrealDB tenant_id 索引策略
- SurrealQL scope clause 注入格式

## Deferred Ideas

- libsql tenant isolation (v2)
- Full invite flow implementation (v2)
- RBAC (v2)
- Cloud session management (v2)
- Server-side token validation (v2)
