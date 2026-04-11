#!/bin/sh
# docker-entrypoint-gateway.sh
# Starts both Pingora gateway and static-web-server in the same container.
#
# - static-web-server runs on port 3002 (serves SvelteKit build output)
# - Pingora gateway runs on port 3000 (reverse proxy)

set -e

WEB_ROOT="${WEB_ROOT:-/app/web-build}"
WEB_PORT="${WEB_PORT:-3002}"

echo "[entrypoint] Starting static-web-server on :${WEB_PORT} (root: ${WEB_ROOT})"
echo "[entrypoint] Starting Pingora gateway on ${BIND:-0.0.0.0:3000}"

# Start static-web-server in background
/usr/local/bin/static-web-server \
    --host 0.0.0.0 \
    --port "${WEB_PORT}" \
    --root "${WEB_ROOT}" \
    --page-fallback "${WEB_ROOT}/index.html" \
    --compression \
    --security-headers \
    --log-level info &
SWS_PID=$!

# Start Pingora gateway in foreground (takes over this process)
exec /usr/local/bin/pingora-gateway
