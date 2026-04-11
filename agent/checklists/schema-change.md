# Checklist — Schema Change

## Pre-Change

- [ ] `docs/contracts/changelog.md` updated with change description
- [ ] `[BREAKING]` tag added if interface incompatible
- [ ] All dependent parties notified (frontend teams, other service owners)

## During Change

- [ ] Contract types updated in `packages/contracts/`
- [ ] `just gen-openapi` executed, `openapi.snapshot.json` updated
- [ ] `just gen-asyncapi` executed, `asyncapi.yaml` updated
- [ ] `just gen-frontend-sdk` executed, `packages/sdk/typescript` updated
- [ ] `git diff --exit-code` passes on all snapshots

## Post-Change

- [ ] Frontend compiles with new types (`bun --filter 'web' run build`)
- [ ] Contract tests pass
- [ ] `agent/checklists/schema-change.md` updated
- [ ] `agent/codemap.yml` updated if boundary changes
- [ ] CI green on all quality gates

## Rollback Plan

- [ ] Git revert to pre-change commit
- [ ] Re-run `just gen-openapi` + `just gen-frontend-sdk`
- [ ] Verify frontend compiles
- [ ] Notify all dependent parties of rollback
