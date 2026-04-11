# Agent Prompt — Add Sync Strategy

## Steps

1. **Define SyncStrategy**: Add variant to `SyncStrategy` enum in relevant contract
2. **Declare on repo method**: All repo methods must include `strategy: SyncStrategy` parameter
3. **Implement ConflictResolver**: Use built-in (Lww/ClientWins/ServerWins) or define custom
4. **Add tests**: Cover offline_write, concurrent_edit, conflict_resolution
5. **Verify storage policy**: Run `just verify-storage-policy`

## Checklist

- [ ] SyncStrategy declared on all repository methods
- [ ] ConflictResolver implemented and tested
- [ ] Data classification matches storage policy (see constraints/storage-policy.yaml)
- [ ] Tests: `cargo test -p <service> -- sync` passes 100%
- [ ] Cloud cost monitoring: sync.pushed_bytes/pulled_bytes tracked
