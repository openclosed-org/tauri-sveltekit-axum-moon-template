# GitOps Guide (Flux)

> Manage deployments through Git using Flux CD.

## Prerequisites

- K3s cluster running (see [K3s Guide](./k3s-cluster.md))
- `flux` CLI installed
- GitHub/GitLab repository with infrastructure code
- age key for SOPS decryption (see [Secret Management](./secret-management.md))

## Architecture

```
┌─────────────────┐
│  Git Repository │
│  (infra/)       │
└────────┬────────┘
         │ Flux watches
         ▼
┌─────────────────┐     ┌──────────────────┐
│  Flux           │────▶│  Kubernetes      │
│  Controllers    │     │  Cluster         │
└─────────────────┘     └──────────────────┘
```

## Step 1: Install Flux CLI

```bash
# Install Flux CLI
brew install fluxcd/tap/flux

# Or
curl -s https://fluxcd.io/install.sh | sudo bash

# Verify
flux --version
```

## Step 2: Bootstrap Flux

```bash
# Set your Git details
export GITHUB_TOKEN=your-token
export GITHUB_USER=your-username
export GITHUB_REPO=tauri-sveltekit-axum-moon-template

# Bootstrap Flux
flux bootstrap github \
  --owner=$GITHUB_USER \
  --repository=$GITHUB_REPO \
  --branch=main \
  --path=infra/gitops/flux \
  --personal

# Verify Flux is running
flux check
```

## Step 3: Flux Configuration

### Directory Structure

```
infra/gitops/flux/
├── apps/
│   └── api.yaml              # API application Kustomization
└── infrastructure/
    └── infrastructure.yaml   # Infrastructure components Kustomization
```

### Infrastructure Kustomization

```yaml
# infra/gitops/flux/infrastructure/infrastructure.yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: infrastructure
  namespace: flux-system
spec:
  interval: 10m0s
  path: ./infra/kubernetes/addons
  prune: true
  sourceRef:
    kind: GitRepository
    name: flux-system
  healthChecks:
  - apiVersion: apps/v1
    kind: StatefulSet
    name: nats
    namespace: infrastructure
  - apiVersion: apps/v1
    kind: StatefulSet
    name: valkey
    namespace: infrastructure
  - apiVersion: apps/v1
    kind: StatefulSet
    name: minio
    namespace: infrastructure
```

### Application Kustomization

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
  dependsOn:
  - name: infrastructure
  healthChecks:
  - apiVersion: apps/v1
    kind: Deployment
    name: web-bff
    namespace: default
  postBuild:
    substitute: {}
    substituteFrom:
    - kind: ConfigMap
      name: cluster-config
```

## Step 4: Deploy

### Push to Git

```bash
# Make changes to infrastructure/ or infra/k3s/
git add infra/
git commit -m "feat: add new NATS configuration"
git push

# Flux will automatically reconcile within 10 minutes
```

### Manual Reconciliation

```bash
# Reconcile infrastructure
flux reconcile kustomization infrastructure

# Reconcile applications
flux reconcile kustomization api

# Check status
flux get kustomizations
```

## Step 5: Monitor

```bash
# List all Kustomizations
flux get kustomizations

# View events
flux events

# Check specific Kustomization
flux get kustomization api --watch
```

## Environment Overlays

### Dev

```yaml
# infra/k3s/overlays/dev/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
- ../../base
- ../../addons
patches:
- target:
    kind: Deployment
    name: web-bff
  patch: |
    - op: replace
      path: /spec/replicas
      value: 1
```

### Staging

```yaml
# infra/k3s/overlays/staging/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
- ../../base
- ../../addons
patches:
- target:
    kind: Deployment
    name: web-bff
  patch: |
    - op: replace
      path: /spec/replicas
      value: 2
```

### Production

```yaml
# infra/k3s/overlays/prod/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
- ../../base
- ../../addons
patches:
- target:
    kind: Deployment
    name: web-bff
  patch: |
    - op: replace
      path: /spec/replicas
      value: 3
```

## Rollback

```bash
# Revert Git commit
git revert HEAD
git push

# Or reconcile to previous state
flux reconcile kustomization api --with-source
```

## Troubleshooting

### Kustomization not applied
```bash
# Check Kustomization status
flux get kustomization api

# Check events
flux events --for Kustomization/api

# Force reconciliation
flux reconcile kustomization api --with-source
```

### Health check failing
```bash
# Check deployment
kubectl get deployment web-bff

# Check pods
kubectl get pods -l app=web-bff

# Check logs
kubectl logs deployment/web-bff
```

### SOPS decryption failed
```bash
# Verify age key
cat ~/.config/sops/age/keys.txt

# Re-bootstrap Flux with correct key
flux bootstrap github --owner=$GITHUB_USER --repository=$GITHUB_REPO --path=infra/gitops/flux
```
