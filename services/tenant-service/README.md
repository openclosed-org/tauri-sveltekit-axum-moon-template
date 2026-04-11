# services/tenant-service

> Tenant domain service — multi-tenant isolation strategy, member management, tenant lifecycle.

## Status
- [ ] Phase 0: Stub — business logic lives in `packages/core/usecases/`
- [ ] Phase 1: Implement domain/application/ports
- [ ] Phase 2: Independent deployment

## Dependencies
- `packages/core/kernel` (TenantId, AppError)
- `packages/core/domain` (port traits)
- `packages/contracts/*` (HTTP/Event contracts)
- `packages/features/auth` (AuthService trait for tenant auth)

## Architecture
- `domain/` — Tenant entity, membership rules, isolation policies
- `application/` — Use cases (create_tenant, add_member, remove_member)
- `ports/` — External dependency abstractions (TenantRepository)
- `contracts/` — Stable contract definitions (DTOs, events)
- `sync/` — OfflineFirst sync strategies
