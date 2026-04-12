# SOPS Secret Management Setup Guide

This guide explains how to set up and use SOPS (Secrets OPerationS) for managing secrets in this repository.

## Overview

SOPS encrypts secret files using age encryption, allowing encrypted secrets to be safely committed to Git. Only those with the decryption key can read the secrets.

## Setup (One-Time)

### 1. Install Required Tools

```bash
# macOS
brew install age sops

# Ubuntu/Debian
sudo apt install sops
cargo install age  # or use snap/brew

# Or use mise (already in .mise.toml)
mise install
```

### 2. Generate Age Key

```bash
# Generate a new age key pair
mkdir -p ~/.config/sops/age
age-keygen -o ~/.config/sops/age/key.txt 2>&1 | tee ~/.config/sops/age/key.txt

# Output looks like:
# Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
```

**IMPORTANT**: Back up this key securely! Loss of the key means loss of access to secrets.

### 3. Update .sops.yaml

Replace `CHANGE_ME_REPLACE_WITH_YOUR_AGE_PUBLIC_KEY` in `.sops.yaml` with your actual age public key:

```bash
# Get your public key
cat ~/.config/sops/age/key.txt | grep "Public key" | awk '{print $3}'

# Edit .sops.yaml
$EDITOR infra/security/sops/.sops.yaml
```

### 4. Test Encryption

```bash
# Create a test secret file
cat > /tmp/test-secrets.yaml <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: test-secret
type: Opaque
stringData:
  MY_SECRET: super-secret-value
EOF

# Encrypt the file
sops --encrypt /tmp/test-secrets.yaml > infra/security/sops/test-secrets.enc.yaml

# Verify it's encrypted (should show encrypted data)
cat infra/security/sops/test-secrets.yaml

# Decrypt to verify
sops --decrypt infra/security/sops/test-secrets.enc.yaml

# Clean up
rm /tmp/test-secrets.yaml infra/security/sops/test-secrets.enc.yaml
```

## Creating Encrypted Secrets

### 1. Create Plaintext Secret File

```bash
cat > /tmp/my-secrets.yaml <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: app
type: Opaque
stringData:
  JWT_SECRET: "your-jwt-secret-here"
  DATABASE_URL: "libsql://localhost:8080"
  NATS_URL: "nats://localhost:4222"
  OAUTH_CLIENT_ID: "your-oauth-client-id"
  OAUTH_CLIENT_SECRET: "your-oauth-client-secret"
  S3_ACCESS_KEY: "minioadmin"
  S3_SECRET_KEY: "minioadmin"
EOF
```

### 2. Encrypt the File

```bash
sops --encrypt /tmp/my-secrets.yaml > infra/security/sops/secrets.enc.yaml
```

### 3. Commit Encrypted File

```bash
git add infra/security/sops/secrets.enc.yaml
git commit -m "Add encrypted secrets"
```

### 4. Decrypt When Needed

```bash
# View decrypted content
sops --decrypt infra/security/sops/secrets.enc.yaml

# Use in Kubernetes
sops --decrypt infra/security/sops/secrets.enc.yaml | kubectl apply -f -
```

## Using Encrypted Secrets in CI/CD

### GitHub Actions

```yaml
- name: Setup age key
  run: |
    echo "${{ secrets.SOPS_AGE_KEY }}" > /tmp/age.key
    echo "SOPS_AGE_KEY_FILE=/tmp/age.key" >> $GITHUB_ENV

- name: Deploy secrets
  run: |
    sops --decrypt infra/security/sops/secrets.enc.yaml | kubectl apply -f -
```

### GitOps (Flux)

Flux can automatically decrypt SOPS secrets if configured with the age key:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: sops-age
  namespace: flux-system
type: Opaque
stringData:
  age.agekey: |
    # Content of your age key file
```

## Rotating Keys

If you need to rotate encryption keys:

```bash
# 1. Generate new age key
age-keygen -o ~/.config/sops/age/new-key.txt

# 2. Update .sops.yaml with new public key

# 3. Re-encrypt all secrets with new key
for file in infra/security/sops/*.enc.yaml; do
  sops --update --in-place "$file"
done

# 4. Commit changes
git add infra/security/sops/
git commit -m "Rotate SOPS encryption key"
```

## Best Practices

1. **Never commit plaintext secrets** - Always encrypt before committing
2. **Backup your age key** - Store in a secure password manager
3. **Use different keys per environment** - dev, staging, prod should have separate keys
4. **Rotate keys regularly** - At least annually, or when team members leave
5. **Limit access** - Only grant decryption key to those who need it

## Troubleshooting

### "No matching key for encryption"

Your age key is not in the recipients list. Add your public key to `.sops.yaml`.

### "Decryption failed"

Make sure your age key file is pointed to by `SOPS_AGE_KEY_FILE`:

```bash
export SOPS_AGE_KEY_FILE=~/.config/sops/age/key.txt
```

### "File already encrypted"

SOPS detects re-encryption attempts. Use `--in-place` to update:

```bash
sops --encrypt --in-place secrets.yaml
```

## See Also

- [SOPS GitHub](https://github.com/getsops/sops) - Official documentation
- [age encryption](https://age-encryption.org/) - Age encryption tool
- [Flux SOPS integration](https://fluxcd.io/flux/guides/mozilla-sops/) - GitOps with SOPS
