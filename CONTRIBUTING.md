# Contributing

Thanks for contributing to this template.

## Development Setup

1. Install Rust `1.82.0`, Node.js `22.11.0`, and Bun `1.3.11`.
2. Install frontend dependencies:

```bash
cd apps/desktop-ui
bun install
```

3. Run checks from repository root:

```bash
cargo check
```

4. Run frontend quality gates:

```bash
cd apps/desktop-ui
bun run check
bun run lint
bun run build
```

## Branch and Pull Request Rules

- Keep each pull request focused on one topic.
- Add or update tests/checks when behavior changes.
- Keep CI green before requesting review.
- Use clear commit messages that explain intent.

## Commit Message Style

Use concise messages in imperative mood, for example:

- `feat: add tauri command scaffold`
- `fix: align ci branch filters with default branch`
- `docs: clarify template bootstrap steps`

## What Not To Change In Template PRs

- Do not add product-specific business logic.
- Do not add secrets or local machine paths.
- Avoid introducing new dependencies unless required.

## Reporting Problems

- Use GitHub Issues for bugs and feature requests.
- For security-sensitive issues, follow `SECURITY.md`.
