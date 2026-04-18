# Flux GitOps Configuration

This directory contains Flux CD configuration for GitOps-based deployment to Kubernetes (k3s staging and production).

## Structure

```
infra/gitops/flux/
├── apps/                  # Application definitions
│   ├── api.yaml           # API service deployment
│   ├── web.yaml           # Web BFF deployment
│   ├── gateway.yaml       # Edge gateway deployment
│   ├── outbox-relay-worker.yaml
│   └── projector-worker.yaml
├── infrastructure/        # Infrastructure components
│   ├── nats.yaml          # NATS message broker
│   ├── valkey.yaml        # Valkey cache (Redis-compatible)
│   └── minio.yaml         # MinIO object storage (S3-compatible)
├── policies/              # Policy definitions
│   ├── namespace.yaml     # Namespace and ServiceAccount configuration
│   ├── rbac.yaml          # RBAC Role and RoleBinding
│   └── quotas.yaml        # ResourceQuota and LimitRange
└── TOPOLOGY-MAPPING.md    # Topology ↔ Flux configuration mapping
```

## Bootstrap Flux

### 1. Install Flux CLI

```bash
# macOS
brew install fluxcd/tap/flux

# Or using curl
curl -s https://fluxcd.io/install.sh | sudo bash
```

### 2. Bootstrap Flux on Cluster

```bash
# Bootstrap with GitHub repository
flux bootstrap github \
  --owner=<your-github-org> \
  --repository=axum-harness \
  --branch=main \
  --path=infra/gitops/flux \
  --read-write-key \
  --personal

# Or with generic Git server
flux bootstrap git \
  --url=ssh://git@github.com/<org>/<repo>.git \
  --branch=main \
  --path=infra/gitops/flux
```

### 3. Verify Bootstrap

```bash
flux check
flux get sources git
flux get kustomizations
```

## Deploy New Service

1. Create Kustomization file in `apps/` directory
2. Commit and push to repository
3. Flux will automatically apply changes

```bash
git add infra/gitops/flux/apps/my-service.yaml
git commit -m "Add my-service deployment"
git push
```

## Monitor Deployments

```bash
# Get all Kustomizations
flux get kustomizations

# Get specific Kustomization
flux get kustomization api

# View reconciliation logs
flux logs -f

# Suspend automatic reconciliation
flux suspend kustomization api

# Resume reconciliation
flux resume kustomization api

# Trigger manual reconciliation
flux reconcile kustomization api --with-source
```

## Rollback

```bash
# View Kustomization history
flux get kustomization api --show-events

# Rollback to previous revision (requires suspension first)
flux suspend kustomization api
# Revert commit in Git
git revert <bad-commit>
git push
flux resume kustomization api
```

## Current Status

### ✅ Implemented
- Core infrastructure (NATS, Valkey, MinIO)
- Application deployments (API, Web BFF, Admin BFF, Edge Gateway)
- Dedicated outbox-relay-worker Flux Kustomization (disabled by default until shared libSQL/Turso secret is configured)
- Dedicated projector-worker Flux Kustomization (disabled by default until shared libSQL/Turso secret is configured)
- Namespace, RBAC, and Resource Quota policies
- SOPS decryption integration
- Health checks for all components

### ⏳ TODO
- Worker Kustomizations (indexer, scheduler, sync-reconciler)
- Observability stack (OpenObserve, Grafana)
- Dragonfly cache configuration (for k3s-staging topology)
- Network policies (Cilium)

## See Also

- [Flux Documentation](https://fluxcd.io/flux/)
- [SOPS Integration](../../security/sops/SETUP.md)
- [Kubernetes Manifests](../../kubernetes/)
- [Topology Mapping](./TOPOLOGY-MAPPING.md)
