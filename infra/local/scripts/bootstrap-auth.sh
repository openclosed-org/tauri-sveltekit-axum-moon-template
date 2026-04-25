#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
AUTH_COMPOSE_FILE="$ROOT_DIR/infra/docker/compose/auth.yaml"
STATE_DIR="$ROOT_DIR/infra/local/state"
GENERATED_DIR="$ROOT_DIR/infra/local/generated"
AUTH_ENV_FILE="$GENERATED_DIR/auth.env"
OPENFGA_MODEL_FILE="$ROOT_DIR/fixtures/authz-tuples/counter-model.openfga.json"
ZITADEL_ADMIN_PAT_FILE="$STATE_DIR/zitadel-admin.pat"
ZITADEL_LOGIN_CLIENT_PAT_FILE="$STATE_DIR/zitadel-login-client.pat"
ZITADEL_PROJECT_ID_FILE="$STATE_DIR/zitadel.project_id"
ZITADEL_API_APP_ID_FILE="$STATE_DIR/zitadel.api_app_id"
ZITADEL_API_CLIENT_ID_FILE="$STATE_DIR/zitadel.api_client_id"
ZITADEL_API_CLIENT_SECRET_FILE="$STATE_DIR/zitadel.api_client_secret"
ZITADEL_MACHINE_USER_ID_FILE="$STATE_DIR/zitadel.machine_user_id"
ZITADEL_MACHINE_PAT_FILE="$STATE_DIR/zitadel.machine_user.pat"
ZITADEL_MACHINE_SECRET_FILE="$STATE_DIR/zitadel.machine_user.secret"

mkdir -p "$STATE_DIR" "$GENERATED_DIR"

compose() {
  podman compose -f "$AUTH_COMPOSE_FILE" "$@"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  }
}

wait_http() {
  local url="$1"
  local name="$2"
  local retries="${3:-60}"
  local i

  for ((i=1; i<=retries; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done

  printf 'Timed out waiting for %s at %s\n' "$name" "$url" >&2
  exit 1
}

read_pat_from_container() {
  local source_path="$1"
  local target_path="$2"

  podman cp "compose_zitadel_1:$source_path" "$target_path"
}

zitadel_admin_pat() {
  tr -d '\r\n' < "$ZITADEL_ADMIN_PAT_FILE"
}

zitadel_api() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local token

  token="$(zitadel_admin_pat)"
  if [[ -n "$body" ]]; then
    curl -fsS -X "$method" "http://localhost:8082$path" \
      -H "Authorization: Bearer $token" \
      -H 'Content-Type: application/json' \
      -d "$body"
  else
    curl -fsS -X "$method" "http://localhost:8082$path" \
      -H "Authorization: Bearer $token"
  fi
}

zitadel_bootstrap() {
  local org_resp
  local org_id
  local project_resp
  local project_id
  local api_app_resp
  local api_app_id
  local api_client_id
  local api_client_secret
  local machine_resp
  local machine_user_id
  local machine_pat_resp
  local machine_secret_resp
  local suffix

  suffix="$(date +%s)"

  read_pat_from_container /zitadel/bootstrap/admin.pat "$ZITADEL_ADMIN_PAT_FILE"
  read_pat_from_container /zitadel/bootstrap/login-client.pat "$ZITADEL_LOGIN_CLIENT_PAT_FILE"

  org_resp="$(zitadel_api GET /management/v1/orgs/me)"
  org_id="$(printf '%s' "$org_resp" | jq -r '.org.id')"

  project_resp="$(zitadel_api POST /management/v1/projects '{
      "name": "local-web-bff-'"$suffix"'",
      "projectRoleAssertion": true,
      "projectRoleCheck": false,
      "hasProjectCheck": false,
      "privateLabelingSetting": "PRIVATE_LABELING_SETTING_UNSPECIFIED"
    }')"
  project_id="$(printf '%s' "$project_resp" | jq -r '.id')"

  zitadel_api POST "/management/v1/projects/$project_id/roles" '{
      "roleKey": "member",
      "displayName": "Member"
    }' >/dev/null

  api_app_resp="$(zitadel_api POST "/management/v1/projects/$project_id/apps/api" '{
      "name": "web-bff-introspection-'"$suffix"'",
      "authMethodType": "API_AUTH_METHOD_TYPE_BASIC"
    }')"
  api_app_id="$(printf '%s' "$api_app_resp" | jq -r '.appId')"
  api_client_id="$(printf '%s' "$api_app_resp" | jq -r '.clientId')"
  api_client_secret="$(printf '%s' "$api_app_resp" | jq -r '.clientSecret')"

  machine_resp="$(curl -fsS -X POST http://localhost:8082/v2/users/new \
    -H "Authorization: Bearer $(zitadel_admin_pat)" \
    -H 'Content-Type: application/json' \
    -d '{
      "organizationId": '"$(printf '%s' "$org_id" | jq -R .)"',
      "username": "local-web-bff-machine-'"$suffix"'",
      "machine": {
        "name": "Local Web BFF Machine",
        "description": "Local smoke token issuer",
        "accessTokenType": "ACCESS_TOKEN_TYPE_JWT"
      }
    }')"
  machine_user_id="$(printf '%s' "$machine_resp" | jq -r '.id')"

  machine_pat_resp="$(curl -fsS -X POST http://localhost:8082/v2/users/"$machine_user_id"/pats \
    -H "Authorization: Bearer $(zitadel_admin_pat)" \
    -H 'Content-Type: application/json' \
    -d '{"expirationDate":"2099-01-01T00:00:00Z"}')"
  machine_secret_resp="$(curl -fsS -X POST http://localhost:8082/v2/users/"$machine_user_id"/secret \
    -H "Authorization: Bearer $(zitadel_admin_pat)" \
    -H 'Content-Type: application/json')"

  printf '%s\n' "$project_id" > "$ZITADEL_PROJECT_ID_FILE"
  printf '%s\n' "$api_app_id" > "$ZITADEL_API_APP_ID_FILE"
  printf '%s\n' "$api_client_id" > "$ZITADEL_API_CLIENT_ID_FILE"
  printf '%s\n' "$api_client_secret" > "$ZITADEL_API_CLIENT_SECRET_FILE"
  printf '%s\n' "$machine_user_id" > "$ZITADEL_MACHINE_USER_ID_FILE"
  printf '%s\n' "$(printf '%s' "$machine_pat_resp" | jq -r '.token')" > "$ZITADEL_MACHINE_PAT_FILE"
  printf '%s\n' "$(printf '%s' "$machine_secret_resp" | jq -r '.clientSecret')" > "$ZITADEL_MACHINE_SECRET_FILE"

  cat >> "$AUTH_ENV_FILE" <<EOF
APP_ZITADEL_INTROSPECTION_CLIENT_ID=$api_client_id
APP_ZITADEL_INTROSPECTION_CLIENT_SECRET=$api_client_secret
EOF

  printf 'Zitadel org: %s\n' "$org_id"
  printf 'Zitadel project: %s\n' "$project_id"
  printf 'Zitadel API app: %s\n' "$api_app_id"
  printf 'Zitadel machine user: %s\n' "$machine_user_id"
  printf 'Wrote %s, %s, %s, %s\n' "$ZITADEL_PROJECT_ID_FILE" "$ZITADEL_API_CLIENT_ID_FILE" "$ZITADEL_MACHINE_USER_ID_FILE" "$ZITADEL_MACHINE_PAT_FILE"
}

openfga_bootstrap() {
  local store_resp
  local store_id
  local model_resp
  local model_id

  store_resp="$(curl -fsS -X POST http://localhost:8081/stores \
    -H 'content-type: application/json' \
    -d '{"name":"local-counter-authz"}')"
  store_id="$(printf '%s' "$store_resp" | jq -r '.id')"

  model_resp="$(curl -fsS -X POST "http://localhost:8081/stores/$store_id/authorization-models" \
    -H 'content-type: application/json' \
    --data-binary "@$OPENFGA_MODEL_FILE")"
  model_id="$(printf '%s' "$model_resp" | jq -r '.authorization_model_id')"

  cat > "$AUTH_ENV_FILE" <<EOF
APP_ZITADEL_ISSUER=http://localhost:8082
APP_ZITADEL_AUDIENCE=web-bff-local
APP_OPENFGA_ENDPOINT=http://localhost:8081
APP_OPENFGA_STORE_ID=$store_id
APP_OPENFGA_AUTHORIZATION_MODEL_ID=$model_id
EOF

  printf '%s\n' "$store_id" > "$STATE_DIR/openfga.store_id"
  printf '%s\n' "$model_id" > "$STATE_DIR/openfga.model_id"

  printf 'OpenFGA store: %s\n' "$store_id"
  printf 'OpenFGA model: %s\n' "$model_id"
  printf 'Wrote %s\n' "$AUTH_ENV_FILE"
}

zitadel_status() {
  printf 'Zitadel bootstrap volume contents:\n'
  podman volume inspect zitadel-bootstrap >/dev/null 2>&1 || true
  compose ps
}

bootstrap() {
  require_cmd podman
  require_cmd curl
  require_cmd jq

  compose up -d
  wait_http "http://localhost:8081/healthz" "OpenFGA"
  wait_http "http://localhost:8082/healthz" "Zitadel"

  openfga_bootstrap
  zitadel_bootstrap

  cat <<EOF

Local auth stack is running.

Next:
1. source infra/local/generated/auth.env
2. export APP_DATABASE_URL=file:./.data/web-bff.db
3. cargo run -p web-bff

Zitadel quick facts:
- Console: http://localhost:8082/ui/console
- Admin user: zitadel-admin@zitadel.localhost
- Admin password: Password1!

Current scope of automation:
- Zitadel instance startup is automated
- OpenFGA store/model bootstrap is automated
- local Zitadel API app scaffold is automated
- local machine-user PAT/secret scaffold is automated
- web-bff env scaffold is automated

Still manual if you need true end-user interactive login:
- create an OIDC web app for the frontend shell
- or wire the frontend shell to a real OIDC redirect/callback flow
EOF
}

case "${1:-bootstrap}" in
  up)
    compose up -d
    ;;
  down)
    compose down
    ;;
  ps|status)
    compose ps
    ;;
  logs)
    compose logs -f
    ;;
  bootstrap)
    bootstrap
    ;;
  openfga-bootstrap)
    require_cmd jq
    openfga_bootstrap
    ;;
  zitadel-status)
    zitadel_status
    ;;
  *)
    printf 'Usage: %s {up|down|ps|status|logs|bootstrap|openfga-bootstrap|zitadel-status}\n' "$0" >&2
    exit 1
    ;;
esac
