# ADR-004: K3s + Cilium + Gateway API + Flux

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context
The system needs a production-ready Kubernetes deployment strategy that supports:
- Local development with minimal infrastructure
- Staging environments for testing
- Production deployments with high availability
- GitOps-based continuous delivery
- Network security and service isolation
- Easy scaling and maintenance

Traditional choices considered:
1. **EKS/GKE**: Managed but expensive, vendor lock-in
2. **Vanilla Kubernetes**: Powerful but complex to operate
3. **Docker Swarm**: Simple but limited features
4. **Nomad**: Flexible but smaller ecosystem
5. **K3s**: Lightweight, production-ready, CNCF certified

## Decision
We selected a **K3s + Cilium + Gateway API + Flux** stack:

### K3s (Kubernetes Distribution)
- Lightweight: Single binary, <1GB RAM
- Production-ready: CNCF certified, Rancher-backed
- Easy upgrades: Single binary replacement
- SQLite default, supports external DB
- Perfect for edge computing and small clusters

### Cilium (CNI + Network Policy)
- eBPF-based: High performance, low overhead
- NetworkPolicy: Fine-grained network control
- Service mesh: L7 awareness without sidecars
- Observability: Hubble for network visibility
- Gateway API: Native support

### Gateway API (Ingress/Routing)
- Kubernetes native: Standard API, not Ingress-specific
- Multi-tenancy: Gateway classes per tenant/environment
- Advanced routing: Header-based, path-based, weight-based
- TLS management: Centralized certificate handling

### Flux (GitOps)
- Declarative: Git as source of truth
- Automated reconciliation: Continuous sync
- SOPS integration: Encrypted secrets
- Health checks: Deployment verification
- Rollback: Git revert for instant rollback

### Infrastructure Structure
```
infra/kubernetes/
├── bootstrap/          # Cluster initialization
├── base/              # Namespaces, RBAC, NetworkPolicy
├── addons/            # NATS, Valkey, MinIO, etc.
├── rendered/          # Generated workloads
└── overlays/          # Environment-specific overrides

infra/gitops/flux/
├── apps/              # Application definitions
└── infrastructure/    # Infrastructure components
```

### Rationale
1. **Cost-effective**: K3s runs on minimal hardware
2. **Security**: Cilium provides network isolation at L3-L7
3. **GitOps**: Flux ensures infrastructure is version-controlled
4. **Standardization**: Gateway API is the future of Kubernetes ingress
5. **Simplicity**: K3s reduces operational overhead

## Consequences
### What becomes easier
- Local-to-prod: Same Kubernetes for dev and prod
- GitOps: Changes via PR, automatic deployment
- Network security: Cilium policies prevent lateral movement
- Scaling: K3s clusters are easy to add/remove

### What becomes more difficult
- Learning curve: Kubernetes concepts are complex
- Debugging: eBPF and Gateway API require new skills
- Bootstrap: Initial cluster setup is non-trivial
- Secrets management: Requires SOPS + age integration

### Trade-offs
- **Pros**: Cost, security, GitOps, standardization, simplicity
- **Cons**: Learning curve, debugging complexity, bootstrap effort

## References
- `infra/k3s/base/` - Base Kubernetes manifests
- `infra/kubernetes/addons/` - Infrastructure addons
- `infra/gitops/flux/` - Flux GitOps configuration
- `infra/security/sops/` - Secret management setup
- [K3s Documentation](https://rancher.com/docs/k3s/latest/)
- [Cilium Documentation](https://docs.cilium.io/)
- [Flux Documentation](https://fluxcd.io/flux/)
