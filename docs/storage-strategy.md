# Three-Tier Storage Strategy

> **Status**: Draft — Phase 1 implementation partial

## Overview

The system uses three storage tiers, each with distinct sync strategies and conflict resolution policies.

## Tier 1: Local (Tauri Store)

- **Data**: Application config (theme, language, preferences)
- **Storage**: `~/.app/settings.json` via Tauri Store plugin
- **Sync**: None — pure local
- **Conflict**: N/A

## Tier 2: Embedded (Turso)

- **Data**: User private data + tenant business data
- **Storage**: `local.db` (libSQL embedded replica)
- **Sync**: OfflineFirst — local writes, background async sync to cloud
- **Conflict**: ClientWins (user private), LWW + business rules (tenant business)
- **Target**: embedded hit rate > 80%

## Tier 3: Cloud (Turso Cloud)

- **Data**: Shared/public data, cross-tenant operations
- **Storage**: Turso Cloud database
- **Sync**: OnlineOnly — direct cloud writes
- **Conflict**: ServerWins

## Sync Strategy Declaration

All repository methods MUST declare `SyncStrategy`:

```rust
pub async fn operation(&self, ctx: &TenantContext, strategy: SyncStrategy, ...) -> Result<T>;
```

## Cloud Cost Monitoring

- `sync.pushed_bytes` / `sync.pulled_bytes` tracked in OpenObserve
- `sync.conflict_rate` alert threshold: >1%
- `turso.embedded_hits` target: >80%
