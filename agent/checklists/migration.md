# Checklist — Database Migration

## Pre-Migration

- [ ] Migration file created in `services/<service>/migrations/`
- [ ] Migration is reversible (has `up` and `down`)
- [ ] Migration tested locally against Turso embedded
- [ ] No data loss scenario identified (or documented)

## Testing

- [ ] `cargo test -p <service>` passes with fresh migration
- [ ] Migration applied and rolled back successfully
- [ ] Data integrity verified after migration + rollback cycle
- [ ] Integration tests pass with testcontainers

## Deployment

- [ ] Migration runner updated to include new migration
- [ ] `ops/migrations/runner/` can execute migration
- [ ] Rollback plan documented
- [ ] Affected services notified

## Post-Migration

- [ ] Production migration executed
- [ ] Health checks pass
- [ ] No regression in sync behavior
- [ ] `just verify-storage-policy` passes
