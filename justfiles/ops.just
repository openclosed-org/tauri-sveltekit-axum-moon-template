# Operations control plane
#
# This file owns environment-coupled actions: deploy, migration, and host-level
# service management. These commands are intentionally separated from dev.just so
# risky or low-frequency operational paths do not pollute the default local loop.

# 部署到 k3s 集群
deploy-prod ENV='dev':
    @echo "Deploying to k3s environment: {{ENV}}..."
    cargo run -p repo-tools -- infra k3s deploy --env "{{ENV}}"

# 预览 k3s 部署变更
deploy-prod-dry-run ENV='dev':
    @echo "Dry run: deploying to k3s environment: {{ENV}}..."
    cargo run -p repo-tools -- infra k3s deploy --env "{{ENV}}" --dry-run client

# 初始化一台新的 k3s VPS
deploy-bootstrap-k3s:
    cargo run -p repo-tools -- infra k3s bootstrap

# 规划裸 VPS 的工具链 bootstrap
deploy-bootstrap-vps:
    cargo run -p repo-tools -- ops bootstrap-vps --plan

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

# 生成 systemd service 文件（需在目标服务器执行）
generate-service BIN_PATH='' ENV_FILE='' USER='root' GROUP='root':
    @test -n "{{BIN_PATH}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    @test -n "{{ENV_FILE}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    cargo run -p repo-tools -- generate-service --bin-path "{{BIN_PATH}}" --env-file "{{ENV_FILE}}" --user "{{USER}}" --group "{{GROUP}}"
    @echo "Generated axum-api.service — copy to /etc/systemd/system/ and run: just deploy-api"

# 运行所有待执行 migration
migrate-up:
    @echo "=== Running migrations ==="
    cargo run -p repo-tools -- ops migrate --env local --direction up --dry-run

# 回滚上一条 migration
migrate-down:
    cargo run -p repo-tools -- ops migrate --env local --direction down --dry-run

# 查看 migration 状态
migrate-status:
    cargo run -p repo-tools -- ops migrate --env local --direction status --dry-run

# 创建新的 migration 文件
migrate-create NAME='':
    @if [ -z "{{NAME}}" ]; then \
        echo "Usage: just migrate-create add_counter_index"; \
        exit 1; \
    fi
    @TIMESTAMP=$(date +%Y%m%d%H%M%S); \
    MIGRATION_FILE="ops/migrations/api/${TIMESTAMP}_{{NAME}}.sql"; \
    mkdir -p ops/migrations/api; \
    touch "$MIGRATION_FILE"; \
    echo "Created migration: $MIGRATION_FILE"

# 重置数据库（仅限开发环境，危险操作）
migrate-reset:
    cargo run -p repo-tools -- ops migrate --env local --direction reset --dry-run
