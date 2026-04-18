# Contributing

Thanks for your interest in contributing.

This repository is a backend-first Rust reference architecture with a built-in multi-agent harness. The fastest way to contribute successfully is to follow the existing boundaries instead of treating the repo like a generic monorepo.

## Before you start

- Read `README.md` for the project overview.
- Read `AGENTS.md` for the repository protocol.
- Read `agent/codemap.yml` for ownership boundaries and anti-patterns.
- Read `docs/operations/counter-service-reference-chain.md` before changing backend architecture patterns.

## Ground rules

- Keep changes small and reversible.
- Prefer evidence from code, validators, and gates over prose.
- Do not hand-edit generated directories such as `platform/catalog/**`, `docs/generated/**`, `infra/kubernetes/rendered/**`, or generated SDK output.
- Do not introduce product-specific business logic under the name of a generic pattern improvement.
- Secrets must never be committed in plaintext. Use SOPS-based flows, not `.env` files, for backend secrets.

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

Common commands:

```bash
just fmt
just lint
just typecheck
just test
just validate-platform
just validate-state strict
just validate-workflows strict
just verify-replay strict
just verify-counter-delivery strict
just verify
```

Agent- and domain-specific validation is also available:

```bash
just gate-scoped AGENT=service-agent
bun run scripts/run-scoped-gates.ts service-agent
```

Only claim checks passed if you actually ran them.

## Pull requests

- Explain what changed and why.
- Include exact verification commands you ran.
- Keep PRs focused.
- Update docs when behavior, workflows, or operator expectations change.
- Avoid unrelated cleanup in the same PR.

## Reporting bugs and features

- Use the GitHub issue templates.
- For security issues, use the private contact in `.github/ISSUE_TEMPLATE/config.yml` instead of opening a public issue.

## Code of conduct

By participating in this project, you agree to follow `CODE_OF_CONDUCT.md`.

## License

This project is licensed under the Apache License, Version 2.0. See `LICENSE`.
