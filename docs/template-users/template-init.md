# Template Init Design

> Status: partially scaffolded.
> `just template-init` currently exists as a conservative planning/dry-run entrypoint.

## Goal

Turn this upstream repository from:

1. template product
2. contributor workspace
3. architecture research repo

into a derived project with less upstream-maintainer baggage.

## Non-goals

`template-init` should not:

1. silently delete files without preview
2. guess business requirements for the user
3. rewrite the main backend code layout aggressively
4. claim a profile is production-ready

## Modes

Initial command shape:

```bash
just template-init PROFILE=backend-core MODE=dry-run
```

Supported parameters:

1. `PROFILE`
2. `MODE`

### MODE

1. `dry-run` — print what would be removed or kept; default and safest mode
2. `apply` — later phase; will require explicit opt-in once the rules are stable

### PROFILE

1. `backend-core` — keep the default Rust backend reference chain and review upstream-only materials
2. `backend-desktop` — future profile; keep backend plus desktop-related app paths
3. `full-research` — keep everything; useful as a no-op baseline for contributors

Current implementation only documents and previews the `backend-core` profile.

## backend-core profile

### Keep

1. `README.md`
2. `LICENSE`
3. `AGENTS.md`
4. `agent/**`
5. `CODE_OF_CONDUCT.md`
6. `docs/operations/**`
7. `docs/contracts/README.md`
8. `services/counter-service/**`
9. `servers/bff/web-bff/**`
10. `workers/outbox-relay/**`
11. `workers/projector/**`
12. shared packages required by the default backend path
13. `infra/**` required by local dev and secrets flow
14. stable setup / dev / test / quality commands

### Candidate removals or review

1. `docs/governance/**`
2. `docs/archive/**`
3. `release-plz.toml`
4. `.github/ISSUE_TEMPLATE/**`
5. `.github/pull_request_template.md`

### Review manually

1. `CONTRIBUTING.md`
2. `CODE_OF_CONDUCT.md`
3. `docs/template-users/**`
4. `.github/workflows/**`
5. optional apps or extra modules your derived project may or may not keep

## Safety rules

Before `apply` mode exists, the command should only emit a clear plan.

When `apply` mode is added later, it should:

1. require explicit `MODE=apply`
2. print the selected profile
3. print the exact paths to remove
4. refuse to run if the worktree is dirty unless the user explicitly acknowledges the risk
5. stay focused on upstream-maintainer/open-source cleanup instead of broad code deletion

## Why profiles exist

Profiles matter because this repository serves two very different groups:

1. people adopting a backend template quickly
2. people maintaining the upstream template as an open-source project

Those users should not be forced to carry the same repository surface forever.
