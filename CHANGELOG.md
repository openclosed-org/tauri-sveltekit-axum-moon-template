# Changelog

All notable changes to this repository-level template are documented here.

This project ships as one template product:

- Repository tags and GitHub Releases are the public version contract.
- Internal Cargo crate versions remain workspace metadata for tooling.
- `release-plz` prepares release updates from merged conventional commits.

Preferred release views:

- Releases: <https://github.com/openclosed-org/axum-harness/releases>
- Tags: <https://github.com/openclosed-org/axum-harness/tags>

## Unreleased

### Changed

- Consolidated release-plz changelog output into the root `CHANGELOG.md` so the template has one public change history.
- Clarified that the root `axum-harness` package is an upstream maintainer release anchor, not a required derived-project runtime contract.

### Fixed

- Extended `template-init backend-core apply` so derived projects can remove the upstream release-plz workflow, runtime config, repo-release helper, and root release anchor together.

## v0.3.0 - 2026-04-27

### Added

- Added a `backend-core` template audit path so maintainers and adopters can prove the root command surface no longer depends on optional app-shell directories.
- Added `docs/architecture/harness-philosophy.md` to define harness boundaries, truth hierarchy, evidence levels, metadata limits, and gate strength.
- Added configurable release tag strategy inputs so maintainers can override tag template, tag glob, and bootstrap baseline without editing tracked files.

### Changed

- Decoupled optional web, desktop, mobile, and UI shell surfaces from the default backend-core template contract.
- Reworked root `just`, `moon`, and shared scripts so backend-core commands do not require SvelteKit, Tauri, `apps/**`, or `packages/ui/**` by default.
- Reframed `agent/codemap.yml` as a compact navigation map instead of a full system model or heavyweight architecture constitution.
- Rebuilt `agent/manifests/gate-matrix.yml` around changed paths, risk categories, and evidence levels instead of subagent identity.
- Clarified that `services/<name>/model.yaml`, `platform/model/**`, and agent YAML declarations are semantic summaries or metadata, not formal proof of system correctness.
- Clarified that `advisory`, `guardrail`, and `invariant` gates have different blocking strength, and that invariant gates are reserved for P0 correctness and release readiness.
- Clarified that `just verify-backend-primary` is the default backend-core guardrail and `just verify` is a broader repo-wide guardrail, not an automatic requirement to run every platform, frontend, desktop, production, or release gate.
- Strengthened the root agent protocol around bug fixes: reproduce or localize failures, identify violated invariants, make minimal causal repairs, add regression evidence, and never claim unrun gates as passed.

### Fixed

- Fixed release-plz workflow scoping so release PRs are prepared from the repository-level release anchor instead of unrelated workspace packages.
- Fixed release baseline comparison to use the active template tag line instead of stale hard-coded assumptions.
- Fixed release-plz worktree hygiene so generated release state does not leave the repository dirty during automation.
- Fixed backend-core root entrypoint drift by removing stale app-shell command exposure from shared validation and development lanes.

### Documentation

- Updated README, CONTRIBUTING, docs index, and agent docs to describe path/risk/evidence-based gate selection.
- Documented that metadata-only changes can raise a claim to `declared`, but `checked`, `tested`, and `proven` claims require executable evidence such as validators, tests, gates, or command output.

### Migration Notes

- Use `just verify-backend-primary` for ordinary backend-core development and add path-specific guardrails from `agent/manifests/gate-matrix.yml` when contracts, platform model, workers, topology, delivery, or release risk is involved.
- Use `bun run scripts/run-scoped-gates.ts --list` as compatibility guidance only; it no longer runs heavyweight gates just because a subagent handled a change.
- Treat app-shell validation as local to retained app shells. Root backend-core admission should remain independent from optional frontend and desktop surfaces.

## v0.2.0 - 2026-04-04

### Notes

- Current repository baseline tag for the template line.
- Tagged from commit `95ae1c9` (`chore: archive v0.2.0 milestone`).
