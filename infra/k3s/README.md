# k3s — Production Kubernetes (Single-Binary K8s)

> k3s is a certified Kubernetes distribution with a single <100MB binary.
> It includes containerd, Traefik ingress, and local-path storage out of the box.

## Structure

```
k3s/
├── base/               # Base resources shared across environments
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── deployment-api.yaml
│   ├── deployment-web.yaml
│   ├── service.yaml
│   └── ingress.yaml
├── overlays/
│   ├── dev/            # Development: minimal resources, single replica, debug on
│   ├── staging/        # Staging: production-like, moderate resources
│   └── prod/           # Production: multi-replica, HPA, strict resource quotas
└── scripts/
    ├── bootstrap-k3s.sh    # One-click k3s install + dependencies
    └── deploy.sh           # kubectl/kustomize deployment entrypoint
```

## Evolution Path

| Phase | State |
|-------|-------|
| **Phase 1** (Current) | Docker Compose for dev, k3s manifests defined |
| **Phase 2** | k3s deployed to single VPS, all services running |
| **Phase 3** | Multi-region k3s clusters with edge routing |

## Quick Start

```bash
# Bootstrap k3s on a fresh VPS
./infra/k3s/scripts/bootstrap-k3s.sh

# Deploy to dev
kubectl apply -k infra/k3s/overlays/dev

# Deploy to prod
kubectl apply -k infra/k3s/overlays/prod
```
