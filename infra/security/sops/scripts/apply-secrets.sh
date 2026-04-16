#!/usr/bin/env bash
# Apply SOPS-encrypted secrets to the cluster
#
# Usage: bash infra/security/sops/scripts/apply-secrets.sh <environment>
# Example: bash infra/security/sops/scripts/apply-secrets.sh dev
#
# This script:
# 1. Decrypts all .enc.yaml files for the specified environment
# 2. Applies them to the cluster
# 3. No plaintext secrets are written to disk

set -euo pipefail

export SOPS_AGE_KEY_FILE="${SOPS_AGE_KEY_FILE:-${HOME}/.config/sops/age/key.txt}"

ENV="${1:-dev}"
ENV="${ENV#ENV=}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOPS_DIR="$(dirname "$SCRIPT_DIR")/$ENV"

if [ ! -d "$SOPS_DIR" ]; then
  echo "ERROR: SOPS directory not found: $SOPS_DIR"
  exit 1
fi

echo "Applying SOPS-encrypted secrets for environment: $ENV"
echo "SOPS directory: $SOPS_DIR"

# Check if SOPS is installed
if ! command -v sops &> /dev/null; then
  echo "ERROR: sops is not installed. Install with: mise install"
  exit 1
fi

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
  echo "ERROR: kubectl is not installed or not in PATH"
  exit 1
fi

# Apply each encrypted secret
for enc_file in "$SOPS_DIR"/*.enc.yaml; do
  if [ ! -f "$enc_file" ]; then
    echo "No encrypted secrets found for environment: $ENV"
    exit 0
  fi

  secret_name=$(basename "$enc_file" .enc.yaml)
  echo "Decrypting and applying: $secret_name"

  # Decrypt and apply
  sops --decrypt "$enc_file" | kubectl apply -f -

  echo "✓ Applied: $secret_name"
done

echo ""
echo "All secrets applied for environment: $ENV"
echo "To verify: kubectl get secrets -l app.kubernetes.io/env=$ENV"
