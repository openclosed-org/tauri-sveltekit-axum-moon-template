# Sync Conflict Resolution Guide

> **Status**: Draft

## Built-in Conflict Resolvers

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `Lww` | Last-Write-Wins (timestamp) | General shared data |
| `ClientWins` | Local overwrites remote | User private data |
| `ServerWins` | Remote overwrites local | Shared/platform data |

## Conflict Detection

Conflicts are detected when two versions of the same record have diverged.
Each record carries a `version` field (monotonically increasing or timestamp-based).

## Resolution Flow

1. Pull remote records during sync cycle
2. Compare `version` with local records
3. If versions diverge → invoke registered `ConflictResolver`
4. Apply resolution, update local record
5. Push resolved state to cloud
6. Log conflict metrics to OpenObserve

## Custom Resolvers

Custom conflict resolvers require:
- ADR approval
- Complete test coverage (offline_write, concurrent_edit, conflict_resolution)
- Must not be implemented directly in business modules

## Monitoring

Conflict rate must stay < 1%. Alert fires if threshold exceeded.
