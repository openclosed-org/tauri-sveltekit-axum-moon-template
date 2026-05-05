# infra/security — Security Stack

Lightweight security configuration for the template. This directory describes declared security scaffolding, not a complete production security proof.

## Structure

```
security/
├── sops/                    # Secret encryption (SOPS + age)
│   ├── .sops.yaml           # Encryption rules
│   └── secrets.enc.yaml     # Encrypted secrets template (commit this)
└── policies/                # Kubernetes security policies
    ├── network-policy.yaml  # Network isolation (default deny)
    └── pod-security.yaml    # Resource quotas + LimitRange
```

## Quick Start

### 1. Set up SOPS + age (Secret Encryption)

```bash
# Install tools
brew install age sops  # macOS
# or: cargo install age + apt install sops  # Linux

# Generate age key pair
age-keygen -o infra/security/sops/age.key
# Copy the PUBLIC KEY output

# Update .sops.yaml with your public key
# Then encrypt the secrets template
sops --encrypt --in-place infra/security/sops/secrets.enc.yaml

# Edit secrets (auto-decrypts, auto-encrypts on save)
sops infra/security/sops/secrets.enc.yaml
```

### 2. Apply Security Policies

```bash
# Network policies (default deny + explicit allows)
kubectl apply -f infra/security/policies/network-policy.yaml

# Resource quotas and limits
kubectl apply -f infra/security/policies/pod-security.yaml
```

## Design Decisions

- **SOPS + age**: Chosen over Vault for lightweight setup. No infrastructure needed.
- **Network policies**: Default deny, explicit allow for Traefik → services + DNS
- **Pod security**: "Restricted" PSS enforced at namespace level
- **Resource quotas**: Prevent resource exhaustion attacks

## Stack Compatibility

- First-party application images should run as **nonroot** where enforced by Dockerfiles and Kubernetes security contexts.
- Third-party image posture must be checked per manifest; do not infer nonroot or distroless behavior from this README.
- The minimal backend-core path should not require PostgreSQL or Redis sidecars, but local/cluster profiles can include Valkey, NATS, MinIO, OpenFGA, Rauthy, OpenObserve, OTel Collector, and Vector.
- **Pingora** remains the Rust gateway direction where the gateway is used.
- **Moka** remains the in-process cache direction for backend-core code paths that do not need external cache infrastructure.

## Future Enhancements

- [ ] Casbin RBAC/ABAC policies (for fine-grained API authorization)
- [ ] Regorus/OPA policies (for admission control)
- [ ] External-secrets operator for cloud KMS integration
