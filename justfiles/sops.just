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
    cargo run -p repo-tools -- dev age-key generate

# Show age public key
sops-show-age-key:
    cargo run -p repo-tools -- dev age-key show

# ── Secrets Encryption ──────────────────────────────────────

# Encrypt secrets for dev environment
sops-encrypt-dev DEPLOYABLE='web-bff':
    cargo run -p repo-tools -- secrets encrypt --deployable "{{DEPLOYABLE}}" --env dev

# Encrypt secrets for staging environment
sops-encrypt-staging DEPLOYABLE='web-bff':
    cargo run -p repo-tools -- secrets encrypt --deployable "{{DEPLOYABLE}}" --env staging

# ── Secrets Editing ─────────────────────────────────────────

# Edit encrypted secrets (opens editor with decrypted content)
sops-edit DEPLOYABLE='web-bff' ENV='dev':
    cargo run -p repo-tools -- secrets edit --deployable "{{DEPLOYABLE}}" --env "{{ENV}}"

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
    cargo run -p repo-tools -- secrets setup-flux-secret

# ── Validation ──────────────────────────────────────────────

# Validate SOPS configuration
sops-validate:
    @cargo run -p repo-tools -- secrets validate

# Verify that shared counter DB secret is ready for independent worker pods
sops-verify-counter-shared-db ENV='dev':
    @cargo run -p repo-tools -- secrets verify-counter-shared-db --env "{{ENV}}"
