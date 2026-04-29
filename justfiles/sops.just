# SOPS — Secrets management (SOPS + age + Flux)
#
# Backend binaries consume standard environment variables.
# Environment variables are injected via SOPS/Kustomize/Flux, NOT from .env files.
#
# Quick reference:
#   just sops-gen-age-key          # Generate age key (first-time setup)
#   just sops-show-age-key         # Show public key
#   just sops-encrypt-dev <name>   # Encrypt secrets for dev
#   just sops-edit <name> <env>    # Edit encrypted secrets
#   just sops-run <name>           # Run binary with decrypted env vars
#   just sops-reconcile <env>      # Apply secrets to cluster

# ── Age Key Management ──────────────────────────────────────

# Generate a new age key pair
sops-gen-age-key:
    @mkdir -p ~/.config/sops/age
    @if [ -f ~/.config/sops/age/key.txt ]; then \
        echo "WARNING: Age key already exists at ~/.config/sops/age/key.txt"; \
        echo "Current public key:"; \
        grep "# public key:" ~/.config/sops/age/key.txt | sed 's/# public key: //' || true; \
        echo ""; \
        echo "Set confirm=y to overwrite: just sops-gen-age-key confirm=y"; \
        exit 1; \
    fi
    @age-keygen -o ~/.config/sops/age/key.txt
    @echo ""
    @echo "✓ Age key generated: ~/.config/sops/age/key.txt"
    @echo "Public key:"
    @grep "# public key:" ~/.config/sops/age/key.txt | sed 's/# public key: //'
    @echo ""
    @echo "Next steps:"
    @echo "  1. Copy the public key above"
    @echo "  2. Update .sops.yaml with the public key for each environment"
    @echo "  3. Back up ~/.config/sops/age/key.txt securely"

# Show age public key
sops-show-age-key:
    @if [ ! -f ~/.config/sops/age/key.txt ]; then \
        echo "ERROR: Age key not found. Generate one with: just sops-gen-age-key"; \
        exit 1; \
    fi
    @echo "Age public key:"
    @grep "# public key:" ~/.config/sops/age/key.txt | sed 's/# public key: //'
    @echo ""
    @echo "Key file: ~/.config/sops/age/key.txt"

# ── Secrets Encryption ──────────────────────────────────────

# Encrypt secrets for dev environment
sops-encrypt-dev DEPLOYABLE='web-bff':
    @echo "Encrypting secrets for deployable: {{DEPLOYABLE}} (dev)"
    @sops --encrypt --input-type yaml --output-type yaml \
        "infra/security/sops/templates/dev/{{DEPLOYABLE}}.yaml" \
        > "infra/security/sops/dev/{{DEPLOYABLE}}.enc.yaml"
    @echo "✓ Encrypted: infra/security/sops/dev/{{DEPLOYABLE}}.enc.yaml"
    @echo "Commit this file to Git"

# Encrypt secrets for staging environment
sops-encrypt-staging DEPLOYABLE='web-bff':
    @echo "Encrypting secrets for deployable: {{DEPLOYABLE}} (staging)"
    @sops --encrypt "infra/security/sops/templates/staging/{{DEPLOYABLE}}.yaml" > "infra/security/sops/staging/{{DEPLOYABLE}}.enc.yaml"
    @echo "✓ Encrypted: infra/security/sops/staging/{{DEPLOYABLE}}.enc.yaml"

# ── Secrets Editing ─────────────────────────────────────────

# Edit encrypted secrets (opens editor with decrypted content)
sops-edit DEPLOYABLE='web-bff' ENV='dev':
    @echo "Editing: {{DEPLOYABLE}} ({{ENV}})"
    @sops "infra/security/sops/{{ENV}}/{{DEPLOYABLE}}.enc.yaml"
    @echo "✓ Saved (encrypted in place)"

# ── Local Development (No Cluster) ──────────────────────────

# Run binary with SOPS-decrypted environment variables
# This is the ONLY allowed non-cluster dev path
# Does NOT create .env files
sops-run DEPLOYABLE='web-bff' ENV='dev' CMD='':
    @cargo run -p repo-tools -- secrets run --deployable "{{DEPLOYABLE}}" --env "{{ENV}}" -- {{CMD}}

# ── Cluster Reconciliation ──────────────────────────────────

# Apply SOPS-encrypted secrets to k3s cluster
sops-reconcile ENV='dev':
    @cargo run -p repo-tools -- secrets reconcile --env "{{ENV}}"

# ── Flux Integration ────────────────────────────────────────

# Create Flux SOPS secret (age key in cluster)
sops-setup-flux-secret:
    @if [ ! -f ~/.config/sops/age/key.txt ]; then \
        echo "ERROR: Age key not found. Generate one with: just sops-gen-age-key"; \
        exit 1; \
    fi
    @echo "Creating Flux SOPS secret in flux-system namespace"
    @kubectl create secret generic sops-age \
        --namespace flux-system \
        --from-file=age.agekey=~/.config/sops/age/key.txt \
        --dry-run=client -o yaml | kubectl apply -f -
    @echo "✓ Flux SOPS secret created"
    @echo "Flux will now be able to decrypt SOPS-encrypted secrets"

# ── Validation ──────────────────────────────────────────────

# Validate SOPS configuration
sops-validate:
    @cargo run -p repo-tools -- secrets validate

# Verify that shared counter DB secret is ready for independent worker pods
sops-verify-counter-shared-db ENV='dev':
    @cargo run -p repo-tools -- secrets verify-counter-shared-db --env "{{ENV}}"
