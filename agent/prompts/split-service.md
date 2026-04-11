# Agent Prompt — Split Service

## Steps

1. **Verify independence**: `cargo build -p <service> --workspace` must succeed
2. **Run preview**: `./ops/scripts/preview-split.sh <service>` must succeed
3. **Extract contracts**: Ensure all shared types in `packages/contracts/`
4. **Extract events**: Move cross-cutting events to `packages/contracts/events/`
5. **Create new repo**: Copy service directory + Cargo.toml
6. **Update workspace**: Remove from parent Cargo.toml, add as external dep
7. **Update CI**: Add independent build pipeline
8. **Verify**:
   - Independent `cargo build` succeeds
   - Independent `cargo test` succeeds
   - Dependent services compile against new external crate

## Checklist

- [ ] No circular dependencies
- [ ] All contracts externalized
- [ ] Migrations self-contained
- [ ] Config loaded externally (not hardcoded)
- [ ] CI pipeline updated
- [ ] Documentation updated
