# Deploy — Infrastructure deployment (Podman Compose + k3s)
#
# Dev:   just deploy-dev            (starts podman compose)
# Prod:  just deploy-prod           (deploys to k3s)
# Legacy: just deploy-api           (systemd-based, kept for backward compat)

# ── Podman Compose (Development) ──────────────────────────────

# Start all dev services (API + SurrealDB + Gateway + Web)
deploy-dev:
    @echo "Starting dev infrastructure (Podman Compose)..."
    podman compose -f infra/docker/compose/app.yaml up -d
    @echo "Dev services started. Check health: podman compose -f infra/docker/compose/app.yaml ps"

# Start dev services with observability stack
deploy-dev-full:
    @echo "Starting dev infrastructure + observability..."
    podman compose -f infra/docker/compose/app.yaml up -d
    podman compose -f infra/docker/compose/observability.yaml up -d
    @echo "OpenObserve UI: http://localhost:5080 (admin@localhost / admin)"

# Start local auth stack (Zitadel + OpenFGA)
auth-up:
    @echo "Starting local auth stack..."
    bash infra/local/scripts/bootstrap-auth.sh up

# Bootstrap local auth stack and write runtime env scaffold
auth-bootstrap:
    @echo "Bootstrapping local auth stack..."
    bash infra/local/scripts/bootstrap-auth.sh bootstrap

# Stop local auth stack
auth-down:
    @echo "Stopping local auth stack..."
    bash infra/local/scripts/bootstrap-auth.sh down

# Inspect local auth stack status
auth-status:
    bash infra/local/scripts/bootstrap-auth.sh status

# Follow local auth stack logs
auth-logs:
    bash infra/local/scripts/bootstrap-auth.sh logs

# Stop all dev services
stop-dev:
    @echo "Stopping dev infrastructure..."
    podman compose -f infra/docker/compose/app.yaml down
    podman compose -f infra/docker/compose/observability.yaml down 2>/dev/null || true
    @echo "Dev services stopped"

# View dev service logs
logs-dev SERVICE='api':
    podman compose -f infra/docker/compose/app.yaml logs -f {{SERVICE}}

# ── k3s (Production) ─────────────────────────────────────────

# Deploy to k3s cluster
deploy-prod ENV='dev':
    @echo "Deploying to k3s environment: {{ENV}}..."
    bash infra/k3s/scripts/deploy.sh {{ENV}}

# Deploy to k3s (dry run — preview changes)
deploy-prod-dry-run ENV='dev':
    @echo "Dry run: deploying to k3s environment: {{ENV}}..."
    bash infra/k3s/scripts/deploy.sh {{ENV}} --dry-run=client

# Bootstrap a fresh k3s cluster on a VPS
deploy-bootstrap-k3s:
    @echo "Bootstrapping k3s on the current machine..."
    @echo "WARNING: This requires root access and will install system packages."
    sudo bash infra/k3s/scripts/bootstrap-k3s.sh

# ── Legacy systemd deployment (backward compatible) ──────────

# 在目标服务器上部署并启动 axum API 服务
deploy-api:
    @systemctl is-active --quiet axum-api.service && echo "API service is running" || echo "API service will be started"
    sudo systemctl daemon-reload
    sudo systemctl enable axum-api.service
    sudo systemctl restart axum-api.service
    sudo systemctl status axum-api --no-pager

# 停止 API 服务
stop-api:
    sudo systemctl stop axum-api.service
    @echo "API service stopped"

# 查看 API 服务日志
logs-api:
    sudo journalctl -u axum-api.service -f --no-pager

# 检查 storage-turso 编译
check-storage-turso:
    cargo check -p web-bff -p admin-bff

# 构建桌面 E2E 二进制
build-desktop-bin:
    cargo build -p native-tauri --profile e2e --features e2e-testing

# 生成 systemd service 文件（需在目标服务器执行）
# 需要 BIN_PATH, ENV_FILE, USER, GROUP 变量
generate-service BIN_PATH='' ENV_FILE='' USER='root' GROUP='root':
    @test -n "{{BIN_PATH}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    @test -n "{{ENV_FILE}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    bash scripts/deploy/generate-service.sh "{{BIN_PATH}}" "{{ENV_FILE}}" "{{USER}}" "{{GROUP}}"
    @echo "Generated axum-api.service — copy to /etc/systemd/system/ and run: just deploy-api"
