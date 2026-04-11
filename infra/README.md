# Infrastructure

Declarative infrastructure definitions for deployment, orchestration, and security.

## Structure

```
infra/
├── docker/                    # Podman-compatible Dockerfiles + Compose
│   ├── compose/
│   │   ├── app.yaml           # Main app stack (gateway + api + surrealdb + web)
│   │   └── observability.yaml # OpenObserve + Vector (logs/metrics/traces)
│   ├── Dockerfile.api         # Axum API service (distroless runtime)
│   ├── Dockerfile.web         # SvelteKit frontend (static-web-server, Rust)
│   ├── Dockerfile.gateway     # Pingora reverse proxy + static-web-server
│   └── docker-entrypoint-gateway.sh  # Gateway container entrypoint
├── k3s/                       # Production Kubernetes (single-binary K8s)
│   ├── base/                  # Base resources (namespace, deployments, services, ingress)
│   ├── overlays/              # Environment-specific overrides
│   │   ├── dev/               # Minimal resources, debug logging
│   │   ├── staging/           # Production-like configuration
│   │   └── prod/              # HPA, multi-replica, strict quotas
│   └── scripts/
│       ├── bootstrap-k3s.sh   # One-click k3s installation
│       └── deploy.sh          # kustomize deployment entrypoint
├── security/                  # Security configuration
│   ├── sops/                  # Secret encryption (SOPS + age)
│   │   ├── .sops.yaml         # Encryption rules
│   │   └── secrets.enc.yaml   # Encrypted secrets template
│   └── policies/              # Kubernetes security policies
│       ├── network-policy.yaml  # Default deny network isolation
│       └── pod-security.yaml    # Resource quotas + LimitRange
└── terraform/                 # Cloud resource provisioning (Phase 2+)
    ├── modules/               # Reusable Terraform modules
    └── environments/          # dev/prod instances
```

## Quick Start

### Development (Podman Compose)

```bash
# Start gateway + API + SurrealDB + web-static
podman compose -f infra/docker/compose/app.yaml up -d

# Start with observability stack (OpenObserve + Vector)
podman compose -f infra/docker/compose/app.yaml up -d
podman compose -f infra/docker/compose/observability.yaml up -d

# Or use just commands
just deploy dev
just deploy dev-full

# Stop all services
just stop dev
```

### Production (k3s)

```bash
# Bootstrap k3s on a fresh VPS
just deploy bootstrap-k3s

# Deploy to dev environment
just deploy prod ENV=dev

# Deploy to production
just deploy prod ENV=prod
```

### Bare-Metal VPS Setup

```bash
# On a fresh VPS, install all required tools (podman instead of docker)
sudo bash ops/scripts/bootstrap/vps.sh
```

## Stack

| Layer | Technology |
|-------|-----------|
| Container runtime | **Podman** (daemonless, rootless) |
| Reverse proxy | **Pingora** (Cloudflare's Rust proxy) |
| Static files | **static-web-server** (Rust) |
| API | **Axum** (Rust) |
| Database | **Turso** (libSQL) + **SurrealDB** |
| Cache | **Moka** (in-process) |
| Event bus | In-process (tokio broadcast, Phase 1) |
| Orchestrator | **k3s** (single-binary K8s) |
| Secrets | **SOPS** + **age** |

## Evolution Path

| Phase | Infrastructure |
|-------|---------------|
| **Phase 1** (Current) | Podman Compose (dev) + k3s manifests defined |
| **Phase 2** | k3s deployed to VPS, all services running |
| **Phase 3** | Terraform for cloud resources |
| **Phase 4** | Multi-region k3s + edge routing |

See [GOAL.md](../../docs/GOAL.md) for the full evolution roadmap.
