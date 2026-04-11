# Base k3s manifests

Base resources shared across all environments. These are the foundational
Kubernetes manifests that define the application structure.

## Stack

| Component | Technology | Notes |
|-----------|-----------|-------|
| Gateway | Pingora (Rust) | Reverse proxy, replaces nginx |
| API | Axum (Rust) | Backend service, distroless runtime |
| Web | static-web-server (Rust) | SvelteKit SPA, distroless runtime |
| Database | SurrealDB + Turso | No postgres/redis sidecars |
| Cache | Moka | In-process, zero infrastructure |
| Events | In-process (tokio) | Phase 1, no NATS server needed |

## Resources

| File | Purpose |
|------|---------|
| `namespace.yaml` | Application namespace with restricted Pod Security Standard |
| `configmap.yaml` | Environment variable injection (SurrealDB, Moka, gateway config) |
| `deployment-api.yaml` | Axum API server deployment |
| `deployment-web.yaml` | SvelteKit frontend deployment (static-web-server) |
| `service.yaml` | ClusterIP services |
| `ingress.yaml` | Traefik ingress routing rules |
| `kustomization.yaml` | Kustomize entrypoint |

## Usage

Apply directly or via an overlay:

```bash
# Direct (not recommended — use overlays instead)
kubectl apply -k infra/k3s/base

# Via overlay (recommended)
kubectl apply -k infra/k3s/overlays/dev
```
