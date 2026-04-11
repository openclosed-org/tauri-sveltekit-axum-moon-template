# Multi-Tenancy Model

> **Status**: Draft — Phase 1 implementation pending

## Tenant Isolation Strategy

All business data is scoped to a `TenantId`, extracted from JWT `sub` claim.
Services receive `TenantContext` in every repository method call — preventing tenant data leakage at compile time.

## Tenant Types

| Type | Description | Isolation |
|------|-------------|-----------|
| Platform | System-wide operations | No tenant scope |
| Tenant | Individual organization | Schema-scoped in Turso |
| User | Individual within a tenant | Row-level via user_tenant_membership |

## Tenant Context Flow

```
JWT (sub=TenantId) → Axum middleware → Extension<TenantId> → Service method → Repository (scoped query)
```

## Cross-Tenant Operations

Platform admin operations bypass tenant scoping but require explicit `PlatformAdmin` role.
Admin BFF enforces role checks; services receive explicit tenant context.

## Data Residency

- Turso embedded: per-tenant local DB file
- Turso cloud: shared database with tenant-scoped queries
- Sync: OfflineFirst with conflict resolution per tenant
