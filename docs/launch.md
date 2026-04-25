# axum-harness: Experimental backend reference architecture for Axum

**axum-harness** is an experimental reference architecture for Rust backends built with Axum.
It contains a real backend reference chain, but it is **not** a fully production-hardened microservice platform.

## Current canonical path

The current default backend path is:

```text
counter-service
  -> web-bff
  -> CAS + idempotency + unified event_outbox
  -> outbox-relay worker
  -> projector worker
  -> replayable read model
```

This is the path new agents and developers should study first.

## What is already real

- `services/*` are library-first business boundaries
- `counter-service` is the default backend reference anchor
- `web-bff` is the current default sync entrypoint for the counter chain
- `outbox-relay` and `projector` are the current async reference workers
- `platform/model/*`, SOPS, Kustomize, and Flux already have partial real landing points

## What is deferred or only partially landed

The repository also contains target-state or partially landed material for:

- broader worker families
- `packages/runtime` abstractions
- future auth platform integration
- future gateway/platform capabilities
- future Wasm extension points

These are **not** the default starting point unless the task explicitly targets them.

## Reading order

For backend work, start with:

1. `README.md`
2. `AGENTS.md`
3. `docs/README.md`
4. `docs/operations/counter-service-reference-chain.md`
5. `docs/architecture/refactor-backlog-monolith-first-topology-late.md`
6. `docs/adr/009-canonical-monolith-first-topology-late-backend.md`

## Quick start

```bash
just setup
just setup-deps
bash infra/local/scripts/bootstrap.sh up
just dev-api
just verify
```

## License

Apache 2.0
