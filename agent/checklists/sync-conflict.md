# Checklist — Sync Conflict Resolution

## Conflict Detection

- [ ] Conflict scenario identified (concurrent writes to same record)
- [ ] Both local and remote versions captured
- [ ] Timestamp/version metadata available

## Resolution Strategy

- [ ] Correct BuiltInResolver selected:
  - [ ] `Lww` — Last-Write-Wins (general shared data)
  - [ ] `ClientWins` — Local wins (user private data)
  - [ ] `ServerWins` — Remote wins (shared/platform data)
- [ ] Custom strategy only if ADR approved
- [ ] Custom strategy has complete test coverage

## Testing

- [ ] `cargo test -p <service> -- sync` passes
- [ ] Conflict rate < 1% in production (OpenObserve dashboard)
- [ ] `turso.embedded_hits` > 80% (OpenObserve dashboard)
- [ ] No data loss in conflict scenarios

## Monitoring

- [ ] `sync.pushed_bytes` tracked
- [ ] `sync.pulled_bytes` tracked
- [ ] `sync.conflict_rate` alert configured (threshold: >1%)
- [ ] Dashboard updated with sync metrics
