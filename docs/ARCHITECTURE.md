# Architecture

This template follows a boundary-first architecture so teams can evolve features
without coupling UI, business rules, and runtime concerns too early.

## High-level Layers

`apps/desktop-ui`

- Desktop shell and frontend runtime.
- Hosts SvelteKit UI and Tauri integration.

`crates/domain`

- Pure domain model and business invariants.
- Should avoid runtime/framework dependencies where possible.

`crates/application`

- Use case orchestration.
- Coordinates domain operations and boundary contracts.

`crates/shared_contracts`

- Shared DTO/schema contracts across runtimes or transport layers.
- Keeps message shape changes explicit and versionable.

`crates/runtime_tauri`

- Tauri runtime integration boundary.
- Bridges desktop shell behavior and application use cases.

`crates/runtime_server`

- Server runtime boundary (Axum-oriented scaffold).
- Reserved for HTTP/service runtime concerns.

## Why This Structure

- Keeps core logic independent from delivery mechanism (desktop/server).
- Allows incremental backend/runtime adoption without rewiring everything.
- Supports small-team velocity by making module responsibilities explicit.

## Current State

The repository is intentionally scaffold-level:

- Layer boundaries exist.
- Crates and wiring points are prepared.
- Most business/runtime implementation is left for adopters.

## Suggested Growth Path

1. Define domain entities and invariants in `crates/domain`.
2. Add use case services in `crates/application`.
3. Define request/response contracts in `crates/shared_contracts`.
4. Expose use cases via Tauri commands in `apps/desktop-ui/src-tauri` and/or `crates/runtime_tauri`.
5. If needed, expose use cases via HTTP in `crates/runtime_server`.

## Quality Gates

The default CI enforces:

- Rust `check`, `fmt`, `clippy`, `test` (excluding Tauri crate for portability in CI).
- Frontend `check`, `lint`, `build`.

See `.github/workflows/ci.yml` for exact commands.
