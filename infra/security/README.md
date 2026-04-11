# infra/security — Security Stack

Lightweight security configuration for the application.

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

- All container images run as **nonroot** (distroless/static)
- **No postgres/redis** sidecars → reduced attack surface
- **Pingora** (Rust) instead of nginx → memory safety
- **Moka** (in-process) instead of Redis → no network-exposed cache

## Future Enhancements

- [ ] Casbin RBAC/ABAC policies (for fine-grained API authorization)
- [ ] Regorus/OPA policies (for admission control)
- [ ] External-secrets operator for cloud KMS integration
