# Local Development Infrastructure

This directory contains infrastructure configurations for local development environments.

## Quick Start

### Start All Services

```bash
# Using the bootstrap script
bash infra/local/scripts/bootstrap.sh up

# Or manually with docker compose
docker compose -f infra/docker/compose/core.yaml up -d
```

### Stop All Services

```bash
# Using the bootstrap script
bash infra/local/scripts/bootstrap.sh down

# Or manually
docker compose -f infra/docker/compose/core.yaml down
```

### View Service Status

```bash
bash infra/local/scripts/bootstrap.sh status
```

### View Logs

```bash
bash infra/local/scripts/bootstrap.sh logs
```

## Services

### Turso/libSQL Database

- **HTTP API**: http://localhost:8080
- **gRPC**: grpc://localhost:5001
- **Profile**: `full` (only starts with `--profile full`)

**Note**: By default, the application uses embedded libSQL (local SQLite files).
The sqld service is only needed if you want to test client/server mode.

### NATS Message Broker

- **Client URL**: nats://localhost:4222
- **Monitoring**: http://localhost:8222
- **Features**: JetStream enabled for persistent messaging

**Usage in code**:
```rust
use runtime::adapters::memory::MemoryPubSub;
// Or for production:
// use runtime::adapters::nats::NatsPubSub;
```

### Valkey Cache (Redis-compatible)

- **URL**: redis://localhost:6379
- **CLI**: `redis-cli -h localhost -p 6379`
- **Max Memory**: 256MB
- **Eviction Policy**: allkeys-lru

**Usage in code**:
```rust
use runtime::adapters::memory::MemoryState;
// Or for production:
// use cache_adapters::valkey::ValkeyState;
```

### MinIO Object Storage

- **API**: http://localhost:9000
- **Console**: http://localhost:9001
- **Credentials**: minioadmin / minioadmin
- **Pre-created Buckets**:
  - `uploads` - User file uploads
  - `backups` - Database backups
  - `temp` - Temporary storage

**Usage in code**:
```rust
// Use S3-compatible SDK with these settings:
let config = ClientConfig {
    endpoint: "http://localhost:9000",
    access_key: "minioadmin",
    secret_key: "minioadmin",
    region: "us-east-1",
    bucket: "uploads",
};
```

## Environment Variables

Add these to your `.env` file for local development:

```bash
# Database
DATABASE_URL=libsql://localhost:8080
# Or for embedded mode:
DATABASE_PATH=./data/app.db

# NATS
NATS_URL=nats://localhost:4222

# Cache/Redis
REDIS_URL=redis://localhost:6379

# MinIO/S3
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_BUCKET=uploads
S3_REGION=us-east-1

# Auth
JWT_SECRET=dev-secret-change-in-production
SESSION_TTL=86400  # 24 hours in seconds
```

## Database Migrations

Run migrations against the local database:

```bash
# Using the migration runner (when implemented)
just migrate-local

# Or manually apply SQL files
sqlite3 ./data/app.db < infra/local/seeds/init.sql
```

## Troubleshooting

### Services Not Starting

Check logs for specific errors:
```bash
bash infra/local/scripts/bootstrap.sh logs
```

### Port Already in Use

If a port is already in use, stop the conflicting service or change the port mapping in `core.yaml`.

### unhealthy Services

Services have health checks configured. If a service remains unhealthy:
1. Check logs: `bash infra/local/scripts/bootstrap.sh logs`
2. Restart the service: `docker compose -f infra/docker/compose/core.yaml restart <service-name>`
3. Re-create the container: `docker compose -f infra/docker/compose/core.yaml up -d --force-recreate <service-name>`

### Reset All Data

To start fresh (WARNING: deletes all data):
```bash
docker compose -f infra/docker/compose/core.yaml down -v
docker compose -f infra/docker/compose/core.yaml up -d
```

## Next Steps

After starting infrastructure, proceed to:
1. **Database Setup**: Run migrations (`ops/migrations/`)
2. **Seed Data**: Load test data (`infra/local/seeds/`)
3. **Start Services**: Run the application (`just dev-web`, `just dev-desktop`)
4. **Verify**: Check health endpoints

## See Also

- [Main Docker Compose](../../docker/compose/app.yaml) - Application stack
- [Observability](../../docker/compose/observability.yaml) - Monitoring stack
- [Kubernetes Manifests](../kubernetes/) - Production deployments
