# Scripts

Cross-platform development and运维 scripts for the monorepo.

## Structure

```
scripts/
├── doctor.ts              # Toolchain and config health check
├── dev-desktop.ts         # Desktop dev environment (API + Tauri)
├── typegen.ts             # Generate contract bindings and sync to frontend
├── boundary-check.ts      # Architecture dependency validation
├── lib/
│   └── spawn.ts           # Shared cross-platform spawn utilities
├── e2e/
│   ├── runtime-preflight.ts    # E2E preflight gate
│   └── run-e2e-gate.ts         # Full E2E pipeline orchestrator
└── deploy/
    └── generate-service.sh     # Service file generation
```

## Usage

These scripts are invoked via `just` commands (see `justfiles/`) or `moon run repo:<task>`.

```bash
just doctor       # → bun run scripts/doctor.ts
just typegen      # → bun run scripts/typegen.ts
just boundary-check  # → bun run scripts/boundary-check.ts
```

## Platform Support

All scripts use `scripts/lib/spawn.ts` for cross-platform process management.
They work on Windows, macOS, and Linux with automatic platform detection.
