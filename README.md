# axum-harness

`axum-harness` is a backend-first Rust/Axum template and living reference architecture for building services with explicit contracts, service boundaries, transactional semantics, and verification gates.

The default path is intentionally lightweight. Runtime topology can grow from local development toward single-VPS and K3s-style deployments, but optional infrastructure such as NATS, Valkey, SOPS, Flux, and richer observability should not be required for the minimal backend path.

> Reality check: this repository is useful as a starting point, not proof that every pattern is production-proven. Treat code, tests, gates, and generated artifacts as stronger evidence than prose, YAML declarations, or discussions.

## Choose Your Path

| Goal                               | Start here                                           |
| ---------------------------------- | ---------------------------------------------------- |
| Use this as a template             | `docs/template-users/README.md`                      |
| Run locally                        | `docs/operations/local-dev.md`                       |
| Manage secrets                     | `docs/operations/secret-management.md`               |
| Contribute upstream                | `CONTRIBUTING.md`                                    |
| Understand the reference chain     | `docs/operations/counter-service-reference-chain.md` |
| Understand architecture principles | `docs/architecture/harness-philosophy.md`            |
| Browse all docs                    | `docs/README.md`                                     |

## What This Repo Is

- A reference chain for DDD-style services, explicit contracts, CAS/idempotency, transactional outbox events, projections, and replay.
- A lightweight default backend path that can grow toward richer deployment topologies when the operational cost is justified.
- An agent-aware repository with codemaps, routing rules, and path/risk-based verification guidance.
- A work-in-progress harness where claims should be checked against executable evidence.

## What This Repo Is Not

- Not a production-proven framework.
- Not a large demo application.
- Not a requirement to run Kubernetes, NATS, Valkey, SOPS, Flux, or full observability by default.
- Not proof that every declared platform model or topology behavior is implemented.
- Not a replacement for evaluating your own operational, security, latency, and compliance requirements.

## Current Reference Chain

`counter-service` is the current backend reference chain. It is intentionally small so the repository can exercise service engineering semantics without hiding behind product complexity.

```text
HTTP / BFF
  -> API contract
  -> application service
  -> domain model
  -> CAS + idempotency
  -> transactional outbox
  -> relay
  -> projection
  -> replay / rebuild
  -> verification gates
```

For the detailed state, current gaps, and expected extension pattern, see `docs/operations/counter-service-reference-chain.md`.

## First Commands

```bash
just --list
just setup
just doctor
just boundary-check
just dev-api
```

`just dev-api` starts the Web BFF on the default local port. After it starts, open `http://localhost:3010/scalar` for the API documentation UI.

Windows users should prefer WSL2 or Git Bash for the current local workflow. The Rust, Bun, Moon, and Just commands are close to cross-platform, but local infrastructure, SOPS, auth bootstrap, and some shell helpers still assume a POSIX-like environment.

For the full local workflow, including local infrastructure, optional auth, and SOPS-managed secrets, see `docs/operations/local-dev.md`.

Template adopters can preview upstream cleanup with:

```bash
just template-init PROFILE=backend-core MODE=dry-run
```

## Agent-Aware Development

This repository includes an agent collaboration protocol. Humans and agents should start with `AGENTS.md`, then use `agent/codemap.yml` and `agent/manifests/gate-matrix.yml` to select ownership boundaries and verification.

Do not treat model metadata, documentation, or discussions as proof of runtime behavior.

## Project Status

The backend reference chain is actively developed and locally gated, but the project is not production-proven. Some workers, app shells, and topology paths are incomplete, optional, or profile-driven.

Use `counter-service` as the current anchor before extending new service patterns.

## Versioning

Template releases are tracked by repository tags and GitHub Releases. Cargo crate versions are internal workspace metadata unless documented otherwise.

See `CHANGELOG.md` and [GitHub Releases](https://github.com/openclosed-org/axum-harness/releases).

## License

Apache 2.0. See `LICENSE`.
