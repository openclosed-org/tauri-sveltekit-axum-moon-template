# Justfiles

All developer and operational commands live here.

## Files

| File | Purpose |
|------|---------|
| `setup.just` | Toolchain installation, doctor, sccache, hakari |
| `dev.just` | Dev servers (fullstack, web, API, desktop, Tauri) |
| `test.just` | Unit, integration, contract, E2E, coverage |
| `quality.just` | Format, lint, boundary-check, verify, deny |
| `build.just` | Workspace build, single-service build, cross-compile |
| `deploy.just` | Docker Compose deploy, systemd deploy, K8s deploy |
| `migrate.just` | DB migrations up/down, status, rollback |
| `processes.just` | Cross-platform process management (ps, stop, ports) |
| `clean.just` | cargo clean, sweep, coverage cleanup |
| `skills.just` | AI agent skills integration |
| `platform.just` | Platform generation, validation, golden baselines |
| `gates.just` | Gate orchestration and verification |
| `sops.just` | SOPS encryption, decryption, key management |

## Usage

```bash
just --list       # Show all available commands
just setup        # Install all dependencies
just dev          # Start full-stack dev
just verify       # Run all quality checks
```
