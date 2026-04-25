# Docs

`docs/` is a small, shared layer for durable guidance. If something is only useful for one active task, it should not stay here.

When docs conflict with code, validators, gates, or executable scripts, trust the executable sources.

## Start Here

For general readers and template adopters:

1. `README.md`
2. `docs/operations/local-dev.md`
3. `docs/operations/secret-management.md`
4. `docs/operations/counter-service-reference-chain.md`
5. `docs/template-users/template-init.md`

For maintainers and agent-assisted development:

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `docs/adr/**`
4. `docs/governance/docs-lifecycle.md`

## What Belongs Here

Tracked `docs/**` should stay limited to:

1. current operator/developer guidance in `docs/operations/**`
2. durable architecture decisions in `docs/adr/**`
3. minimal template-adoption guidance in `docs/template-users/**`
4. historical notes in `docs/archive/**`
5. a small amount of governance guidance when it remains stable and worth keeping

## What Does Not Belong Here

Put these in `docs/_local/` so they stay outside the shared docs entry flow and out of tracked repository docs:

1. one-off refactor backlogs
2. temporary execution plans
3. personal scratch notes
4. exploratory comparisons that have not become durable decisions

`docs/_local/` is the maintainer workspace for active plans, execution notes, and temporary guidance.
It should stay gitignored; if you need lightweight versioning, keep that inside the document itself with frontmatter or explicit revision notes.

## Default Backend Anchor

`counter-service` remains the default backend reference anchor for this repository:

1. business chain: `service -> contracts -> server -> outbox -> relay -> projector`
2. engineering chain: `platform model -> secrets -> deploy -> GitOps -> runbook`

If you are trying to understand the current backend path, start there instead of reading speculative planning material.

## Repository Notes

Repository-level policy and project-level caveats live here instead of expanding the root `README.md`.

### Versioning

This repository is versioned as a single template product.

1. the repository tag/release is the version contract template users should follow
2. pre-`1.0.0` releases cover iterative template improvements, docs fixes, tooling updates, and internal refactors within the active pre-1.0 line
3. bump the minor version when template structure, migration expectations, or public usage patterns change materially
4. start `1.0.0` only when the template is ready to make a stronger stability promise to adopters

Cargo crate versions still exist as workspace metadata, but they do not represent independent product release channels.

If you derive a new repository from this template, bootstrap the first official repository release manually with your chosen starting tag, then let automation continue from there.

`release-plz` prepares repository releases, and `just semver-check` is the local visibility check for repository-level compatibility assumptions within the active pre-1.0 line.

### Configuration

There are three different configuration paths and they should not be confused:

1. canonical backend path: `SOPS -> Kustomize/Flux`, with `just sops-run` locally
2. quick backend debug path: explicit `APP_*` exports for short host-process loops
3. local tooling or desktop convenience path: `.env`, which is not the canonical backend secrets path

If you are working on backend deployables, prefer the first path by default.

### Desktop Scope

Desktop/Tauri is a local convenience shell in this repository, not part of the default backend CI contract.

1. default GitHub CI covers backend, platform, contracts, and web-facing verification lanes
2. desktop and Tauri validation stay opt-in and local because cross-platform packaging and debugging requirements are materially different across macOS, Linux, and Windows
3. if you change `apps/desktop/**`, run the desktop checks explicitly instead of assuming the default CI lanes cover them

### Architecture Rules

The root `README.md` keeps the public overview short. The main repository rules live here and in `AGENTS.md`:

1. `platform/model/*` is the truth source for platform shape
2. contracts change before implementation when external shapes change
3. services are libraries; servers and workers compose them
4. workers are first-class and must make retry, idempotency, and replay explicit
5. generated directories are read-only
6. topology may change deployment shape, but not business semantics
