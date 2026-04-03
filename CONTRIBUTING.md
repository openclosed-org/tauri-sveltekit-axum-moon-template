# Contributing

Thanks for contributing to this template.

## Development Setup

1. Install **mise** (toolchain manager):
   - macOS: `brew install mise`
   - Linux: `curl https://mise.run | sh`

2. Install all project tools (Rust, Node.js, Bun):

```bash
mise install
```

3. Install frontend dependencies:

```bash
bun install
```

4. Run checks from repository root:

```bash
cargo check
```

5. Run frontend quality gates:

```bash
cd apps/client/web/app
bun run check
bun run lint
bun run build
```

> All commands can also be run via `just` — the unified entry point for humans and agents.
> Run `just` to see all available commands.

## Testing Workflow

For detailed testing and coverage guidance, see [`docs/TESTING_AND_COVERAGE.md`](docs/TESTING_AND_COVERAGE.md).

**Quick summary:**

- **Daily development**: `just test` (fast, no coverage)
- **Before PR / After refactoring**: `just test-coverage-html` (check completeness)
- **CI/CD**: Automatic coverage upload to Codecov

Install coverage tools (one-time):
```bash
just setup-coverage
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
