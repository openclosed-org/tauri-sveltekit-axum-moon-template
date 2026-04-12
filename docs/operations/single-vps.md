# Single VPS Deployment Guide

> Deploy the entire platform on a single VPS using Docker Compose.

## Prerequisites

| Requirement | Specification |
|-------------|---------------|
| CPU | 4+ cores |
| RAM | 8+ GB |
| Storage | 50+ GB SSD |
| OS | Ubuntu 22.04 LTS |
| Docker | 24+ with Compose v2 |

## Architecture

```
┌──────────────────────────────────────────┐
│                VPS                       │
│                                          │
│  ┌─────┐    ┌────────────────────────┐  │
│  │Caddy│───▶│  Application Containers │  │
│  │:443 │    │  (web-bff, workers)     │  │
│  └─────┘    └────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Infrastructure Containers         │  │
│  │  (NATS, Valkey, MinIO, Zitadel)   │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Docker Volumes (persistent data)  │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## Step 1: Server Setup

```bash
# SSH into your VPS
ssh root@your-vps-ip

# Update system
apt update && apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com | sh

# Install Docker Compose (usually included)
docker compose version

# Create application directory
mkdir -p /opt/tauri-platform
cd /opt/tauri-platform
```

## Step 2: Configuration

### Create docker-compose.yml

```yaml
version: '3.8'

services:
  # === Edge ===
  caddy:
    image: caddy:2
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - web-bff
      - admin-bff

  # === Applications ===
  web-bff:
    image: your-registry/web-bff:${TAG:-latest}
    environment:
      - DATABASE_URL=libsql://file:/data/app.db
      - NATS_URL=nats://nats:4222
      - VALKEY_URL=redis://valkey:6379
      - MINIO_URL=http://minio:9000
      - MINIO_ACCESS_KEY=${MINIO_ACCESS_KEY}
      - MINIO_SECRET_KEY=${MINIO_SECRET_KEY}
    volumes:
      - app_data:/data
    depends_on:
      - nats
      - valkey
      - minio

  admin-bff:
    image: your-registry/admin-bff:${TAG:-latest}
    environment:
      - DATABASE_URL=libsql://file:/data/app.db
      - NATS_URL=nats://nats:4222
      - VALKEY_URL=redis://valkey:6379
    volumes:
      - app_data:/data
    depends_on:
      - nats
      - valkey

  # === Workers ===
  indexer-worker:
    image: your-registry/worker-indexer:${TAG:-latest}
    environment:
      - NATS_URL=nats://nats:4222
      - DATABASE_URL=libsql://file:/data/app.db
    depends_on:
      - nats

  outbox-relay:
    image: your-registry/worker-outbox-relay:${TAG:-latest}
    environment:
      - NATS_URL=nats://nats:4222
      - DATABASE_URL=libsql://file:/data/app.db
    depends_on:
      - nats

  # === Infrastructure ===
  nats:
    image: nats:2-alpine
    command: -js -m 8222
    ports:
      - "4222:4222"
      - "8222:8222"
    volumes:
      - nats_data:/data

  valkey:
    image: valkey/valkey:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - valkey_data:/data
    command: valkey-server --appendonly yes

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ACCESS_KEY=${MINIO_ACCESS_KEY}
      - MINIO_SECRET_KEY=${MINIO_SECRET_KEY}
    volumes:
      - minio_data:/data

volumes:
  caddy_data:
  caddy_config:
  app_data:
  nats_data:
  valkey_data:
  minio_data:
```

### Create Caddyfile

```
your-domain.com {
    reverse_proxy /api/* web-bff:3000
    reverse_proxy /* web-bff:3000
}

admin.your-domain.com {
    reverse_proxy admin-bff:3001
}
```

### Create .env

```bash
MINIO_ACCESS_KEY=your-access-key
MINIO_SECRET_KEY=your-secret-key
TAG=latest
```

## Step 3: Build and Deploy

### Option A: Build on Server

```bash
# Copy built binaries to server
# (Build locally, then scp)
scp target/release/web-bff root@vps:/opt/tauri-platform/
```

### Option B: Use Docker Images

```bash
# Build images locally
docker build -f infra/images/Dockerfile.rust-service -t web-bff .

# Or pull from registry
docker pull your-registry/web-bff:latest

# Deploy
docker compose up -d
```

## Step 4: Run Migrations

```bash
# Run migrations
bash ops/migrations/runner/migrate.sh up production
```

## Step 5: Verify Deployment

```bash
# Check all services are running
docker compose ps

# Check health endpoints
curl https://your-domain.com/healthz
curl https://admin.your-domain.com/healthz

# Check logs
docker compose logs -f web-bff
docker compose logs -f indexer-worker
```

## Step 6: Set Up Monitoring

### Install Vector (Log Collector)

```bash
# Install Vector
curl --proto '=https' --tlsv1.2 -sSf https://sh.vector.dev | sh

# Configure Vector
cat > /etc/vector/vector.yaml << 'EOF'
sources:
  docker_logs:
    type: docker_logs

sinks:
  local_file:
    type: file
    inputs: [docker_logs]
    path: /var/log/platform/%Y-%m-%d.log
EOF

# Start Vector
vector --config /etc/vector/vector.yaml &
```

## Updating

```bash
# Pull new images
docker compose pull

# Restart services (rolling)
docker compose up -d --no-deps web-bff
docker compose up -d --no-deps admin-bff
docker compose up -d --no-deps indexer-worker
# ... repeat for other services

# Run new migrations if any
bash ops/migrations/runner/migrate.sh up production
```

## Rolling Back

```bash
# Set previous tag
export TAG=previous-version

# Restart with previous version
docker compose up -d

# Run rollback migrations if needed (manual only — migrations are forward-only)
```

## Troubleshooting

### Service won't start
```bash
docker compose logs web-bff
```

### Database connection failed
```bash
# Check data volume permissions
ls -la /var/lib/docker/volumes/app_data/
```

### Out of memory
```bash
# Check memory usage
free -h

# Check Docker memory
docker stats
```

### Disk full
```bash
# Check disk usage
df -h

# Clean Docker
docker system prune -af
```
