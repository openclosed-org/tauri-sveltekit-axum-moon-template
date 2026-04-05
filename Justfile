# Justfile — 统一人类/Agent 命令入口
# 工具链由 mise 管理,任务编排由 moon 负责
# Justfile 只暴露稳定、可读性高的接口
set export  # 导出环境变量到子进程

# ── 模块导入 ────────────────────────────────────────────────
import? 'justfiles/processes.just'
import? 'justfiles/skills.just'

# ── 默认行为 ────────────────────────────────────────────────

default:
    @just --list

# ── 工具链（mise）──────────────────────────────────────────

# 安装/校验所有工具版本（Rust、Node、Bun）
setup:
    mise install

# 检查工具链状态（不安装，只验证）
doctor:
    @echo "=== Toolchain ==="
    @mise current rust  || echo "MISSING: rust — run: just setup"
    @mise current node  || echo "MISSING: node — run: just setup"
    @mise current bun   || echo "MISSING: bun  — run: just setup"
    @echo ""
    @echo "=== Versions ==="
    @cargo --version 2>/dev/null || echo "cargo not in PATH"
    @node --version   2>/dev/null || echo "node not in PATH"
    @bun --version    2>/dev/null || echo "bun not in PATH"

# 安装覆盖率工具（一次性）
setup-coverage:
    @which cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov
    @which cargo-sweep  >/dev/null 2>&1 || cargo install cargo-sweep
    @rustup component add llvm-tools 2>/dev/null || true
    @echo "Coverage tools ready"

# ── 开发 ───────────────────────────────────────────────────

# 启动全栈开发（api + web）
dev:
    moon run repo:dev-fullstack

# 启动 Web 开发
dev-web:
    moon run repo:dev-web

# 启动 API 开发（axum）
dev-api:
    moon run repo:dev-api

# 启动桌面应用开发
dev-desktop:
    moon run repo:dev-desktop

# 生成类型定义并同步到前端
typegen:
    moon run repo:typegen

# ── 质量验证 ───────────────────────────────────────────────

# 完整质量门禁（fmt + lint + check + test）
verify:
    moon run repo:verify

# 格式化
fmt:
    moon run repo:fmt

# Lint
lint:
    moon run repo:lint

# ── 测试 ───────────────────────────────────────────────────

# 日常开发：快速单元测试（无覆盖率）
test:
    moon run repo:test-unit

# 使用 nextest 并行测试
test-nextest:
    moon run repo:test-nextest

# 本地覆盖率：生成 LCOV（供 CI/Codecov 用）
test-coverage:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage

# 本地覆盖率：生成 HTML 并在浏览器打开
test-coverage-html:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage-html

# 清理覆盖率产物
test-coverage-clean:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage-clean

# 运行 E2E 测试
test-e2e:
    moon run repo:test-e2e

# 运行 Tauri 桌面 E2E（tauri-driver + WebDriver）
test-desktop:
    moon run repo:test-desktop

# 运行 feature powerset 检查
test-hack:
    moon run repo:test-hack

# 运行变异测试
test-mutants:
    moon run repo:test-mutants

# 运行全部 Rust 测试
test-all-rust:
    moon run repo:test-all-rust

# 运行全部前端测试
test-all-frontend:
    moon run repo:test-all-frontend

# ── 清理 ───────────────────────────────────────────────────

# 暴力清理所有 cargo 缓存（慢，下次全量重建）
clean:
    cargo clean

# 智能清理 7 天未访问的构建缓存
clean-sweep:
    just _require cargo-sweep "cargo install cargo-sweep"
    cargo sweep --time 7

# 清理不在当前 Cargo.toml 中的依赖缓存
clean-sweep-deps:
    just _require cargo-sweep "cargo install cargo-sweep"
    cargo sweep --installed

# 仅清理覆盖率产物
clean-coverage:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    cargo llvm-cov clean

# ── 生产部署（systemd + 后端二进制）────────────────────────

# 在目标服务器上部署并启动 axum API 服务
deploy-api:
    just _deploy-api-check
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
# 需要 BIN_PATH, ENV_FILE, USER, GROUP 变量
generate-service BIN_PATH='' ENV_FILE='' USER='root' GROUP='root':
    @test -n "{{BIN_PATH}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    @test -n "{{ENV_FILE}}" || (echo 'Usage: just generate-service BIN_PATH=/path/to/binary ENV_FILE=/path/to/.env USER=appuser GROUP=appgroup' && exit 1)
    bash scripts/deploy/generate-service.sh "{{BIN_PATH}}" "{{ENV_FILE}}" "{{USER}}" "{{GROUP}}"
    @echo "Generated axum-api.service — copy to /etc/systemd/system/ and run: just deploy-api"

# ── 内部工具（不暴露给人类/Agent 直接调用）─────────────────

_require TOOL CMD:
    @which {{TOOL}} >/dev/null 2>&1 || (echo "Missing {{TOOL}} — install with: {{CMD}}" && exit 1)

_deploy-api-check:
    @systemctl is-active --quiet axum-api.service && echo "API service is running" || echo "API service will be started"
