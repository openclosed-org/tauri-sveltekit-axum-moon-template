# Justfile — 开发者日常任务编排
# 用法: just <task>
# 安装: cargo install just

# ── 默认任务 ──────────────────────────────────────────────────────
default:
  @just --list

# ── 开发环境 ──────────────────────────────────────────────────────

# 安装所有依赖
setup:
  @echo "Installing Node dependencies..."
  bun install
  @echo "Setting up Rust toolchain..."
  rustup show
  @echo "Installing dev tools..."
  cargo install cargo-nextest || true
  cargo install cargo-bloat || true
  @echo "Setup complete!"

# 启动完整开发环境（前端 + 后端）
dev:
  @echo "Starting development environment..."
  moon run app:dev & moon run server-api:dev

# 仅启动前端
dev:web:
  moon run app:dev

# 仅启动后端
dev:api:
  moon run server-api:dev

# 启动 Tauri 桌面开发
dev:tauri:
  moon run native:build

# ── 构建 ──────────────────────────────────────────────────────────

# 完整构建
build:
  moon run :build

# 仅构建前端
build:web:
  moon run app:build

# 仅构建后端
build:api:
  moon run server-api:build

# 构建 Tauri 桌面应用
build:tauri:
  moon run native:build

# ── 检查 ──────────────────────────────────────────────────────────

# 运行所有检查（lint + typecheck + test）
check:
  moon lint
  moon run :check
  moon test

# 快速检查（仅 Rust）
check:rust:
  moon run :check

# 快速检查（仅前端）
check:web:
  moon run app:check
  moon run app:lint

# ── 测试 ──────────────────────────────────────────────────────────

# 运行所有测试
test:
  moon run :test

# 仅 Rust 测试
test:rust:
  cargo nextest run --workspace

# 带覆盖率的测试
test:cov:
  cargo tarpaulin --workspace --out Html

# ── 格式化 ────────────────────────────────────────────────────────

# 检查格式
fmt:check:
  moon run :format
  moon run app:format

# 自动格式化
fmt:
  moon run :format-fix
  moon run app:format

# ── CI ────────────────────────────────────────────────────────────

# CI 完整验证
ci:
  moon run :lint
  moon run :test
  moon run :build
  moon run app:lint
  moon run app:check
  moon run app:build

# ── 清理 ──────────────────────────────────────────────────────────

# 清理所有构建产物
clean:
  @echo "Cleaning Rust artifacts..."
  cargo clean
  @echo "Cleaning Node artifacts..."
  rm -rf node_modules
  @echo "Cleaning moon cache..."
  rm -rf .moon/cache
  @echo "Clean complete."

# ── 工具 ──────────────────────────────────────────────────────────

# 查看 moon 项目列表
projects:
  moon projects

# 查看 moon 任务列表
tasks:
  moon tasks

# 二进制体积分析
bloat:
  moon run :bloat

# 更新依赖
update:
  cargo update
  bun update

# 数据库迁移（如有）
migrate:
  @echo "Running database migrations..."

# 生成 API 契约
generate:contracts:
  @echo "Generating API contracts..."
