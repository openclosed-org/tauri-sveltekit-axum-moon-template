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

- Aligned repository release automation and SemVer checks with the latest real template baseline instead of the old hard-coded `v0.1.x` line.
- Clarified in README and maintainer docs that repository tags are the template version contract and that desktop/Tauri validation is local-only, not part of the default backend CI admission flow.
- Reworked Quick Start and local development guidance to emphasize the backend-first path and remove stale or misleading command examples.
- Fixed multiple broken or outdated `just` recipes in backend/platform workflows, including SemVer baseline detection, platform inventory commands, migration helpers, process utilities, and stale package/path references.
- Added a repository-level changelog entry point so release notes have an explicit home in the template itself.

### Notes

- The active baseline tag in this repository remains `v0.2.0`.
- GitHub Releases remains the best public view for generated release notes once release automation runs.

## v0.2.0 - 2026-04-04

### Notes

- Current repository baseline tag for the template line.
- Tagged from commit `95ae1c9` (`chore: archive v0.2.0 milestone`).
