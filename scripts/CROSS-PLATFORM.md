# Cross-Platform Scripts Guide

All scripts are now written in **TypeScript (.ts)** and run via **Bun**.

They work on Windows, macOS, and Linux with automatic platform detection via `scripts/lib/spawn.ts`.

## Script Inventory

| Script | Purpose | Stage |
|--------|---------|-------|
| `scripts/doctor.ts` | Toolchain and config health check | Setup |
| `scripts/typegen.ts` | Generate backend contract bindings | Codegen |
| `scripts/boundary-check.ts` | Architecture dependency validation | Quality Gate |
| `scripts/test/run.ts` | Rust test runner (nextest, coverage, hack, mutants) | Testing |
| `scripts/lib/spawn.ts` | Shared cross-platform spawn utilities | Library |

## Usage

```bash
# Run via moon (recommended for CI/automation)
moon run repo:doctor
moon run repo:typegen
moon run repo:boundary-check

# Run via just (recommended for humans)
just doctor
just typegen
just verify

# Run directly with bun
bun run scripts/doctor.ts
bun run scripts/test/run.ts all
```

## Shared Library

All scripts use `scripts/lib/spawn.ts` for:
- Cross-platform process spawning (Windows/macOS/Linux)
- Async execution with proper error handling
- Process tree cleanup
- Port waiting and availability checks
- Tool availability checks

This eliminates code duplication and ensures consistent behavior across all scripts.

## Migration History

- **Phase 1**: Original `.sh` scripts (Unix only)
- **Phase 2**: Migrated to `.mjs` (cross-platform but no types)
- **Phase 3** (current): Migrated to `.ts` with shared library (type-safe, DRY, cross-platform)

All `.sh` and `.mjs` files have been removed. Only `.ts` files remain.
