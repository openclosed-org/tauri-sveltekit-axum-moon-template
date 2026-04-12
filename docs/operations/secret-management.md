# Secret Management Guide (SOPS + age)

> Manage encrypted secrets using SOPS and age encryption.

## Prerequisites

- `sops` installed
- `age` installed (age encryption tool)

```bash
# Install on macOS
brew install sops age

# Install on Ubuntu
sudo apt install sops age
```

## Architecture

```
┌──────────────────┐
│  secrets.yaml    │ (plaintext — NEVER commit)
└────────┬─────────┘
         │ sops encrypt
         ▼
┌──────────────────┐
│  secrets.enc.yaml│ (encrypted — safe to commit)
└────────┬─────────┘
         │ sops decrypt
         ▼
┌──────────────────┐
│  Flux/K8s        │ (decrypts at runtime using age key)
└──────────────────┘
```

## Step 1: Generate age Key

```bash
# Generate age key
age-keygen -o ~/.config/sops/age/keys.txt

# Output looks like:
# Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p

# Backup the key securely
cat ~/.config/sops/age/keys.txt
```

## Step 2: Configure SOPS

Create `.sops.yaml` in the repository root:

```yaml
creation_rules:
  - path_regex: infra/security/sops/.*\.yaml$
    age: >-
      age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
```

Replace the public key with your generated key.

## Step 3: Create Secrets

### Create secrets template

Copy the template:
```bash
cp infra/security/sops/secrets.template.yaml infra/security/sops/secrets.yaml
```

### Edit the template

```yaml
# infra/security/sops/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: database-secrets
  namespace: default
type: Opaque
stringData:
  url: "libsql://file:/data/app.db?token=your-jwt-token"
  admin-password: "super-secret-password"

---
apiVersion: v1
kind: Secret
metadata:
  name: minio-secrets
  namespace: infrastructure
type: Opaque
stringData:
  access-key: "minioadmin"
  secret-key: "your-minio-secret-key"

---
apiVersion: v1
kind: Secret
metadata:
  name: oauth-secrets
  namespace: default
type: Opaque
stringData:
  google-client-id: "your-google-client-id"
  google-client-secret: "your-google-client-secret"
```

## Step 4: Encrypt Secrets

```bash
# Encrypt
sops -e -i infra/security/sops/secrets.yaml

# Verify encrypted file
cat infra/security/sops/secrets.yaml
# Should show encrypted content

# Decrypt to verify
sops -d infra/security/sops/secrets.yaml
```

## Step 5: Commit Encrypted Secrets

```bash
git add infra/security/sops/secrets.yaml
git commit -m "chore: add encrypted secrets"
git push
```

**NEVER commit plaintext secrets!**

## Step 6: Use with Flux

Flux will automatically decrypt SOPS-encrypted secrets if configured with the age key:

```bash
# Create age key secret in cluster
kubectl create secret generic sops-age \
  --namespace=flux-system \
  --from-file=age.agekey=/root/.config/sops/age/keys.txt
```

Then Flux will decrypt secrets during reconciliation.

## Step 7: Apply Secrets Manually

```bash
# Decrypt and apply
sops -d infra/security/sops/secrets.yaml | kubectl apply -f -
```

## Key Rotation

### 1. Generate New Key

```bash
age-keygen -o ~/.config/sops/age/keys-new.txt
```

### 2. Update .sops.yaml

Add the new public key to `.sops.yaml`:

```yaml
creation_rules:
  - path_regex: infra/security/sops/.*\.yaml$
    age: >-
      age1newpublickey...,age1oldpublickey...
```

### 3. Re-encrypt Secrets

```bash
# Decrypt with old key, encrypt with both keys
sops -d infra/security/sops/secrets.yaml > secrets-temp.yaml
sops -e -i secrets-temp.yaml
mv secrets-temp.yaml infra/security/sops/secrets.yaml
```

### 4. Remove Old Key

After verifying, remove the old public key from `.sops.yaml` and update all team members.

## CI/CD Integration

### GitHub Actions

```yaml
- name: Decrypt secrets
  run: |
    echo "${{ secrets.SOPS_AGE_KEY }}" > keys.txt
    export SOPS_AGE_KEY_FILE=keys.txt
    sops -d infra/security/sops/secrets.yaml | kubectl apply -f -
```

### Flux with GitHub

Store the age key as a GitHub secret `SOPS_AGE_KEY` and reference it in your Flux configuration.

## Troubleshooting

### "no matching key found"

```bash
# Check your age key
cat ~/.config/sops/age/keys.txt

# Verify public key matches .sops.yaml
grep age infra/security/sops/.sops.yaml
```

### "decryption failed"

```bash
# Check SOPS version
sops --version

# Try with explicit key file
export SOPS_AGE_KEY_FILE=~/.config/sops/age/keys.txt
sops -d infra/security/sops/secrets.yaml
```

### Accidentally committed plaintext

```bash
# IMMEDIATELY rotate the key
# 1. Generate new key
# 2. Update .sops.yaml
# 3. Re-encrypt secrets
# 4. Force push to remove plaintext from history
# 5. Invalidate all exposed credentials
```
