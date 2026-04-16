# SOPS + Age Key Management

> **Backend binaries consume standard environment variables.**
> **Environment variables are injected via SOPS/Kustomize/Flux, NOT from `.env` files.**
>
> This document is the single source of truth for age key generation, storage, and Flux integration.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Developer / CI/CD                                          │
│                                                             │
│  1. Edit plaintext template:                                │
│     infra/security/sops/templates/<env>/<deployable>.yaml   │
│                                                             │
│  2. Encrypt with SOPS (uses age key):                       │
│     sops --encrypt template.yaml > <env>/<deployable>.enc.yaml│
│                                                             │
│  3. Commit encrypted file to Git                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  Flux (GitOps Controller)                                   │
│                                                             │
│  1. Reads age key from Secret sops-age (flux-system ns)    │
│  2. Decrypts *.enc.yaml files automatically                 │
│  3. Applies decrypted Kubernetes manifests                  │
│  4. Secrets injected as env vars into pods                  │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  Backend Binary (web-bff, outbox-relay-worker, etc.)       │
│                                                             │
│  1. Reads standard env vars (SERVER_HOST, DATABASE_URL, etc)│
│  2. No awareness of .env, SOPS, or encryption               │
│  3. Same binary runs in dev/staging/prod                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Commands

All SOPS operations are exposed via `just` commands:

```bash
# Generate a new age key (first-time setup)
just sops-gen-age-key

# Show your age public key
just sops-show-age-key

# Create encrypted secrets for dev environment
just sops-encrypt-dev web-bff
just sops-encrypt-dev outbox-relay-worker
just sops-encrypt-dev counter-service

# Edit encrypted secrets (opens editor with decrypted content)
just sops-edit web-bff dev
just sops-edit outbox-relay-worker dev

# Apply decrypted secrets to local k3s cluster
just sops-reconcile ENV=dev

# Run binary with decrypted env vars (no cluster, quick inner loop)
just sops-run web-bff
just sops-run outbox-relay-worker
```

---

## Age Key Generation

### One-Time Setup

```bash
# Generate age key pair
just sops-gen-age-key
```

This creates:
- `~/.config/sops/age/key.txt` — private key (KEEP SECURE)
- Outputs public key — copy this to `.sops.yaml`

### Key File Format

```
# created: 2026-04-15T10:00:00+08:00
# public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
AGE-SECRET-KEY-1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### Backup

Back up `~/.config/sops/age/key.txt` to a secure password manager.
Loss of this key = loss of access to all encrypted secrets.

---

## Environment Key Strategy

| Environment | Key Scope | Purpose |
|---|---|---|
| **dev** | Shared among dev team | Local development, easy rotation |
| **staging** | Separate from dev | Pre-production, restricted access |
| **prod** | Separate, most restricted | Production, minimal access |

All environments CAN use the same key for simplicity, but separation is recommended for staging/prod.

---

## Flux Integration

Flux automatically decrypts SOPS-encrypted secrets when configured with the age key.

### 1. Create Flux SOPS Secret

```bash
# Apply age key to flux-system namespace
kubectl create secret generic sops-age \
  --namespace flux-system \
  --from-file=age.agekey=~/.config/sops/age/key.txt
```

### 2. Flux Kustomization References

Flux Kustomization files should include decryption provider:

```yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: app-secrets
  namespace: flux-system
spec:
  interval: 10m
  path: ./infra/kubernetes/base
  decryption:
    provider: sops
    secretRef:
      name: sops-age
```

### 3. No Manual Decryption Needed

Once configured, Flux handles:
- Pulling encrypted secrets from Git
- Decrypting with age key
- Applying to cluster
- Reconciling on changes

---

## Local Development (No Cluster)

For quick inner loop development without a cluster:

```bash
# sops exec-env decrypts secrets and runs binary with env vars
sops exec-env \
  infra/security/sops/dev/web-bff.enc.yaml \
  -- cargo run -p web-bff
```

This:
1. Decrypts the secrets file in memory
2. Exports values as environment variables
3. Runs the specified command
4. No `.env` file is created

---

## Creating New Secrets

### 1. Edit Template

```bash
# Copy and modify template
cp infra/security/sops/templates/dev/<deployable>.yaml /tmp/<deployable>-plaintext.yaml
$EDITOR /tmp/<deployable>-plaintext.yaml
```

### 2. Encrypt

```bash
sops --encrypt /tmp/<deployable>-plaintext.yaml > infra/security/sops/dev/<deployable>.enc.yaml
```

### 3. Clean Up

```bash
rm /tmp/<deployable>-plaintext.yaml
```

### 4. Commit

```bash
git add infra/security/sops/dev/<deployable>.enc.yaml
git commit -m "Add encrypted secrets for <deployable>"
```

---

## Rotating Keys

```bash
# 1. Generate new age key
just sops-gen-age-key

# 2. Update .sops.yaml with new public key

# 3. Re-encrypt all secrets with new key
for file in infra/security/sops/**/*.enc.yaml; do
  sops updatekeys --yes "$file"
done

# 4. Update Flux secret
kubectl create secret generic sops-age \
  --namespace flux-system \
  --from-file=age.agekey=~/.config/sops/age/key.txt \
  --dry-run=client -o yaml | kubectl apply -f -

# 5. Commit changes
git add .sops.yaml infra/security/sops/
git commit -m "Rotate SOPS encryption key"
```

---

## Troubleshooting

### "No matching key for encryption"

Your age public key is not in `.sops.yaml` recipients. Run:
```bash
just sops-show-age-key
```
Then update `.sops.yaml` with the public key.

### "Decryption failed"

Ensure SOPS can find your age key:
```bash
export SOPS_AGE_KEY_FILE=~/.config/sops/age/key.txt
```

### "File already encrypted"

SOPS detects re-encryption. Use `sops --encrypt --in-place` or edit with `sops edit`.

---

## Security Notes

1. **Never commit plaintext secrets** — only `.enc.yaml` files go to Git
2. **Rotate keys regularly** — at least annually, or when team members leave
3. **Limit access** — only grant decryption key to those who need it
4. **Audit changes** — encrypted file changes are visible in Git history
