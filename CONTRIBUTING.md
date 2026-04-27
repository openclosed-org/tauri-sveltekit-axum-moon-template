# Contributing

Thanks for your interest in contributing.

This repository is a backend-first Rust reference architecture with a built-in multi-agent harness. The fastest way to contribute successfully is to follow the existing boundaries instead of treating the repo like a generic monorepo.

## Before you start

- Read `README.md` for the project overview.
- Read `AGENTS.md` for the repository protocol.
- Read `agent/codemap.yml` for ownership boundaries and anti-patterns.
- Read `docs/operations/counter-service-reference-chain.md` before changing backend architecture patterns.

## Who this guide is for

This file is for people contributing to the template itself.

If you are using the repository via GitHub "Use this template", the repository release and README are your primary contract. You do not need to preserve every contributor-facing doc or research artifact in your derived project.

## Ground rules

- Keep changes small and reversible.
- Prefer evidence from code, validators, and gates over prose.
- Do not hand-edit generated directories such as `platform/catalog/**`, `docs/generated/**`, `infra/kubernetes/rendered/**`, or generated SDK output.
- Do not introduce product-specific business logic under the name of a generic pattern improvement.
- Secrets must never be committed in plaintext. Use SOPS-based flows, not `.env` files, for backend secrets.
- Put temporary private task notes, local refactor checklists, and scratch guidance under `docs/_local/`; it is gitignored by design.
- Only commit docs that are meant to remain useful to future contributors or template users.
- Follow `docs/governance/docs-lifecycle.md` when deciding whether a document should stay tracked, move to archive, or remain local-only.

## Development setup

```bash
just setup
just setup-deps
just doctor
bash infra/local/scripts/bootstrap.sh up
```

For local backend runs that need secrets, use SOPS-based injection:

```bash
just sops-run DEPLOYABLE=web-bff ENV=dev
```

## Change order

Use the existing repository order of operations:

1. If the platform schema cannot express the change, update `platform/schema/**` first.
2. Update `platform/model/**` for platform-level metadata and topology.
3. Update `services/<name>/model.yaml` for service-local semantics.
4. Update `packages/contracts/**` before implementation when API/event shapes change.
5. Then update `services/**`, `servers/**`, `workers/**`, and related verification.

## Where changes belong

- `platform/**` - platform model, validators, generators
- `packages/contracts/**` - contracts and source-of-truth protocol types
- `services/**` - domain logic and service-local semantics
- `servers/**` - sync protocol adaptation and entrypoints
- `workers/**` - async execution, replay, checkpoint, recovery
- `apps/**` - optional frontend shells, not the default backend entry

If a change crosses multiple domains, preserve the existing boundaries rather than collapsing everything into one patch.

## Validation

Run the smallest relevant validation set for your change, then run the global gate before merging.

Desktop/Tauri is not part of the default CI contract for this repository. If your change touches `apps/desktop/**` or desktop-specific runtime behavior, run the desktop commands explicitly on the target OS instead of assuming the backend CI lanes cover it.

Common commands:

```bash
just verify-backend-primary
just test-backend-primary
just boundary-check
just contracts-check strict
just verify
```

Heavier platform, replay, delivery, and release gates are selected by changed paths, risk, and evidence level. Do not run or require them for every ordinary backend-core change.

Gate-selection guidance is available here:

```bash
bun run scripts/run-scoped-gates.ts --list
```

Only claim checks passed if you actually ran them.

## Pull requests

- Explain what changed and why.
- Include exact verification commands you ran.
- Keep PRs focused.
- Update docs when behavior, workflows, or operator expectations change.
- Avoid unrelated cleanup in the same PR.

## Commit and release semantics

- This repository is versioned as one template product using repository-level SemVer.
- Follow the latest repository tag/release as the active template line until there is a deliberate reason to open a new pre-1.0 line or to declare `1.0.0` stability.
- Prefer conventional commits such as `feat:`, `fix:`, `docs:`, `refactor:`, `ci:`, and `chore:`.
- Use scopes when they add signal, for example `feat(template):`, `fix(bff):`, `docs(readme):`, `refactor(worker-runtime):`.
- If a change alters template defaults, project layout, setup flow, or migration expectations for template users, call that out explicitly in the PR description and release notes.
- If a change is only contributor-facing or internal, keep the commit message clear so `release-plz` changelogs stay readable.

## Release process

- This repository uses `release-plz` for repository release preparation.
- Maintainers should prefer conventional commits such as `feat:`, `fix:`, and `docs:` so generated changelogs stay readable.
- If you derive a new repository from this template, bootstrap the first official repository release manually with your chosen starting tag; after that, let release automation continue from that baseline.
- `release-plz` opens a release PR on `main` and prepares version/changelog updates from merged commits.
- Most binaries/internal crates are configured with `publish = false`, so release automation can still generate changelogs and GitHub releases without requiring every crate to publish to crates.io.
- The SemVer contract users should rely on first is the repository tag/release, because this repo is primarily shipped as a template rather than as a set of independently consumed crates.
- Cargo crate versions are still required for workspace tooling, but they are not independent product version promises.
- Use `just semver-check` locally when validating repository-level compatibility assumptions for the current template line.
- Default release automation uses `v{{ version }}`, but maintainers can override the tag template, detection glob, and bootstrap baseline through repository variables or `workflow_dispatch` inputs.

## Reporting bugs and features

- Use the GitHub issue templates.
- For security issues, use the private contact in `.github/ISSUE_TEMPLATE/config.yml` instead of opening a public issue.

## Code of conduct

By participating in this project, you agree to follow `CODE_OF_CONDUCT.md`.

## License

This project is licensed under the Apache License, Version 2.0. See `LICENSE`.
