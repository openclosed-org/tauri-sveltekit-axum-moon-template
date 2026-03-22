# tauri-sveltekit-axum-moon-template

A desktop-first monorepo template built with Rust workspace, Tauri v2, SvelteKit 5, and moon.

## Stack

- Rust workspace (Cargo)
- Tauri v2 (desktop shell)
- SvelteKit 5 + Vite (frontend)
- moonrepo (task orchestration)
- Bun (JavaScript runtime + package manager)

## Project Structure

```text
.
|- apps/
|  |- desktop-ui/
|     |- src/              # SvelteKit UI
|     |- src-tauri/        # Tauri app
|- crates/                 # Rust domain/application/adapters/runtime skeleton crates
|- .moon/                  # moon workspace/toolchain config
|- .github/workflows/      # CI
```

## Prerequisites

- Rust toolchain `1.82.0`
- Node.js `22.11.0`
- Bun

## Create From Template

1. Click `Use this template` on GitHub.
2. Create a new repository from this template.
3. Update `apps/desktop-ui/src-tauri/tauri.conf.json` values:
   - `productName`
   - `identifier`
   - window `title`
4. Update UI text in `apps/desktop-ui/src/routes/+page.svelte`.

## Quick Start

```bash
# install frontend deps
bun install --cwd apps/desktop-ui

# Rust workspace check (exclude Tauri crate for CI portability)
cargo check --workspace --exclude desktop-ui-tauri
```

## Common Commands

From repository root:

```bash
# Rust
cargo check --workspace --exclude desktop-ui-tauri
cargo test --workspace --exclude desktop-ui-tauri
cargo clippy --workspace --exclude desktop-ui-tauri -- -D warnings
cargo fmt --all -- --check
```

From repository root (frontend):

```bash
bun run --cwd apps/desktop-ui dev
bun run --cwd apps/desktop-ui check
bun run --cwd apps/desktop-ui lint
bun run --cwd apps/desktop-ui build
```

## CI

GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

- Rust check, test, clippy, fmt check
- Frontend check, lint, build

## Community Health

- Contribution guide: `CONTRIBUTING.md`
- Code of conduct: `CODE_OF_CONDUCT.md`
- Security policy: `SECURITY.md`
- Change history: `CHANGELOG.md`

## Security Note

`apps/desktop-ui/src-tauri/tauri.conf.json` currently uses `"csp": null` for a frictionless template start.
For production apps, define a strict CSP before release.

## License

This repository uses the WTF-0 Public License. See `LICENSE`.
