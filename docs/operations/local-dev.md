# Local Development Guide

> Everything you need to know to run the platform locally.

## Prerequisites

Before starting, ensure you have the following installed:

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable (from rust-toolchain.toml) | `rustup install stable` |
| Node.js | 20+ | `mise install node` |
| Bun | latest | `mise install bun` |
| Docker + Compose | 24+ | Docker Desktop |
| Just | latest | `mise install just` |
| Mise | latest | [mise.jdx.dev](https://mise.jdx.dev/) |

Verify setup:
```bash
mise doctor
just doctor
```

## Quick Start

### 1. Start Infrastructure

```bash
# Start NATS, Valkey, MinIO (embedded libSQL is in-process)
bash infra/local/scripts/bootstrap.sh up

# Check status
bash infra/local/scripts/bootstrap.sh status
```

This starts:
| Service | Port | Purpose |
|---------|------|---------|
| NATS | 4222 (client), 8222 (monitoring) | Message broker |
| Valkey | 6379 | Cache |
| MinIO API | 9000 | Object storage |
| MinIO Console | 9001 | Storage UI (http://localhost:9001, minioadmin/minioadmin) |

**Optional**: Start sqld (client/server mode):
```bash
docker compose --profile full -f infra/docker/compose/core.yaml up -d
```

### 2. Seed Data (Optional)

```bash
# Seed data is applied automatically when starting infrastructure
# Manual seed:
cat infra/local/seeds/init.sql | sqlite3 data.db
```

### 3. Run Migrations

```bash
# Run database migrations for local environment
bash ops/migrations/runner/migrate.sh up local
```

### 4. Start Web Application

```bash
# Start web BFF (Axum server)
just dev-web

# In another terminal, start SvelteKit dev server
just dev-web-frontend
```

### 5. Start Desktop Application

```bash
# Start desktop app in dev mode
just dev-desktop
```

### 6. Start Workers (Optional)

```bash
# Start indexer worker
cargo run -p worker-indexer

# Start outbox relay
cargo run -p worker-outbox-relay
```

## Stopping Everything

```bash
# Stop all Docker services
bash infra/local/scripts/bootstrap.sh down

# Stop individual services
docker compose -f infra/docker/compose/core.yaml down

# Stop with volumes (destroys data)
docker compose -f infra/docker/compose/core.yaml down -v
```

## Viewing Logs

```bash
# View all logs
bash infra/local/scripts/bootstrap.sh logs

# View specific service
bash infra/local/scripts/bootstrap.sh logs nats
bash infra/local/scripts/bootstrap.sh logs valkey
bash infra/local/scripts/bootstrap.sh logs minio
```

## Accessing Services

| Service | URL | Credentials |
|---------|-----|-------------|
| Web App | http://localhost:5173 | N/A |
| Web BFF API | http://localhost:3000 | N/A |
| Admin BFF API | http://localhost:3001 | N/A |
| MinIO Console | http://localhost:9001 | minioadmin / minioadmin |
| NATS Monitoring | http://localhost:8222 | N/A |
| Valkey CLI | `docker exec -it valkey valkey-cli` | N/A |

## Development Workflow

### Adding a New Service

1. Create service structure: `services/<name>/`
2. Add to workspace `Cargo.toml`
3. Add to `platform/model/services/<name>.yaml`
4. Run `just validate-platform`
5. Implement domain → application → ports
6. Write tests
7. Wire to BFF

### Adding a New API Endpoint

1. Define in `packages/contracts/` (contract-first)
2. Implement in service
3. Add handler in BFF
4. Update OpenAPI spec
5. Run `just gen-contracts`
6. Verify zero drift: `git diff --exit-code`

### Testing

```bash
# Unit tests
just test-unit

# All tests
just test

# Service-specific
cargo test -p auth-service
cargo test -p user-service
cargo test -p tenant-service

# E2E tests
just test-e2e-full
```

### Validation

```bash
# Validate platform models
just validate-platform

# Validate dependency graph
just validate-deps

# Validate security
just validate-security

# Validate observability
just validate-observability

# Check all generated artifacts
just verify-generated
```

## Troubleshooting

### NATS won't start
```bash
# Check port conflict
lsof -i :4222

# Restart
docker compose -f infra/docker/compose/core.yaml restart nats
```

### Valkey connection refused
```bash
# Check Valkey is running
docker compose -f infra/docker/compose/core.yaml ps

# Test connection
docker exec -it valkey valkey-cli ping
```

### libSQL file locked
```bash
# Kill processes holding lock
lsof | grep data.db

# Remove lock (dev only, destroys data)
rm -f data.db
```

### Build fails after restructuring
```bash
# Clean and rebuild
cargo clean
cargo check --workspace
```

## Environment Variables

Create `.env` from `.env.example`:

```bash
cp .env.example .env
```

Key variables:
| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `libsql://file:data.db` | Local database URL |
| `NATS_URL` | `nats://localhost:4222` | NATS connection |
| `VALKEY_URL` | `redis://localhost:6379` | Valkey/Redis connection |
| `MINIO_URL` | `http://localhost:9000` | MinIO endpoint |
| `MINIO_ACCESS_KEY` | `minioadmin` | MinIO access key |
| `MINIO_SECRET_KEY` | `minioadmin` | MinIO secret key |
