# ADR-004: K3s + Flux Delivery Direction

## Status
- [x] Proposed
- [ ] Accepted
- [ ] Deprecated
- [ ] Superseded

> Implementation status: K3s overlays, Flux apps, and SOPS integration have real landing points.
> Cilium and Gateway API are still target-state options, not current default infrastructure facts.
> The current gateway implementation is a lightweight Pingora reverse proxy, not a full Gateway API control plane.

## Context

The repository needs a deployment direction that supports:

1. local and single-node validation
2. GitOps-based reconciliation
3. environment overlays
4. encrypted secret delivery
5. later topology expansion without rewriting business code

Earlier discussions also considered stronger networking and ingress choices such as Cilium and Gateway API,
but those are not required to describe the current default path.

## Decision

We use **K3s + Flux + SOPS** as the current delivery direction.

`Cilium` and `Gateway API` remain deferred platform options that may be introduced later if the deployment topology truly requires them.

### Current default delivery stack

#### K3s

- lightweight enough for the repository's single-node-first direction
- supports environment overlays and gradual topology evolution
- aligns with the current `infra/k3s/**` layout

#### Flux

- keeps Git as the declared delivery source
- reconciles app overlays from `infra/gitops/flux/**`
- integrates with SOPS decryption

#### SOPS + age

- encrypted secrets in Git
- consistent secret shape between local `sops-run` and cluster delivery
- avoids `.env` becoming the backend reference path

### Current infrastructure structure

```text
infra/k3s/
  base/        Base manifests
  overlays/    Environment-specific overrides
  scripts/     Bootstrap and deploy helpers

infra/gitops/flux/
  apps/              Application definitions
  infrastructure/    Infrastructure components
```

### Deferred infrastructure options

The following are deferred until the real deployment path requires them:

1. Cilium as the default networking layer
2. Gateway API as the default ingress/routing control plane
3. more advanced TLS and edge policy machinery
4. service-mesh-like capabilities

## Consequences

### What becomes easier

- aligning dev/staging/prod delivery structure
- GitOps-based reconciliation
- keeping secret delivery consistent with the current backend reference path
- evolving topology gradually as the codebase hardens

### What becomes more difficult

- Kubernetes still introduces operational complexity
- networking and ingress decisions remain partially deferred
- agents must distinguish between current K3s/Flux facts and later platform options

### Trade-offs

- **Pros**: clear delivery path, incremental topology evolution, GitOps alignment
- **Cons**: bootstrap effort, deferred networking decisions still need later validation

## References

- `infra/k3s/base/` - base manifests
- `infra/k3s/overlays/` - environment overlays
- `infra/gitops/flux/` - Flux GitOps configuration
- `infra/security/sops/` - secret management setup
- `servers/gateway/src/main.rs` - current lightweight gateway implementation
- `docs/operations/gitops.md`
