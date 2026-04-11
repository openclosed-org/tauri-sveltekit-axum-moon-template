# services/{{domain}}

> {{domain}} domain service — {{description}}

## Status
- [ ] Phase 0: Stub
- [ ] Phase 1: Implement domain/application/ports
- [ ] Phase 2: Independent deployment

## Dependencies
- `packages/core/kernel` (TenantId, AppError)
- `packages/core/domain` (port traits)
- `packages/contracts/*` (HTTP/Event contracts)
- `packages/features/{{domain}}` (trait definition)

## Architecture
- `domain/` — Pure domain logic
- `application/` — Use case orchestration
- `ports/` — External dependency abstractions
- `contracts/` — Stable contract definitions
- `sync/` — OfflineFirst sync strategies

## Migration
```bash
just migrate up -p {{domain}}
```
