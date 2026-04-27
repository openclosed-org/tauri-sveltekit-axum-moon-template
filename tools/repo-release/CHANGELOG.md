# Changelog

All notable changes to this repository-level release anchor will be documented in this file.

## Unreleased

## v0.3.0 - 2026-04-27

### Added

- Added a dedicated harness philosophy document covering truth hierarchy, evidence levels, metadata limits, and gate strength.
- Added explicit backend-core audit and validation guidance for template adopters.

### Changed

- Bumped the repository release anchor to `0.3.0` for the next template release.
- Decoupled optional app-shell surfaces from the root backend-core contract.
- Reworked agent control-plane guidance so codemap remains a navigation map and gate selection is based on changed paths, risk, and evidence level.
- Clarified that YAML metadata can declare intent but cannot prove distributed semantics or P0 correctness.

### Fixed

- Fixed release-plz workflow scoping, baseline comparison, and worktree cleanliness for repository-level releases.
