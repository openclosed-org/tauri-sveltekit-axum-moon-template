#!/usr/bin/env bash
# Run a binary with SOPS-decrypted environment variables
#
# Usage: bash infra/security/sops/scripts/sops-run.sh <deployable> <environment> -- <command>
# Example: bash infra/security/sops/scripts/sops-run.sh web-bff dev -- cargo run -p web-bff
#
# This script:
# 1. Decrypts the secrets file for the specified deployable/environment
# 2. Also decrypts shared counter DB secrets (if present)
# 3. Exports all stringData values as environment variables
# 4. Runs the specified command with those env vars
# 5. No .env file is created

set -euo pipefail

export SOPS_AGE_KEY_FILE="${SOPS_AGE_KEY_FILE:-${HOME}/.config/sops/age/key.txt}"

DEPLOYABLE="${1:?Usage: sops-run.sh <deployable> <env> -- <command>}"
ENV="${2:-dev}"
DEPLOYABLE="${DEPLOYABLE#DEPLOYABLE=}"
ENV="${ENV#ENV=}"
shift 2

# Remove the -- separator if present
if [ "${1:-}" = "--" ]; then
  shift
fi

# Strip CMD= prefix if present (just passes CMD='cmd' as a literal string)
if [[ "${1:-}" == CMD=* ]]; then
  set -- "${1#CMD=}" "${@:2}"
fi

if [ $# -eq 0 ]; then
  set -- cargo run -p "$DEPLOYABLE"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOPS_DIR="$(dirname "$SCRIPT_DIR")/$ENV"
DEC_ENV="$SCRIPT_DIR/decrypt-env.sh"

ENC_FILE="$SOPS_DIR/${DEPLOYABLE}.enc.yaml"
SHARED_DB_FILE="$SOPS_DIR/counter-shared-db.enc.yaml"

# Build env exports from deployable secrets
ENV_EXPORTS=""

if [ -f "$ENC_FILE" ]; then
  ENV_EXPORTS=$(bash "$DEC_ENV" "$ENC_FILE")
elif [ "$DEPLOYABLE" = "counter-shared-db" ]; then
  # Special case: the deployable IS the shared DB secret
  ENV_EXPORTS=$(bash "$DEC_ENV" "$SHARED_DB_FILE")
fi

# Always layer shared counter DB secrets (if present) on top
if [ -f "$SHARED_DB_FILE" ] && [ "$DEPLOYABLE" != "counter-shared-db" ]; then
  ENV_EXPORTS="$ENV_EXPORTS
$(bash "$DEC_ENV" "$SHARED_DB_FILE")"
fi

if [ -z "$ENV_EXPORTS" ]; then
  echo "WARNING: no encrypted secrets found for $DEPLOYABLE in $ENV" >&2
  echo "         running with default environment" >&2
fi

echo "Running: $*"
echo "Environment: $ENV"
echo "Deployable: $DEPLOYABLE"
echo ""

# Execute with decrypted environment
eval "$ENV_EXPORTS" exec "$@"
