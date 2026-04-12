# Flux GitOps Configuration

This directory contains Flux CD configuration for GitOps-based deployment.

## Structure

```
infra/gitops/flux/
├── apps/              # Application definitions
│   ├── api.yaml       # API service deployment
│   ├── web.yaml       # Web frontend deployment
│   └── gateway.yaml   # Gateway deployment
├── infrastructure/    # Infrastructure components
│   ├── nats.yaml      # NATS message broker
│   ├── valkey.yaml    # Valkey cache
│   └── minio.yaml     # MinIO object storage
└── policies/          # Policy definitions
    ├── namespace.yaml # Namespace configuration
    ├── rbac.yaml      # RBAC configuration
    └── quotas.yaml    # Resource quotas
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
  --repository=tauri-sveltekit-axum-moon-template \
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

## Add Applications

### API Service

```yaml
# infra/gitops/flux/apps/api.yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: api
  namespace: flux-system
spec:
  interval: 10m0s
  path: ./infra/k3s/overlays/dev
  prune: true
  sourceRef:
    kind: GitRepository
    name: flux-system
  decryption:
    provider: sops
    secretRef:
      name: sops-age
  healthChecks:
    - apiVersion: apps/v1
      kind: Deployment
      name: api
      namespace: app
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

## See Also

- [Flux Documentation](https://fluxcd.io/flux/)
- [SOPS Integration](../../security/sops/SETUP.md)
- [Kubernetes Manifests](../../k3s/base/)
