# Changelog

All notable changes to this template will be documented in this file.

The format is based on Keep a Changelog,
and this project follows Semantic Versioning for template tags/releases.

## [Unreleased]

### Added
- Community health files: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`.
- GitHub collaboration templates for issues and pull requests.
- Repository positioning doc: `ABOUT.md`.
- Documentation set in `docs/`:
  - `ARCHITECTURE.md`
  - `BOOTSTRAP_CHECKLIST.md`

### Changed
- CI now validates Rust workspace and frontend quality gates directly.
- Default branch references aligned to `main` while keeping compatibility with `master`.
- Ignore AI planning artifacts with `.planning/` in `.gitignore`.
- README redesigned with clearer scope, bootstrap steps, documentation index, and command map.
- README and ABOUT now provide bilingual (Chinese/English) project introduction content.

## [0.1.0] - 2026-03-22

### Added
- Initial public template skeleton for Tauri + SvelteKit + Rust workspace + moon.
