#!/usr/bin/env bash
# Decrypt a SOPS-encrypted file and print as shell-compatible env vars
#
# Handles Kubernetes Secret YAML format by extracting stringData keys.
# Also handles flat key-value YAML (where all top-level keys are strings).
#
# Usage: eval $(bash decrypt-env.sh infra/security/sops/dev/web-bff.enc.yaml)
#
# Safe to use in a pipeline with eval — each key/value is single-quoted.
set -euo pipefail

export SOPS_AGE_KEY_FILE="${SOPS_AGE_KEY_FILE:-${HOME}/.config/sops/age/key.txt}"

ENC_FILE="${1:?Usage: decrypt-env.sh <encrypted-file>}"

PLAIN=$(sops --decrypt "$ENC_FILE" 2>/dev/null) || {
  echo "ERROR: failed to decrypt $ENC_FILE" >&2
  exit 1
}

# Try Kubernetes Secret format first (stringData.*)
if echo "$PLAIN" | yq -e '.stringData' >/dev/null 2>&1; then
  echo "$PLAIN" | yq -r 'to_entries[] | select(.value != null and (.value | type) == "string") | "export " + .key + "='"'"'" + .value + "'"'"'"'
else
  # Flat YAML: all top-level keys are strings
  echo "$PLAIN" | yq -r 'to_entries[] | select(.value != null and (.value | type) == "string") | "export " + .key + "='"'"'" + .value + "'"'"'"'
fi
