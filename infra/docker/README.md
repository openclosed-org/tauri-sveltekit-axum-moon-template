# Docker / Podman

Container configurations and multi-stage builds. Fully compatible with **Podman** (daemonless, rootless).

## Structure

```
docker/
├── compose/
│   ├── app.yaml              # Main application stack (Podman Compose)
│   └── observability.yaml    # OpenObserve + Vector
├── compose.dev.yml           # DEPRECATED — legacy Docker Compose
├── Dockerfile.api            # Axum API (distroless/static runtime)
├── Dockerfile.web            # SvelteKit web (static-web-server, Rust)
├── Dockerfile.gateway        # Pingora gateway + static-web-server
└── docker-entrypoint-gateway.sh  # Container-only entrypoint (starts both processes)
```

## Quick Start

```bash
# Start local core infrastructure
cargo run -p repo-tools -- infra local up

# Or use just commands
just deploy-dev
just status-dev
```

## Services

| Service | Port | Purpose |
|---------|------|---------|
| gateway | 3000 | Pingora reverse proxy (entry point) |
| api | 3001 | Axum API server |
| web-static | 3002 | static-web-server (SvelteKit SPA) |
| surrealdb | 8000 | SurrealDB (dev sidecar) |
| openobserve | 5080 | Observability UI (admin@localhost / admin) |

## Stack Decisions

| Old | New | Reason |
|-----|-----|--------|
| Docker | Podman | Daemonless, rootless, systemd-integrated |
| nginx | Pingora + static-web-server | Rust-native, better performance, type safety |
| PostgreSQL | Turso (libSQL) / SurrealDB | Local-first, embedded, no sidecar needed in prod |
| Redis | Moka | In-process cache, zero infrastructure |
| NATS server | in-process event bus (Phase 1) | tokio broadcast channels, simpler for single-process |

## Images

- **API**: Multi-stage build → `gcr.io/distroless/static-debian12:nonroot` (~6MB)
- **Web**: Multi-stage build → `ghcr.io/static-web-server/static-web-server:2` (Rust, ~10MB)
- **Gateway**: Multi-stage build → `gcr.io/distroless/static-debian12:nonroot` with Pingora + static-web-server (~15MB)
