# Checklist — Release

## Pre-Release

- [ ] All tests pass: `just test-all-rust` + `just test-all-frontend`
- [ ] Quality gate passes: `just verify`
- [ ] Contract check passes: `just ci-check-contracts`
- [ ] Boundary check passes: `just quality boundary`
- [ ] No lint warnings: `cargo clippy -- -D warnings`
- [ ] No format issues: `cargo fmt --all -- --check`

## Release Verification

- [ ] `mise doctor` passes
- [ ] `just --list` shows all commands
- [ ] `cargo hack check --workspace --feature-powerset` passes
- [ ] `cargo build -p <each-service>` succeeds independently
- [ ] `just deploy compose` starts dev environment
- [ ] E2E tests pass: `just test-e2e-full`

## Documentation

- [ ] CHANGELOG.md updated
- [ ] ADRs created for any architectural changes
- [ ] Contract changelog updated if API changed
- [ ] agent/codemap.yml updated if boundaries changed

## Post-Release

- [ ] Release tag created
- [ ] Deployment successful
- [ ] Health checks pass in production
- [ ] Monitoring dashboards reviewed
- [ ] Rollback plan documented and tested
