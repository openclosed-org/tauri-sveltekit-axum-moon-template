#!/usr/bin/env bash

set -euo pipefail

export SOPS_AGE_KEY_FILE="${SOPS_AGE_KEY_FILE:-${HOME}/.config/sops/age/key.txt}"

ENV="${1:-dev}"
ENV="${ENV#ENV=}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENC_FILE="$(dirname "$SCRIPT_DIR")/$ENV/counter-shared-db.enc.yaml"

if [ ! -f "$ENC_FILE" ]; then
  echo "ERROR: counter-shared-db secret not found: $ENC_FILE"
  exit 1
fi

if ! command -v sops >/dev/null 2>&1; then
  echo "ERROR: sops is not installed. Install with: mise install"
  exit 1
fi

if ! command -v yq >/dev/null 2>&1; then
  echo "ERROR: yq is not installed. Install with: mise install"
  exit 1
fi

mask_token() {
  local value="$1"
  local length=${#value}
  if [ "$length" -le 8 ]; then
    printf '***'
    return
  fi

  printf '%s***%s' "${value:0:4}" "${value: -4}"
}

require_libsql_url() {
  local name="$1"
  local value="$2"

  if [ -z "$value" ]; then
    echo "ERROR: $name is empty"
    exit 1
  fi

  if [[ "$value" == file:* ]]; then
    echo "ERROR: $name still points to local file path: $value"
    exit 1
  fi

  if [[ ! "$value" == libsql://* ]]; then
    echo "ERROR: $name must use libsql:// URL, got: $value"
    exit 1
  fi
}

require_real_token() {
  local name="$1"
  local value="$2"

  if [ -z "$value" ]; then
    echo "ERROR: $name is empty"
    exit 1
  fi

  if [ "$value" = "REPLACE_WITH_TURSO_TOKEN" ]; then
    echo "ERROR: $name still uses template placeholder"
    exit 1
  fi
}

# sops exec-env cannot handle Kubernetes Secret YAML (nested metadata/labels).
# Instead: decrypt -> extract stringData keys via yq.
PLAIN=$(sops --decrypt "$ENC_FILE") || {
  echo "ERROR: failed to decrypt $ENC_FILE"
  echo "       Check that SOPS_AGE_KEY_FILE is set and key matches .sops.yaml"
  exit 1
}

APP_TURSO_URL=$(echo "$PLAIN" | yq -r '.stringData.APP_TURSO_URL')
APP_TURSO_AUTH_TOKEN=$(echo "$PLAIN" | yq -r '.stringData.APP_TURSO_AUTH_TOKEN')
OUTBOX_DATABASE_URL=$(echo "$PLAIN" | yq -r '.stringData.OUTBOX_DATABASE_URL')
OUTBOX_TURSO_AUTH_TOKEN=$(echo "$PLAIN" | yq -r '.stringData.OUTBOX_TURSO_AUTH_TOKEN')
PROJECTOR_DATABASE_URL=$(echo "$PLAIN" | yq -r '.stringData.PROJECTOR_DATABASE_URL')
PROJECTOR_TURSO_AUTH_TOKEN=$(echo "$PLAIN" | yq -r '.stringData.PROJECTOR_TURSO_AUTH_TOKEN')

require_libsql_url APP_TURSO_URL "$APP_TURSO_URL"
require_libsql_url OUTBOX_DATABASE_URL "$OUTBOX_DATABASE_URL"
require_libsql_url PROJECTOR_DATABASE_URL "$PROJECTOR_DATABASE_URL"
require_real_token APP_TURSO_AUTH_TOKEN "$APP_TURSO_AUTH_TOKEN"
require_real_token OUTBOX_TURSO_AUTH_TOKEN "$OUTBOX_TURSO_AUTH_TOKEN"
require_real_token PROJECTOR_TURSO_AUTH_TOKEN "$PROJECTOR_TURSO_AUTH_TOKEN"

if [ "$APP_TURSO_URL" != "$OUTBOX_DATABASE_URL" ] || [ "$APP_TURSO_URL" != "$PROJECTOR_DATABASE_URL" ]; then
  echo "ERROR: shared counter DB URLs are not aligned across web-bff/outbox/projector"
  exit 1
fi

echo "counter-shared-db secret verified"
echo "  url: $APP_TURSO_URL"
echo "  app token: $(mask_token "$APP_TURSO_AUTH_TOKEN")"
echo "  outbox token: $(mask_token "$OUTBOX_TURSO_AUTH_TOKEN")"
echo "  projector token: $(mask_token "$PROJECTOR_TURSO_AUTH_TOKEN")"
