# Testing & Coverage Guide

This guide explains the testing and coverage workflow for this project.

## Toolchain Setup

All tools (Rust, Node.js, Bun) are managed by **mise**. Install once:

```bash
# Install mise if you haven't
# macOS: brew install mise
# Linux: curl https://mise.run | sh

# Install all project tools from .tool-versions
mise install
```

### Coverage Tools (one-time install)

```bash
just setup-coverage
```

This installs `cargo-llvm-cov`, `cargo-sweep`, and `llvm-tools` — all reused from your local system, no duplicate downloads.

## Quick Start

### Daily Development (Fast)

For day-to-day development, use fast tests without coverage overhead:

```bash
# Run unit tests only
just test
# or
moon run repo:test-unit

# Run nextest (faster parallel test runner)
just test-nextest
```

### Coverage Testing (Before PR / After Refactoring)

Generate coverage reports when:
- Preparing to submit a PR
- After major refactoring
- When you want to check test completeness

```bash
# Generate LCOV report (for CI/Codecov)
just test-coverage

# Generate and open HTML report in browser (local review)
just test-coverage-html

# Clean up coverage artifacts
just test-coverage-clean
```

## Tools Setup

### mise (unified toolchain)

All language tools are managed by mise via `.tool-versions`:

```bash
mise install          # install all tools
mise current rust     # check installed rust version
mise current node     # check installed node version
mise current bun      # check installed bun version
```

**Key advantage**: One tool manages everything. No proto, no separate rustup/node version managers needed.

### cargo-llvm-cov (coverage)

The primary coverage tool. If not installed:

```bash
just setup-coverage
```

**Key features:**
- Automatic cleanup of old `.profraw` files
- HTML reports for browser viewing
- LCOV output for CI integration
- `cargo llvm-cov clean` to remove all coverage artifacts

### cargo-sweep (smart cache cleanup)

Smart cache cleanup without nuking your entire build:

```bash
cargo install cargo-sweep
```

**Usage:**

```bash
# Clean build cache older than 7 days
just clean-sweep

# Clean unused dependency caches
just clean-sweep-deps

# Via moon
moon run repo:clean-sweep
```

**Pro tip**: Add to your crontab for automatic cleanup:
```bash
# Run every Sunday at 2 AM
0 2 * * 0 cd /path/to/project && cargo sweep --time 14
```

## Coverage Configuration

### Cargo.toml Profile

The project is configured to minimize debug info for third-party dependencies:

```toml
[profile.dev.package."*"]
debug = 0
```

This:
- ✅ Reduces build size and compilation time
- ✅ Maintains full coverage for workspace crates
- ✅ Doesn't track third-party library coverage (you don't test those anyway)

## CI/CD Integration

### Coverage Workflow

Coverage is automatically run on:
- Push to `main` or `develop`
- Pull requests to `main`

Results are uploaded to Codecov (if configured) and stored as build artifacts for 30 days.

### Local vs CI Strategy

| Scenario | Tool | Frequency | Output |
|----------|------|-----------|--------|
| Daily development | `cargo test` / `just test` | Every save | Console output |
| Pre-PR self-check | `cargo llvm-cov --html` | Before PR | HTML report |
| CI/CD automation | GitHub Actions | On push/PR | Codecov + artifacts |

## Cleanup Strategy

### When to Clean

- **After viewing coverage**: `just test-coverage-clean`
- **Weekly maintenance**: `just clean-sweep` (7-day old cache)
- **Before major branch switch**: `just clean-sweep-deps`
- **Nuclear option**: `just clean` (full `cargo clean` — slow rebuild)

### What Gets Cleaned

| Command | Removes | Impact |
|---------|---------|--------|
| `test-coverage-clean` | `.profraw`, coverage reports | Fast rebuild |
| `clean-sweep` | Old build cache (>7 days) | Minimal impact |
| `clean-sweep-deps` | Unused dependency caches | Medium impact |
| `clean` | Everything | Full rebuild required |

## Troubleshooting

### cargo-llvm-cov not installed

```bash
cargo install cargo-llvm-cov
```

If llvm-tools component is missing:
```bash
rustup component add llvm-tools
```

### cargo-sweep not installed

```bash
cargo install cargo-sweep
```

### Coverage builds are slow

- First coverage run will be slow (full rebuild with instrumentation)
- Subsequent runs are faster due to incremental compilation
- The `[profile.dev.package."*"]` setting already optimizes this

### Large coverage artifacts

Run cleanup:
```bash
just test-coverage-clean
```

## Best Practices

1. **Don't run coverage on every save** — use `cargo test` for fast feedback
2. **Check coverage before PRs** — ensure new code is tested
3. **Clean up after viewing reports** — don't let `.profraw` files accumulate
4. **Use cargo-sweep regularly** — keeps disk usage reasonable
5. **Let CI handle heavy lifting** — upload to Codecov, don't store locally

## Commands Reference

```bash
# ── Testing ──────────────────────────────────────────────
just test                  # Fast unit tests
just test-nextest          # Parallel tests with nextest
just test-coverage         # Generate LCOV coverage
just test-coverage-html    # Generate + open HTML report
just test-coverage-clean   # Remove coverage artifacts

# ── Cleanup ──────────────────────────────────────────────
just clean                 # Full cargo clean (slow)
just clean-sweep           # Smart cleanup: 7+ day old cache
just clean-sweep-deps      # Remove unused dependency cache
just clean-coverage        # Remove only coverage artifacts

# ── Via moon ─────────────────────────────────────────────
moon run repo:test-unit
moon run repo:test-coverage
moon run repo:test-coverage-html
moon run repo:clean-sweep
```
